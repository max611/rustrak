use std::time::Duration;

use chrono::Utc;

use crate::db::DbPool;
use crate::services::issue::IssueService;

/// Background worker that flips expired issue snoozes back to `unresolved`.
///
/// Each tick runs [`IssueService::sweep_expired_snoozes`], mirroring
/// [`crate::workers::monitor_worker::MonitorWorker`]'s shape.
pub struct SnoozeWorker {
    pool: DbPool,
    tick_interval_secs: u64,
}

impl SnoozeWorker {
    pub fn new(pool: DbPool, tick_interval_secs: u64) -> Self {
        Self {
            pool,
            tick_interval_secs,
        }
    }

    pub async fn run(self) {
        log::info!(
            "Snooze worker starting up (tick every {}s)",
            self.tick_interval_secs
        );
        let mut interval = tokio::time::interval(Duration::from_secs(self.tick_interval_secs));
        loop {
            interval.tick().await;
            match IssueService::sweep_expired_snoozes(&self.pool, Utc::now()).await {
                Ok(n) if n > 0 => log::info!("Snooze worker: {} issue(s) unsnoozed", n),
                Ok(_) => {}
                Err(e) => log::error!("Snooze worker sweep error: {:?}", e),
            }
        }
    }
}
