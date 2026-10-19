use actix_web::{web, HttpResponse};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::ApiActor;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::AssignMonitorChannels;
#[cfg(feature = "openapi")]
use crate::models::CheckInResponse;
use crate::models::MonitorResponse;
use crate::pagination::{ListCheckInsQuery, OffsetPaginatedResponse};
use crate::services::access::{self, Action};
use crate::services::monitor::MonitorService;

/// Resolves a monitor's id from its (project-scoped) slug, 404ing on no match
/// — the slug-keyed equivalent of `routes::uptime_monitors::get_owned_monitor`.
async fn monitor_id_for_slug(pool: &DbPool, project_id: i32, slug: &str) -> AppResult<Uuid> {
    let id: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM monitors WHERE project_id = $1 AND slug = $2")
            .bind(project_id)
            .bind(slug)
            .fetch_optional(pool)
            .await?;

    id.ok_or_else(|| AppError::NotFound(format!("Monitor '{slug}' not found")))
}

#[cfg(feature = "openapi")]
use utoipa::OpenApi;

/// Wrapper for the (unpaginated) monitor list response.
#[derive(Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MonitorsListResponse {
    pub monitors: Vec<MonitorResponse>,
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/projects/{project_id}/monitors",
    tag = "Monitors",
    params(("project_id" = i32, Path, description = "Project ID")),
    responses(
        (status = 200, description = "List of monitors", body = MonitorsListResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Not found", body = crate::error::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
))]
/// GET /api/projects/{project_id}/monitors
/// Lists all monitors (Sentry Crons) for a project, most-recently-active first.
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

    let monitors = MonitorService::list_monitors(pool.get_ref(), project_id).await?;
    Ok(HttpResponse::Ok().json(MonitorsListResponse { monitors }))
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/projects/{project_id}/monitors/{slug}/checkins",
    tag = "Monitors",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("slug" = String, Path, description = "Monitor slug"),
        ListCheckInsQuery,
    ),
    responses(
        (status = 200, description = "Paginated check-in list", body = inline(crate::pagination::OffsetPaginatedResponse<CheckInResponse>)),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Not found", body = crate::error::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
))]
/// GET /api/projects/{project_id}/monitors/{slug}/checkins
/// Lists check-ins for one monitor with offset pagination (newest first).
pub async fn list_check_ins(
    pool: web::Data<DbPool>,
    path: web::Path<(i32, String)>,
    query: web::Query<ListCheckInsQuery>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let (project_id, slug) = path.into_inner();

    access::require(
        pool.get_ref(),
        actor.is_admin(),
        actor.user_id(),
        project_id,
        Action::ViewProject,
    )
    .await?;

    let page = query.page.max(1);
    let per_page = query.per_page.clamp(1, 100);

    let (check_ins, total_count) =
        MonitorService::list_check_ins(pool.get_ref(), project_id, &slug, page, per_page).await?;

    Ok(HttpResponse::Ok().json(OffsetPaginatedResponse::new(
        check_ins,
        total_count,
        page,
        per_page,
    )))
}

#[cfg_attr(feature = "openapi", utoipa::path(
    put,
    path = "/api/projects/{project_id}/monitors/{slug}/channels",
    tag = "Monitors",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("slug" = String, Path, description = "Monitor slug"),
    ),
    request_body = AssignMonitorChannels,
    responses(
        (status = 204, description = "Channels assigned"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Not found", body = crate::error::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
))]
/// PUT /api/projects/{project_id}/monitors/{slug}/channels — replace all
/// alert-integration assignments for a monitor (fires on missed/timed-out
/// check-ins).
pub async fn put_monitor_channels(
    pool: web::Data<DbPool>,
    path: web::Path<(i32, String)>,
    body: web::Json<AssignMonitorChannels>,
    actor: ApiActor,
) -> AppResult<HttpResponse> {
    let (project_id, slug) = path.into_inner();
    access::require(
        pool.get_ref(),
        actor.is_admin(),
        actor.user_id(),
        project_id,
        Action::UpdateProject,
    )
    .await?;

    let monitor_id = monitor_id_for_slug(pool.get_ref(), project_id, &slug).await?;
    MonitorService::assign_channels(pool.get_ref(), monitor_id, body.into_inner().integration_ids)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[cfg(feature = "openapi")]
#[derive(OpenApi)]
#[openapi(paths(list_monitors, list_check_ins, put_monitor_channels))]
pub struct MonitorsApi;

/// Configure monitor routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/projects/{project_id}/monitors")
            .route("", web::get().to(list_monitors))
            .route("/{slug}/checkins", web::get().to(list_check_ins))
            .route("/{slug}/channels", web::put().to(put_monitor_channels)),
    );
}
