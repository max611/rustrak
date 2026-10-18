//! Routes for the uptime monitoring API (active HTTP/TCP probing — distinct
//! from `routes::monitors`' Sentry Crons check-ins). Ported from upstream's
//! closed, unmerged PR #81; project-scoped under
//! `/api/projects/{project_id}/uptime-monitors` (the original had no
//! project concept at all) using the same `access::require` + ownership-
//! check pattern `routes::issues` and `routes::monitors` already use.

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::auth::ApiActor;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::uptime_monitor::{AssignUptimeMonitorChannels, CreateUptimeMonitor, UpdateUptimeMonitor};
use crate::services::access::{self, Action};
use crate::services::uptime::probes::{run_http_probe, run_tcp_probe};
use crate::services::uptime_monitor::UptimeMonitorService;

/// Loads a monitor and 404s if it doesn't belong to `project_id` — the same
/// ownership-check shape `routes::issues::update_issue` uses for issues.
async fn get_owned_monitor(
    pool: &DbPool,
    project_id: i32,
    id: Uuid,
) -> AppResult<crate::models::UptimeMonitor> {
    let monitor = UptimeMonitorService::get(pool, id).await?;
    if monitor.project_id != project_id {
        return Err(AppError::NotFound(format!("UptimeMonitor {id} not found")));
    }
    Ok(monitor)
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /api/projects/{project_id}/uptime-monitors — list a project's monitors
pub async fn list_monitors(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let project_id = path.into_inner();
    access::require(
        pool.get_ref(),
        actor.is_admin(),
        actor.user_id(),
        project_id,
        Action::ViewProject,
    )
    .await?;

    let monitors = UptimeMonitorService::list(pool.get_ref(), project_id).await?;
    Ok(HttpResponse::Ok().json(monitors))
}

/// GET /api/projects/{project_id}/uptime-monitors/{id}
pub async fn get_monitor(
    pool: web::Data<DbPool>,
    path: web::Path<(i32, Uuid)>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let (project_id, id) = path.into_inner();
    access::require(
        pool.get_ref(),
        actor.is_admin(),
        actor.user_id(),
        project_id,
        Action::ViewProject,
    )
    .await?;

    let monitor = get_owned_monitor(pool.get_ref(), project_id, id).await?;
    Ok(HttpResponse::Ok().json(monitor))
}

/// POST /api/projects/{project_id}/uptime-monitors — create a monitor
pub async fn create_monitor(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    body: web::Json<CreateUptimeMonitor>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let project_id = path.into_inner();
    access::require(
        pool.get_ref(),
        actor.is_admin(),
        actor.user_id(),
        project_id,
        Action::UpdateProject,
    )
    .await?;

    let monitor =
        UptimeMonitorService::create(pool.get_ref(), project_id, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(monitor))
}

/// PATCH /api/projects/{project_id}/uptime-monitors/{id}
pub async fn update_monitor(
    pool: web::Data<DbPool>,
    path: web::Path<(i32, Uuid)>,
    body: web::Json<UpdateUptimeMonitor>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let (project_id, id) = path.into_inner();
    access::require(
        pool.get_ref(),
        actor.is_admin(),
        actor.user_id(),
        project_id,
        Action::UpdateProject,
    )
    .await?;

    get_owned_monitor(pool.get_ref(), project_id, id).await?;
    let monitor = UptimeMonitorService::update(pool.get_ref(), id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(monitor))
}

/// DELETE /api/projects/{project_id}/uptime-monitors/{id}
pub async fn delete_monitor(
    pool: web::Data<DbPool>,
    path: web::Path<(i32, Uuid)>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let (project_id, id) = path.into_inner();
    access::require(
        pool.get_ref(),
        actor.is_admin(),
        actor.user_id(),
        project_id,
        Action::UpdateProject,
    )
    .await?;

    get_owned_monitor(pool.get_ref(), project_id, id).await?;
    UptimeMonitorService::delete(pool.get_ref(), id).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/projects/{project_id}/uptime-monitors/{id}/check — manual probe
/// (does NOT affect the state machine)
pub async fn trigger_check(
    pool: web::Data<DbPool>,
    path: web::Path<(i32, Uuid)>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let (project_id, id) = path.into_inner();
    access::require(
        pool.get_ref(),
        actor.is_admin(),
        actor.user_id(),
        project_id,
        Action::ViewProject,
    )
    .await?;

    let monitor = get_owned_monitor(pool.get_ref(), project_id, id).await?;

    let probe = match monitor.check_type.as_str() {
        "http" => {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(monitor.timeout_secs as u64))
                .build()
                .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
            run_http_probe(&client, &monitor).await
        }
        "tcp" => run_tcp_probe(&monitor).await,
        other => {
            return Err(crate::error::AppError::Validation(format!(
                "Unknown check_type: {other}"
            )))
        }
    };

    // Record in uptime_checks (status 2 = manual check)
    let check_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();
    let status: i32 = 2;

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
    .bind(probe.latency_ms as i32)
    .bind(&probe.error)
    .execute(pool.get_ref())
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
    .bind(probe.latency_ms as i32)
    .bind(&probe.error)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "ok": probe.ok,
        "latency_ms": probe.latency_ms,
        "error": probe.error,
    })))
}

/// PUT /api/projects/{project_id}/uptime-monitors/{id}/channels — replace
/// all alert-integration assignments for a monitor
pub async fn put_monitor_channels(
    pool: web::Data<DbPool>,
    path: web::Path<(i32, Uuid)>,
    body: web::Json<AssignUptimeMonitorChannels>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let (project_id, id) = path.into_inner();
    access::require(
        pool.get_ref(),
        actor.is_admin(),
        actor.user_id(),
        project_id,
        Action::UpdateProject,
    )
    .await?;

    get_owned_monitor(pool.get_ref(), project_id, id).await?;
    UptimeMonitorService::assign_channels(pool.get_ref(), id, body.into_inner().integration_ids)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

// =============================================================================
// Route configuration
// =============================================================================

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/projects/{project_id}/uptime-monitors")
            .route("", web::get().to(list_monitors))
            .route("", web::post().to(create_monitor))
            .route("/{id}", web::get().to(get_monitor))
            .route("/{id}", web::patch().to(update_monitor))
            .route("/{id}", web::delete().to(delete_monitor))
            .route("/{id}/check", web::post().to(trigger_check))
            .route("/{id}/channels", web::put().to(put_monitor_channels)),
    );
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::models::uptime_monitor::CheckType;
    use crate::services::uptime_monitor::validate_monitor_url;

    #[tokio::test]
    async fn test_rfc1918_url_returns_validation_error() {
        let result = validate_monitor_url("http://10.0.0.1/health", &CheckType::Http).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::AppError::Validation(_)));
        assert!(err.to_string().contains("reserved"));
    }

    #[tokio::test]
    async fn test_interval_too_low_returns_validation_error() {
        use crate::services::uptime_monitor::validate_monitor_bounds;
        let result = validate_monitor_bounds(10, 10, 2, 2);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, crate::error::AppError::Validation(_)));
        assert!(err.to_string().contains("interval_secs"));
    }
}
