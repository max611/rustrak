//! Management routes for project-scoped API keys (feature parity with
//! BugWatch's per-project `api_key`). These routes mint/list/revoke keys;
//! the keys themselves authenticate `routes::api_v1`.

use actix_web::{web, HttpResponse};

use crate::auth::ApiActor;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::CreateProjectApiKey;
#[cfg(feature = "openapi")]
use crate::models::{ProjectApiKeyCreatedResponse, ProjectApiKeyResponse};
use crate::services::access::{self, Action};
use crate::services::ProjectApiKeyService;

#[cfg(feature = "openapi")]
use utoipa::OpenApi;

/// Loads a key and 404s if it doesn't belong to `project_id` — the same
/// ownership-check shape `routes::uptime_monitors::get_owned_monitor` uses.
async fn get_owned_key(
    pool: &DbPool,
    project_id: i32,
    id: i32,
) -> AppResult<crate::models::ProjectApiKey> {
    let key = ProjectApiKeyService::get_by_id(pool, id).await?;
    if key.project_id != project_id {
        return Err(AppError::NotFound(format!("API key {id} not found")));
    }
    Ok(key)
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/projects/{project_id}/api-keys",
    tag = "ProjectApiKeys",
    params(("project_id" = i32, Path, description = "Project ID")),
    responses(
        (status = 200, description = "List of keys (masked)", body = Vec<ProjectApiKeyResponse>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
))]
/// GET /api/projects/{project_id}/api-keys
pub async fn list_keys(
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
        Action::UpdateProject,
    )
    .await?;

    let keys = ProjectApiKeyService::list(pool.get_ref(), project_id).await?;
    let responses: Vec<_> = keys.iter().map(|k| k.to_response()).collect();

    Ok(HttpResponse::Ok().json(responses))
}

#[cfg_attr(feature = "openapi", utoipa::path(
    get,
    path = "/api/projects/{project_id}/api-keys/{id}",
    tag = "ProjectApiKeys",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("id" = i32, Path, description = "Key ID"),
    ),
    responses(
        (status = 200, description = "Key details (full key value)", body = ProjectApiKeyCreatedResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Not found", body = crate::error::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
))]
/// GET /api/projects/{project_id}/api-keys/{id} — reveal the full key value,
/// mirroring routes::tokens::get_token's "GET reveals it again" convention.
pub async fn get_key(
    pool: web::Data<DbPool>,
    path: web::Path<(i32, i32)>,
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

    let key = get_owned_key(pool.get_ref(), project_id, id).await?;
    Ok(HttpResponse::Ok().json(key.to_created_response()))
}

#[cfg_attr(feature = "openapi", utoipa::path(
    post,
    path = "/api/projects/{project_id}/api-keys",
    tag = "ProjectApiKeys",
    params(("project_id" = i32, Path, description = "Project ID")),
    request_body = CreateProjectApiKey,
    responses(
        (status = 201, description = "Key created (full key shown once)", body = ProjectApiKeyCreatedResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
))]
/// POST /api/projects/{project_id}/api-keys
pub async fn create_key(
    pool: web::Data<DbPool>,
    path: web::Path<i32>,
    actor: ApiActor,
    body: web::Json<CreateProjectApiKey>,
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

    let key = ProjectApiKeyService::create(pool.get_ref(), project_id, body.into_inner()).await?;

    // Full key, only time it's visible!
    Ok(HttpResponse::Created().json(key.to_created_response()))
}

#[cfg_attr(feature = "openapi", utoipa::path(
    delete,
    path = "/api/projects/{project_id}/api-keys/{id}",
    tag = "ProjectApiKeys",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("id" = i32, Path, description = "Key ID"),
    ),
    responses(
        (status = 204, description = "Key revoked"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Not found", body = crate::error::ErrorResponse),
    ),
    security(("bearer_auth" = [])),
))]
/// DELETE /api/projects/{project_id}/api-keys/{id}
pub async fn delete_key(
    pool: web::Data<DbPool>,
    path: web::Path<(i32, i32)>,
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

    get_owned_key(pool.get_ref(), project_id, id).await?;
    ProjectApiKeyService::delete(pool.get_ref(), id).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[cfg(feature = "openapi")]
#[derive(OpenApi)]
#[openapi(
    paths(list_keys, create_key, get_key, delete_key),
    components(schemas(
        crate::models::ProjectApiKeyResponse,
        crate::models::ProjectApiKeyCreatedResponse,
        crate::models::CreateProjectApiKey,
    ))
)]
pub struct ProjectApiKeysApi;

/// Configure project API key routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/projects/{project_id}/api-keys")
            .route("", web::get().to(list_keys))
            .route("", web::post().to(create_key))
            .route("/{id}", web::get().to(get_key))
            .route("/{id}", web::delete().to(delete_key)),
    );
}
