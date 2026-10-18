//! `/api/v1/*` — a project-scoped surface authenticated by a single
//! `X-Api-Key` header (see [`crate::auth::ProjectKeyAuth`]), mirroring
//! BugWatch's `/api/v1/*` namespace where one per-project `api_key` reads
//! and writes that project's issues/events/comments/monitors (feature
//! parity item). Reuses the exact same service calls as the equivalent
//! user-authenticated `/api/projects/{project_id}/...` routes — only the
//! auth/scoping layer differs, so behavior stays identical.

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::auth::ProjectKeyAuth;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::uptime_monitor::{CreateUptimeMonitor, UpdateUptimeMonitor};
use crate::models::UpdateIssueState;
use crate::pagination::{EventCursor, ListEventsQuery, ListIssuesQuery, OffsetPaginatedResponse, PaginatedResponse, PAGE_SIZE};
use crate::routes::issues::CommentRequest;
use crate::services::uptime_monitor::UptimeMonitorService;
use crate::services::{EventService, IssueService, IssueSocialService, ProjectService};

/// 404s unless `issue_id` belongs to the key's project.
async fn require_issue_in_project(pool: &DbPool, project_id: i32, issue_id: Uuid) -> AppResult<crate::models::Issue> {
    let issue = IssueService::get_by_id(pool, issue_id).await?;
    if issue.project_id != project_id {
        return Err(AppError::NotFound(format!("Issue {issue_id} not found")));
    }
    Ok(issue)
}

/// 404s unless the uptime monitor belongs to the key's project.
async fn require_monitor_in_project(
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
// Issues
// =============================================================================

/// GET /api/v1/issues
pub async fn list_issues(
    pool: web::Data<DbPool>,
    query: web::Query<ListIssuesQuery>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let project = ProjectService::get_by_id(pool.get_ref(), auth.project_id).await?;

    let (issues, total_count) = IssueService::list_offset(
        pool.get_ref(),
        auth.project_id,
        query.sort,
        query.order,
        query.filter,
        query.page,
        query.per_page,
        query.q.as_deref(),
    )
    .await?;

    let responses: Vec<_> = issues.iter().map(|i| i.to_response(&project.slug)).collect();

    Ok(HttpResponse::Ok().json(OffsetPaginatedResponse::new(
        responses,
        total_count,
        query.page,
        query.per_page,
    )))
}

/// GET /api/v1/issues/{issue_id}
pub async fn get_issue(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let issue_id = path.into_inner();
    let issue = require_issue_in_project(pool.get_ref(), auth.project_id, issue_id).await?;
    let project = ProjectService::get_by_id(pool.get_ref(), auth.project_id).await?;

    Ok(HttpResponse::Ok().json(issue.to_response(&project.slug)))
}

/// PATCH /api/v1/issues/{issue_id} — resolve/mute/assign/reprioritize.
pub async fn update_issue(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateIssueState>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let issue_id = path.into_inner();
    let issue = require_issue_in_project(pool.get_ref(), auth.project_id, issue_id).await?;
    let project = ProjectService::get_by_id(pool.get_ref(), auth.project_id).await?;

    let mut updated = issue;
    if body.status.as_deref() == Some("resolvedInNextRelease") {
        updated = IssueService::resolve_in_next_release(pool.get_ref(), issue_id).await?;
        IssueSocialService::record_status_change(
            pool.get_ref(),
            issue_id,
            None,
            "resolvedInNextRelease",
        )
        .await?;
    } else if body.status.as_deref() == Some("ignored") && body.ignore_until.is_some() {
        updated =
            IssueService::snooze_until(pool.get_ref(), issue_id, body.ignore_until.unwrap())
                .await?;
        IssueSocialService::record_status_change(pool.get_ref(), issue_id, None, "ignored")
            .await?;
    } else if let Some(status) = body.resolved_status(&updated.status)? {
        let substatus = body.validated_substatus()?;
        updated = IssueService::set_status(pool.get_ref(), issue_id, status, substatus).await?;
        IssueSocialService::record_status_change(pool.get_ref(), issue_id, None, status).await?;
    }
    if let Some(priority) = body.priority.as_deref() {
        updated = IssueService::set_priority(pool.get_ref(), issue_id, priority).await?;
    }
    if body.assigned_to.is_some() || body.assignee_type.is_some() {
        updated = IssueService::assign(
            pool.get_ref(),
            issue_id,
            body.assigned_to.flatten(),
            body.assignee_type.as_ref().and_then(|t| t.as_deref()),
        )
        .await?;
    }

    Ok(HttpResponse::Ok().json(updated.to_response(&project.slug)))
}

// =============================================================================
// Events
// =============================================================================

/// GET /api/v1/issues/{issue_id}/events
pub async fn list_events(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    query: web::Query<ListEventsQuery>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let issue_id = path.into_inner();
    require_issue_in_project(pool.get_ref(), auth.project_id, issue_id).await?;

    let cursor = query
        .cursor
        .as_ref()
        .map(|c| EventCursor::decode(c))
        .transpose()?;

    let (events, has_more) = EventService::list_paginated(
        pool.get_ref(),
        issue_id,
        query.order,
        cursor.as_ref(),
        PAGE_SIZE,
    )
    .await?;

    let responses: Vec<_> = events.iter().map(|e| e.to_response()).collect();

    let next_cursor = if has_more {
        events
            .last()
            .map(|last| EventCursor::new(query.order.as_str(), last.timestamp, last.id).encode())
            .transpose()?
    } else {
        None
    };

    Ok(HttpResponse::Ok().json(PaginatedResponse::new(responses, next_cursor, has_more)))
}

/// GET /api/v1/issues/{issue_id}/events/{event_id}
pub async fn get_event(
    pool: web::Data<DbPool>,
    path: web::Path<(Uuid, Uuid)>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let (issue_id, event_id) = path.into_inner();
    require_issue_in_project(pool.get_ref(), auth.project_id, issue_id).await?;

    let event = EventService::get_by_id(pool.get_ref(), event_id).await?;
    if event.issue_id != Some(issue_id) {
        return Err(AppError::NotFound(format!("Event {event_id} not found")));
    }

    Ok(HttpResponse::Ok().json(event.to_detail_response()))
}

// =============================================================================
// Comments (issue activity feed)
// =============================================================================

/// GET /api/v1/issues/{issue_id}/comments
pub async fn list_issue_comments(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let issue_id = path.into_inner();
    require_issue_in_project(pool.get_ref(), auth.project_id, issue_id).await?;

    let activity = IssueSocialService::list_activity(pool.get_ref(), issue_id).await?;
    Ok(HttpResponse::Ok().json(activity))
}

/// POST /api/v1/issues/{issue_id}/comments
pub async fn create_issue_comment(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    body: web::Json<CommentRequest>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let issue_id = path.into_inner();
    require_issue_in_project(pool.get_ref(), auth.project_id, issue_id).await?;

    let entry = IssueSocialService::add_comment(pool.get_ref(), issue_id, None, &body.text).await?;
    Ok(HttpResponse::Created().json(entry))
}

// =============================================================================
// Uptime monitors
// =============================================================================

/// GET /api/v1/uptime-monitors
pub async fn list_uptime_monitors(
    pool: web::Data<DbPool>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let monitors = UptimeMonitorService::list(pool.get_ref(), auth.project_id).await?;
    Ok(HttpResponse::Ok().json(monitors))
}

/// GET /api/v1/uptime-monitors/{id}
pub async fn get_uptime_monitor(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let monitor = require_monitor_in_project(pool.get_ref(), auth.project_id, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(monitor))
}

/// POST /api/v1/uptime-monitors
pub async fn create_uptime_monitor(
    pool: web::Data<DbPool>,
    body: web::Json<CreateUptimeMonitor>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let monitor =
        UptimeMonitorService::create(pool.get_ref(), auth.project_id, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(monitor))
}

/// PATCH /api/v1/uptime-monitors/{id}
pub async fn update_uptime_monitor(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateUptimeMonitor>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    require_monitor_in_project(pool.get_ref(), auth.project_id, id).await?;
    let monitor = UptimeMonitorService::update(pool.get_ref(), id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(monitor))
}

/// DELETE /api/v1/uptime-monitors/{id}
pub async fn delete_uptime_monitor(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    auth: ProjectKeyAuth,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    require_monitor_in_project(pool.get_ref(), auth.project_id, id).await?;
    UptimeMonitorService::delete(pool.get_ref(), id).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Configure the /api/v1/* routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/issues", web::get().to(list_issues))
            .route("/issues/{issue_id}", web::get().to(get_issue))
            .route("/issues/{issue_id}", web::patch().to(update_issue))
            .route("/issues/{issue_id}/events", web::get().to(list_events))
            .route(
                "/issues/{issue_id}/events/{event_id}",
                web::get().to(get_event),
            )
            .route(
                "/issues/{issue_id}/comments",
                web::get().to(list_issue_comments),
            )
            .route(
                "/issues/{issue_id}/comments",
                web::post().to(create_issue_comment),
            )
            .route("/uptime-monitors", web::get().to(list_uptime_monitors))
            .route("/uptime-monitors", web::post().to(create_uptime_monitor))
            .route(
                "/uptime-monitors/{id}",
                web::get().to(get_uptime_monitor),
            )
            .route(
                "/uptime-monitors/{id}",
                web::patch().to(update_uptime_monitor),
            )
            .route(
                "/uptime-monitors/{id}",
                web::delete().to(delete_uptime_monitor),
            ),
    );
}
