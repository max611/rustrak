use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::auth::ApiActor;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::services::notification_inbox::NotificationInboxService;

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    #[serde(default = "default_page")]
    page: i64,
    #[serde(default = "default_per_page")]
    per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

fn require_user(actor: &ApiActor) -> AppResult<i32> {
    actor
        .user_id()
        .ok_or_else(|| AppError::Unauthorized("Notifications require a user session".into()))
}

/// GET /api/notifications — the current user's inbox, newest first.
pub async fn list_notifications(
    pool: web::Data<DbPool>,
    query: web::Query<ListNotificationsQuery>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let user_id = require_user(&actor)?;
    let (items, total) =
        NotificationInboxService::list_for_user(pool.get_ref(), user_id, query.page, query.per_page)
            .await?;
    let unread_count = NotificationInboxService::unread_count(pool.get_ref(), user_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "items": items,
        "total": total,
        "unread_count": unread_count,
    })))
}

/// PATCH /api/notifications/{id}/read
pub async fn mark_read(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let user_id = require_user(&actor)?;
    NotificationInboxService::mark_read(pool.get_ref(), user_id, path.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

/// POST /api/notifications/mark-all-read
pub async fn mark_all_read(pool: web::Data<DbPool>, actor: ApiActor) -> AppResult<HttpResponse> {
    let user_id = require_user(&actor)?;
    let n = NotificationInboxService::mark_all_read(pool.get_ref(), user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "marked_read": n })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/notifications")
            .route("", web::get().to(list_notifications))
            .route("/mark-all-read", web::post().to(mark_all_read))
            .route("/{id}/read", web::patch().to(mark_read)),
    );
}
