use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// ProjectApiKey model — a secret scoped to exactly one project, authenticating
/// the /api/v1/* surface. Mirrors BugWatch's per-project `api_key`.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProjectApiKey {
    pub id: i32,
    pub project_id: i32,
    pub key: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// DTO for creating a new project API key
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateProjectApiKey {
    #[serde(default)]
    pub description: Option<String>,
}

/// Response that includes the full key (only on creation)
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProjectApiKeyCreatedResponse {
    pub id: i32,
    pub project_id: i32,
    pub key: String, // Only shown once!
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Response for listing (key is masked)
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProjectApiKeyResponse {
    pub id: i32,
    pub project_id: i32,
    pub key_prefix: String, // First 8 chars only
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl ProjectApiKey {
    pub fn to_response(&self) -> ProjectApiKeyResponse {
        ProjectApiKeyResponse {
            id: self.id,
            project_id: self.project_id,
            key_prefix: format!("{}...", &self.key[..8.min(self.key.len())]),
            description: self.description.clone(),
            created_at: self.created_at,
            last_used_at: self.last_used_at,
        }
    }

    pub fn to_created_response(&self) -> ProjectApiKeyCreatedResponse {
        ProjectApiKeyCreatedResponse {
            id: self.id,
            project_id: self.project_id,
            key: self.key.clone(),
            description: self.description.clone(),
            created_at: self.created_at,
        }
    }
}
