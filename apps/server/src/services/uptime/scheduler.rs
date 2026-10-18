//! Uptime monitoring scheduler.
//!
//! Runs a 1-second tick loop that picks up uptime_monitors due for checking,
//! dispatches probes, updates state machine, and fires alerts.

use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Semaphore;
use tokio::time::{interval, Duration};
use uuid::Uuid;

use crate::config::UptimeConfig;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::uptime_monitor::UptimeMonitor;
use crate::services::notification::create_dispatcher;

use super::probes::{run_http_probe, run_tcp_probe};
use super::state_machine::{transition, AlertAction, StateMachineConfig};

// =============================================================================
// Scheduler entry point
// =============================================================================

/// Runs the uptime monitoring scheduler forever.
///
/// This is designed to be spawned via `tokio::spawn`. Errors in individual
/// monitor checks are logged and do not stop the loop.
pub async fn run_scheduler(
    pool: DbPool,
    config: UptimeConfig,
    http_client: reqwest::Client,
    dashboard_url: String,
) {
    // Small startup jitter to spread load when multiple instances start together
    let jitter_ms = (rand::random::<u16>() as u64) % 5000;
    tokio::time::sleep(Duration::from_millis(jitter_ms)).await;

    log::info!(
        "Uptime scheduler started (max_concurrent: {})",
        config.max_concurrent_checks
    );

    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_checks));
    let mut tick = interval(Duration::from_secs(1));

    loop {
        tick.tick().await;

        let now = Utc::now();

        // Fetch uptime_monitors that are due for checking
        match fetch_due_monitors(&pool, now).await {
            Ok(uptime_monitors) => {
                for monitor in uptime_monitors {
                    let pool = pool.clone();
                    let client = http_client.clone();
                    let sem = semaphore.clone();
                    let dashboard_url = dashboard_url.clone();

                    tokio::spawn(async move {
                        let _permit = match sem.acquire().await {
                            Ok(p) => p,
                            Err(_) => {
                                log::error!(
                                    "Semaphore closed, stopping check for monitor {}",
                                    monitor.id
                                );
                                return;
                            }
                        };

                        if let Err(e) = check_monitor(&pool, &client, &monitor, &dashboard_url).await {
                            log::error!("Uptime monitor check failed for {}: {}", monitor.id, e);
                        }
                    });
                }
            }
            Err(e) => {
                log::error!("Failed to fetch due uptime_monitors: {}", e);
            }
        }
    }
}

/// Runs the cleanup task forever (deletes old uptime_checks on a 24h cycle).
pub async fn run_cleanup_task(pool: DbPool, retention_days: u32) {
    log::info!(
        "Uptime cleanup task started (retention: {} days)",
        retention_days
    );

    loop {
        tokio::time::sleep(Duration::from_secs(86400)).await;

        if let Err(e) = cleanup_old_checks(&pool, retention_days).await {
            log::error!("Failed to clean up old monitor checks: {}", e);
        } else {
            log::info!(
                "Cleaned up monitor checks older than {} days",
                retention_days
            );
        }
    }
}

// =============================================================================
// UptimeMonitor check pipeline
// =============================================================================

/// Fetches uptime_monitors that are enabled and due for their next check.
///
/// Atomically bumps `next_check_at` to `now + interval` before returning so
/// that a slow probe (>1 s) cannot be re-fetched on the next tick and counted
/// as a second independent failure — which would bypass `fail_threshold`.
async fn fetch_due_monitors(pool: &DbPool, now: chrono::DateTime<Utc>) -> AppResult<Vec<UptimeMonitor>> {
    #[cfg(feature = "postgres")]
    let uptime_monitors = sqlx::query_as::<_, UptimeMonitor>(
        r#"
        WITH to_claim AS (
            SELECT ms.monitor_id, m.interval_secs
            FROM uptime_monitor_states ms
            JOIN uptime_monitors m ON m.id = ms.monitor_id
            WHERE m.enabled = TRUE AND ms.next_check_at <= $1
            ORDER BY ms.next_check_at
            LIMIT 100
            FOR UPDATE OF ms SKIP LOCKED
        ),
        claimed AS (
            UPDATE uptime_monitor_states ms
            SET next_check_at = $1 + make_interval(secs => to_claim.interval_secs)
            FROM to_claim
            WHERE ms.monitor_id = to_claim.monitor_id
            RETURNING ms.monitor_id
        )
        SELECT m.id, m.project_id, m.name, m.check_type, m.url, m.interval_secs, m.timeout_secs,
               m.expected_status, m.fail_threshold, m.recovery_threshold,
               m.repeat_interval_secs, m.enabled, m.created_at, m.updated_at
        FROM uptime_monitors m
        JOIN claimed ON m.id = claimed.monitor_id
        "#,
    )
    .bind(now)
    .fetch_all(pool)
    .await?;

    #[cfg(feature = "sqlite")]
    let uptime_monitors = {
        let mut tx = pool.begin().await?;
        let rows = sqlx::query_as::<_, UptimeMonitor>(
            r#"
            SELECT m.id, m.project_id, m.name, m.check_type, m.url, m.interval_secs, m.timeout_secs,
                   m.expected_status, m.fail_threshold, m.recovery_threshold,
                   m.repeat_interval_secs, m.enabled, m.created_at, m.updated_at
            FROM uptime_monitors m
            JOIN uptime_monitor_states ms ON ms.monitor_id = m.id
            WHERE m.enabled = 1
              AND ms.next_check_at <= $1
            LIMIT 100
            "#,
        )
        .bind(now.naive_utc().to_string())
        .fetch_all(&mut *tx)
        .await?;

        for monitor in &rows {
            let next = now + chrono::Duration::seconds(monitor.interval_secs as i64);
            sqlx::query("UPDATE uptime_monitor_states SET next_check_at = $1 WHERE monitor_id = $2")
                .bind(next.naive_utc().to_string())
                .bind(monitor.id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        rows
    };

    Ok(uptime_monitors)
}

/// Runs the full check pipeline for a single monitor:
/// 1. Run probe
/// 2. Update state in a single transaction (B2: atomicity)
/// 3. Dispatch alert if needed
async fn check_monitor(
    pool: &DbPool,
    http_client: &reqwest::Client,
    monitor: &UptimeMonitor,
    dashboard_url: &str,
) -> AppResult<()> {
    // 1. Run probe
    let probe = run_probe(http_client, monitor).await;

    let now = Utc::now();
    let next_check = now + chrono::Duration::seconds(monitor.interval_secs as i64);

    // 2. State update transaction (B2 fix: single tx with SELECT FOR UPDATE)
    let (action, new_state_str) = update_state_transactional(
        pool,
        monitor,
        probe.ok,
        probe.latency_ms,
        probe.error.as_deref(),
        now,
        next_check,
    )
    .await?;

    log::debug!(
        "Uptime monitor {} checked: ok={}, state={}, action={:?}",
        monitor.id,
        probe.ok,
        new_state_str,
        action
    );

    // 3. Dispatch alert
    if action != AlertAction::None {
        if let Err(e) = dispatch_alert(pool, monitor, &action, now, dashboard_url).await {
            log::error!("Alert dispatch failed for monitor {}: {}", monitor.id, e);
        }
    }

    Ok(())
}

/// Dispatches a probe using the appropriate probe type.
async fn run_probe(http_client: &reqwest::Client, monitor: &UptimeMonitor) -> super::probes::ProbeResult {
    match monitor.check_type.as_str() {
        "http" => run_http_probe(http_client, monitor).await,
        "tcp" => run_tcp_probe(monitor).await,
        other => {
            log::warn!("Unknown check_type '{}' for monitor {}", other, monitor.id);
            super::probes::ProbeResult {
                ok: false,
                latency_ms: 0,
                error: Some(format!("unknown check_type: {other}")),
            }
        }
    }
}

/// Updates monitor state in a single transaction (B2 requirement).
///
/// Returns `(AlertAction, new_state_string)`.
async fn update_state_transactional(
    pool: &DbPool,
    monitor: &UptimeMonitor,
    probe_ok: bool,
    latency_ms: u64,
    error_msg: Option<&str>,
    now: chrono::DateTime<Utc>,
    next_check: chrono::DateTime<Utc>,
) -> AppResult<(AlertAction, String)> {
    let mut tx = pool.begin().await?;

    // Lock and read the state row
    #[cfg(feature = "postgres")]
    let state_opt = sqlx::query_as::<_, crate::models::uptime_monitor::UptimeMonitorState>(
        r#"
        SELECT monitor_id, state, fail_counter, recovery_counter,
               last_check_at, next_check_at, alerted_down_at, last_alerted_at,
               alert_count, incident_id
        FROM uptime_monitor_states
        WHERE monitor_id = $1
        FOR UPDATE
        "#,
    )
    .bind(monitor.id)
    .fetch_optional(&mut *tx)
    .await?;

    #[cfg(feature = "sqlite")]
    let state_opt = sqlx::query_as::<_, crate::models::uptime_monitor::UptimeMonitorState>(
        r#"
        SELECT monitor_id, state, fail_counter, recovery_counter,
               last_check_at, next_check_at, alerted_down_at, last_alerted_at,
               alert_count, incident_id
        FROM uptime_monitor_states
        WHERE monitor_id = $1
        "#,
    )
    .bind(monitor.id.to_string())
    .fetch_optional(&mut *tx)
    .await?;

    let state = match state_opt {
        Some(s) => s,
        None => {
            tx.rollback().await.ok();
            return Err(AppError::NotFound(format!(
                "No state row for monitor {}",
                monitor.id
            )));
        }
    };

    let sm_config = StateMachineConfig {
        fail_threshold: monitor.fail_threshold,
        recovery_threshold: monitor.recovery_threshold,
        repeat_interval_secs: monitor.repeat_interval_secs as i64,
    };

    let (new_state_enum, new_fail, new_recovery, action) =
        transition(&state, &sm_config, probe_ok, now);

    let new_state_str = new_state_enum.to_string();

    // Guard: skip FireDown if we already have alerted_down_at set (server restart case)
    let effective_action = if action == AlertAction::FireDown && state.alerted_down_at.is_some() {
        AlertAction::None
    } else {
        action
    };

    // Determine alerted_down_at update
    let new_alerted_down_at = match &effective_action {
        AlertAction::FireDown => Some(now),
        AlertAction::FireRecovery => None, // clear on recovery
        _ => state.alerted_down_at,
    };

    let new_last_alerted_at = match &effective_action {
        AlertAction::None => state.last_alerted_at,
        _ => Some(now),
    };

    let new_alert_count = match &effective_action {
        AlertAction::None => state.alert_count,
        _ => state.alert_count + 1,
    };

    // Update state row
    #[cfg(feature = "postgres")]
    sqlx::query(
        r#"
        UPDATE uptime_monitor_states
        SET state = $2,
            fail_counter = $3,
            recovery_counter = $4,
            last_check_at = $5,
            next_check_at = $6,
            alerted_down_at = $7,
            last_alerted_at = $8,
            alert_count = $9
        WHERE monitor_id = $1
        "#,
    )
    .bind(monitor.id)
    .bind(&new_state_str)
    .bind(new_fail)
    .bind(new_recovery)
    .bind(now)
    .bind(next_check)
    .bind(new_alerted_down_at)
    .bind(new_last_alerted_at)
    .bind(new_alert_count)
    .execute(&mut *tx)
    .await?;

    #[cfg(feature = "sqlite")]
    sqlx::query(
        r#"
        UPDATE uptime_monitor_states
        SET state = $2,
            fail_counter = $3,
            recovery_counter = $4,
            last_check_at = $5,
            next_check_at = $6,
            alerted_down_at = $7,
            last_alerted_at = $8,
            alert_count = $9
        WHERE monitor_id = $1
        "#,
    )
    .bind(monitor.id.to_string())
    .bind(&new_state_str)
    .bind(new_fail)
    .bind(new_recovery)
    .bind(now.naive_utc().to_string())
    .bind(next_check.naive_utc().to_string())
    .bind(new_alerted_down_at.map(|t| t.naive_utc().to_string()))
    .bind(new_last_alerted_at.map(|t| t.naive_utc().to_string()))
    .bind(new_alert_count)
    .execute(&mut *tx)
    .await?;

    // Insert check record
    let check_id = Uuid::new_v4();
    let status: i32 = if probe_ok { 1 } else { 0 };

    #[cfg(feature = "postgres")]
    sqlx::query(
        r#"
        INSERT INTO uptime_checks (id, monitor_id, checked_at, status, latency_ms, error_message)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(check_id)
    .bind(monitor.id)
    .bind(now)
    .bind(status)
    .bind(latency_ms as i32)
    .bind(error_msg)
    .execute(&mut *tx)
    .await?;

    #[cfg(feature = "sqlite")]
    sqlx::query(
        r#"
        INSERT INTO uptime_checks (id, monitor_id, checked_at, status, latency_ms, error_message)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(check_id.to_string())
    .bind(monitor.id.to_string())
    .bind(now.naive_utc().to_string())
    .bind(status)
    .bind(latency_ms as i32)
    .bind(error_msg)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((effective_action, new_state_str))
}

/// Dispatches an alert for a monitor state change.
async fn dispatch_alert(
    pool: &DbPool,
    monitor: &UptimeMonitor,
    action: &AlertAction,
    _now: chrono::DateTime<Utc>,
    dashboard_url: &str,
) -> AppResult<()> {
    let alert_type = match action {
        AlertAction::FireDown => "monitor_down",
        AlertAction::FireRecovery => "monitor_recovery",
        AlertAction::FireRepeat => "monitor_down_repeat",
        AlertAction::None => return Ok(()),
    };

    // Load linked channels
    let channels = load_monitor_channels(pool, monitor.id).await?;

    if channels.is_empty() {
        log::debug!("No channels configured for monitor {}", monitor.id);
        return Ok(());
    }

    let project = crate::services::ProjectService::get_by_id(pool, monitor.project_id).await?;

    for (provider_type_str, credentials, channel_name) in channels {
        let provider_type = match provider_type_str.as_str() {
            "webhook" => crate::models::ProviderType::Webhook,
            "custom_webhook" => crate::models::ProviderType::CustomWebhook,
            "email" => crate::models::ProviderType::Email,
            "slack" => crate::models::ProviderType::Slack,
            other => {
                log::warn!("Unknown provider type '{}', skipping", other);
                continue;
            }
        };

        let dispatcher = create_dispatcher(provider_type.clone());

        // Build a minimal AlertPayload for the uptime alert
        let payload = build_uptime_alert_payload(monitor, &project, alert_type, dashboard_url);

        // The dispatcher takes an AlertIntegration, not the raw tuple
        // load_monitor_channels returned — build a synthetic one from it.
        // No per-rule routing_override exists for uptime monitors, so that
        // argument is just `{}`.
        let synthetic_integration = crate::models::AlertIntegration {
            id: 0, // not used by dispatcher
            name: channel_name.clone(),
            provider_type,
            credentials,
            is_enabled: true,
            failure_count: 0,
            last_failure_at: None,
            last_failure_message: None,
            last_success_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = dispatcher
            .send(&synthetic_integration, &serde_json::json!({}), &payload)
            .await;

        if !result.success {
            log::warn!(
                "Alert dispatch to {} for monitor {} failed: {:?}",
                channel_name,
                monitor.id,
                result.error_message
            );
        } else {
            log::info!(
                "Alert ({}) sent to {} for monitor {}",
                alert_type,
                channel_name,
                monitor.id
            );
        }
    }

    Ok(())
}

/// Loads notification channels linked to a monitor.
///
/// Returns `Vec<(provider_type, credentials_json, name)>`.
async fn load_monitor_channels(
    pool: &DbPool,
    monitor_id: Uuid,
) -> AppResult<Vec<(String, serde_json::Value, String)>> {
    #[cfg(feature = "postgres")]
    let rows: Vec<(String, serde_json::Value, String)> = sqlx::query_as(
        r#"
        SELECT nc.provider_type, nc.credentials, nc.name
        FROM alert_integrations nc
        JOIN uptime_monitor_alert_channels mac ON nc.id = mac.integration_id
        WHERE mac.monitor_id = $1 AND nc.is_enabled = TRUE
        "#,
    )
    .bind(monitor_id)
    .fetch_all(pool)
    .await?;

    #[cfg(feature = "sqlite")]
    let rows: Vec<(String, serde_json::Value, String)> = sqlx::query_as(
        r#"
        SELECT nc.provider_type, nc.credentials, nc.name
        FROM alert_integrations nc
        JOIN uptime_monitor_alert_channels mac ON nc.id = mac.integration_id
        WHERE mac.monitor_id = $1 AND nc.is_enabled = 1
        "#,
    )
    .bind(monitor_id.to_string())
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Builds a minimal AlertPayload for an uptime alert.
fn build_uptime_alert_payload(
    monitor: &UptimeMonitor,
    project: &crate::models::Project,
    alert_type: &str,
    dashboard_url: &str,
) -> crate::models::AlertPayload {
    use crate::models::{AlertPayload, IssueInfo, ProjectInfo};

    AlertPayload {
        alert_id: format!("uptime-{}-{}", monitor.id, Utc::now().timestamp_millis()),
        alert_type: alert_type.to_string(),
        triggered_at: Utc::now(),
        project: ProjectInfo {
            id: project.id,
            name: project.name.clone(),
            slug: project.slug.clone(),
        },
        issue: IssueInfo {
            id: monitor.id.to_string(),
            short_id: format!("MON-{}", &monitor.id.to_string()[..8]),
            title: format!("Monitor '{}' is {}", monitor.name, alert_type),
            level: Some("error".to_string()),
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            event_count: 1,
        },
        issue_url: format!(
            "{}/projects/{}/uptime-monitors/{}",
            dashboard_url.trim_end_matches('/'),
            project.id,
            monitor.id
        ),
        actor: "Rustrak Uptime".to_string(),
    }
}

/// Deletes monitor check records older than `retention_days`.
async fn cleanup_old_checks(pool: &DbPool, retention_days: u32) -> AppResult<()> {
    let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);

    #[cfg(feature = "postgres")]
    sqlx::query("DELETE FROM uptime_checks WHERE checked_at < $1")
        .bind(cutoff)
        .execute(pool)
        .await?;

    #[cfg(feature = "sqlite")]
    sqlx::query("DELETE FROM uptime_checks WHERE checked_at < $1")
        .bind(cutoff.naive_utc().to_string())
        .execute(pool)
        .await?;

    Ok(())
}
