//! Monitor models for the uptime monitoring system (active HTTP/TCP probing
//! of external URLs — distinct from `models::check_in`'s Sentry Crons
//! check-in monitoring). Ported from upstream's closed, unmerged PR #81
//! (`feat/uptime-monitoring-backend-core`); renamed throughout to avoid
//! colliding with the Sentry Crons `monitors`/`MonitorService` this fork
//! already shipped, and given project scoping (the original had none —
//! every other resource in this codebase is project-scoped).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// CheckType Enum
// =============================================================================

/// Type of uptime check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum CheckType {
    Http,
    Tcp,
}

impl std::fmt::Display for CheckType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckType::Http => write!(f, "http"),
            CheckType::Tcp => write!(f, "tcp"),
        }
    }
}

impl TryFrom<&str> for CheckType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "http" => Ok(CheckType::Http),
            "tcp" => Ok(CheckType::Tcp),
            other => Err(format!("unknown check_type: {other}")),
        }
    }
}

// =============================================================================
// UptimeMonitorState Enum
// =============================================================================

/// State of a monitor in the state machine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum UptimeMonitorStateEnum {
    Up,
    PendingDown,
    Down,
    PendingUp,
}

impl TryFrom<&str> for UptimeMonitorStateEnum {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "up" => Ok(UptimeMonitorStateEnum::Up),
            "pending_down" => Ok(UptimeMonitorStateEnum::PendingDown),
            "down" => Ok(UptimeMonitorStateEnum::Down),
            "pending_up" => Ok(UptimeMonitorStateEnum::PendingUp),
            other => Err(format!("unknown state: {other}")),
        }
    }
}

impl std::fmt::Display for UptimeMonitorStateEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UptimeMonitorStateEnum::Up => write!(f, "up"),
            UptimeMonitorStateEnum::PendingDown => write!(f, "pending_down"),
            UptimeMonitorStateEnum::Down => write!(f, "down"),
            UptimeMonitorStateEnum::PendingUp => write!(f, "pending_up"),
        }
    }
}

// =============================================================================
// UptimeMonitor Model
// =============================================================================

/// An uptime monitor configuration record
#[derive(Debug, Clone, Serialize, FromRow)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UptimeMonitor {
    pub id: Uuid,
    pub project_id: i32,
    pub name: String,
    pub check_type: String,
    pub url: String,
    pub interval_secs: i32,
    pub timeout_secs: i32,
    pub expected_status: Option<i32>,
    pub fail_threshold: i32,
    pub recovery_threshold: i32,
    pub repeat_interval_secs: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating a new uptime monitor
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateUptimeMonitor {
    pub name: String,
    pub check_type: String,
    pub url: String,
    pub interval_secs: Option<i32>,
    pub timeout_secs: Option<i32>,
    pub expected_status: Option<i32>,
    pub fail_threshold: Option<i32>,
    pub recovery_threshold: Option<i32>,
    pub repeat_interval_secs: Option<i32>,
}

/// DTO for assigning notification channels (alert integrations) to a
/// monitor (replaces existing assignments)
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AssignUptimeMonitorChannels {
    pub integration_ids: Vec<i32>,
}

/// DTO for updating an uptime monitor
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateUptimeMonitor {
    pub name: Option<String>,
    pub url: Option<String>,
    pub interval_secs: Option<i32>,
    pub timeout_secs: Option<i32>,
    pub expected_status: Option<i32>,
    pub fail_threshold: Option<i32>,
    pub recovery_threshold: Option<i32>,
    pub repeat_interval_secs: Option<i32>,
    pub enabled: Option<bool>,
}

// =============================================================================
// UptimeCheck Model
// =============================================================================

/// A single probe result stored in `uptime_checks`
#[derive(Debug, Clone, Serialize, FromRow)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UptimeCheck {
    pub id: Uuid,
    pub monitor_id: Uuid,
    pub checked_at: DateTime<Utc>,
    pub status: i32,
    pub latency_ms: Option<i32>,
    pub error_message: Option<String>,
}

// =============================================================================
// UptimeIncident Model
// =============================================================================

/// An incident representing a period of downtime
#[derive(Debug, Clone, Serialize, FromRow)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UptimeIncident {
    pub id: Uuid,
    pub monitor_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

// =============================================================================
// UptimeMonitorState Model
// =============================================================================

/// State machine state for an uptime monitor
#[derive(Debug, Clone, Serialize, FromRow)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UptimeMonitorState {
    pub monitor_id: Uuid,
    pub state: String,
    pub fail_counter: i32,
    pub recovery_counter: i32,
    pub last_check_at: Option<DateTime<Utc>>,
    pub next_check_at: DateTime<Utc>,
    pub alerted_down_at: Option<DateTime<Utc>>,
    pub last_alerted_at: Option<DateTime<Utc>>,
    pub alert_count: i32,
    pub incident_id: Option<Uuid>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_type_display() {
        assert_eq!(CheckType::Http.to_string(), "http");
        assert_eq!(CheckType::Tcp.to_string(), "tcp");
    }

    #[test]
    fn test_check_type_try_from_valid() {
        assert_eq!(CheckType::try_from("http"), Ok(CheckType::Http));
        assert_eq!(CheckType::try_from("tcp"), Ok(CheckType::Tcp));
    }

    #[test]
    fn test_check_type_try_from_invalid() {
        let result = CheckType::try_from("udp");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown check_type: udp"));
    }

    #[test]
    fn test_monitor_state_enum_display() {
        assert_eq!(UptimeMonitorStateEnum::Up.to_string(), "up");
        assert_eq!(
            UptimeMonitorStateEnum::PendingDown.to_string(),
            "pending_down"
        );
        assert_eq!(UptimeMonitorStateEnum::Down.to_string(), "down");
        assert_eq!(UptimeMonitorStateEnum::PendingUp.to_string(), "pending_up");
    }

    #[test]
    fn test_monitor_state_enum_try_from_valid() {
        assert_eq!(
            UptimeMonitorStateEnum::try_from("up"),
            Ok(UptimeMonitorStateEnum::Up)
        );
        assert_eq!(
            UptimeMonitorStateEnum::try_from("pending_down"),
            Ok(UptimeMonitorStateEnum::PendingDown)
        );
        assert_eq!(
            UptimeMonitorStateEnum::try_from("down"),
            Ok(UptimeMonitorStateEnum::Down)
        );
        assert_eq!(
            UptimeMonitorStateEnum::try_from("pending_up"),
            Ok(UptimeMonitorStateEnum::PendingUp)
        );
    }

    #[test]
    fn test_monitor_state_enum_try_from_invalid() {
        let result = UptimeMonitorStateEnum::try_from("unknown");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown state: unknown"));
    }

    #[test]
    fn test_check_type_equality() {
        assert_eq!(CheckType::Http, CheckType::Http);
        assert_ne!(CheckType::Http, CheckType::Tcp);
    }

    #[test]
    fn test_monitor_state_enum_equality() {
        assert_eq!(UptimeMonitorStateEnum::Up, UptimeMonitorStateEnum::Up);
        assert_ne!(UptimeMonitorStateEnum::Up, UptimeMonitorStateEnum::Down);
    }
}
