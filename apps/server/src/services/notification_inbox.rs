use uuid::Uuid;

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::Notification;

/// The in-app notification inbox (distinct from `services::notification`,
/// which handles *outbound* alert delivery to Slack/email/webhook — this is
/// the in-app equivalent, independent of whether any outbound channel is
/// configured for a project).
pub struct NotificationInboxService;

impl NotificationInboxService {
    /// Creates one notification per project member. Called from
    /// [`crate::services::alert::AlertService::trigger_alert`] alongside
    /// outbound dispatch, so the inbox fills in even for projects with no
    /// Slack/email/webhook integration configured.
    pub async fn create_for_project(
        pool: &DbPool,
        project_id: i32,
        issue_id: Uuid,
        kind: &str,
        title: &str,
        body: &str,
        url: &str,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, project_id, issue_id, kind, title, body, url)
            SELECT user_id, $1, $2, $3, $4, $5, $6
            FROM project_members WHERE project_id = $1
            "#,
        )
        .bind(project_id)
        .bind(issue_id)
        .bind(kind)
        .bind(title)
        .bind(body)
        .bind(url)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Paginated inbox for one user, newest first.
    pub async fn list_for_user(
        pool: &DbPool,
        user_id: i32,
        page: i64,
        per_page: i64,
    ) -> AppResult<(Vec<Notification>, i64)> {
        let offset = (page - 1).max(0) * per_page;

        let items: Vec<Notification> = sqlx::query_as(
            r#"
            SELECT id, user_id, project_id, issue_id, kind, title, body, url, read_at, created_at
            FROM notifications WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let total: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM notifications WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok((items, total))
    }

    pub async fn unread_count(pool: &DbPool, user_id: i32) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM notifications WHERE user_id = $1 AND read_at IS NULL",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Marks one notification read. Scoped to `user_id` so a user can only
    /// mark their own notifications — a mismatched id is a silent no-op
    /// (0 rows affected), same shape as other owner-scoped mutations in
    /// this codebase.
    pub async fn mark_read(pool: &DbPool, user_id: i32, id: i32) -> AppResult<()> {
        sqlx::query(
            "UPDATE notifications SET read_at = CURRENT_TIMESTAMP WHERE id = $1 AND user_id = $2 AND read_at IS NULL",
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn mark_all_read(pool: &DbPool, user_id: i32) -> AppResult<u64> {
        let res = sqlx::query(
            "UPDATE notifications SET read_at = CURRENT_TIMESTAMP WHERE user_id = $1 AND read_at IS NULL",
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(res.rows_affected())
    }
}
