use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// A row in a user's in-app notification inbox. Created by
/// [`crate::services::alert::AlertService::trigger_alert`] for every project
/// member alongside outbound alert dispatch (Slack/email/webhook) — this is
/// the in-app equivalent, independent of whether any outbound channel is
/// configured for the project.
#[derive(Debug, Clone, FromRow, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Notification {
    pub id: i32,
    pub user_id: i32,
    pub project_id: Option<i32>,
    pub issue_id: Option<Uuid>,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub url: Option<String>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
