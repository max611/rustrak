use crate::auth::generate_token;
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateProjectApiKey, ProjectApiKey};

pub struct ProjectApiKeyService;

impl ProjectApiKeyService {
    /// Lists keys for a project, most recent first.
    pub async fn list(pool: &DbPool, project_id: i32) -> AppResult<Vec<ProjectApiKey>> {
        let keys = sqlx::query_as::<_, ProjectApiKey>(
            r#"
            SELECT id, project_id, key, description, created_at, last_used_at
            FROM project_api_keys
            WHERE project_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        Ok(keys)
    }

    /// Gets a key by ID (unscoped — callers must verify `project_id` themselves,
    /// matching the ownership-check convention `routes::uptime_monitors` uses).
    pub async fn get_by_id(pool: &DbPool, id: i32) -> AppResult<ProjectApiKey> {
        let key = sqlx::query_as::<_, ProjectApiKey>(
            r#"
            SELECT id, project_id, key, description, created_at, last_used_at
            FROM project_api_keys
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("API key with id {} not found", id)))?;

        Ok(key)
    }

    /// Looks up a key by its secret value (for authentication).
    pub async fn get_by_key(pool: &DbPool, key: &str) -> AppResult<Option<ProjectApiKey>> {
        let result = sqlx::query_as::<_, ProjectApiKey>(
            r#"
            SELECT id, project_id, key, description, created_at, last_used_at
            FROM project_api_keys
            WHERE key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// Creates a new key for a project.
    pub async fn create(
        pool: &DbPool,
        project_id: i32,
        input: CreateProjectApiKey,
    ) -> AppResult<ProjectApiKey> {
        let key_str = format!("rpk_{}", generate_token());

        let key = sqlx::query_as::<_, ProjectApiKey>(
            r#"
            INSERT INTO project_api_keys (project_id, key, description)
            VALUES ($1, $2, $3)
            RETURNING id, project_id, key, description, created_at, last_used_at
            "#,
        )
        .bind(project_id)
        .bind(&key_str)
        .bind(&input.description)
        .fetch_one(pool)
        .await?;

        Ok(key)
    }

    /// Deletes a key (revoke).
    pub async fn delete(pool: &DbPool, id: i32) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM project_api_keys WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "API key with id {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Updates last_used_at timestamp.
    pub async fn update_last_used(pool: &DbPool, id: i32) -> AppResult<()> {
        sqlx::query("UPDATE project_api_keys SET last_used_at = CURRENT_TIMESTAMP WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
