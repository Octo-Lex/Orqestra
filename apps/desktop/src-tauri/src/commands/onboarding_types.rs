//! Persistent app state DTOs (v2.5.3).
//!
//! Onboarding state, project records, and recent projects.
//! Stored in `{app_data_dir}/app-state.json`.
//!
//! Boundaries:
//!   app-state.json: may store local paths (private local state)
//!   diagnostic bundles: must hash/redact all paths
//!
//! No secrets, tokens, PATs, or CRDT data in this file.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// AppState — persisted to disk
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub onboarding_completed: bool,
    pub last_project_root: Option<String>,
    pub last_project_id: Option<String>,
    pub recent_projects: Vec<ProjectRecord>,
    pub last_opened_at: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            onboarding_completed: false,
            last_project_root: None,
            last_project_id: None,
            recent_projects: Vec::new(),
            last_opened_at: None,
        }
    }
}

// ---------------------------------------------------------------------------
// ProjectRecord
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    /// Stable identity: sha256(canonical_root_path)
    pub project_id: String,
    /// Local filesystem path (not exported in diagnostics)
    pub root: String,
    /// Display name (directory name or user-provided)
    pub name: String,
    /// ISO-8601 timestamp of last open
    pub last_opened_at: String,
    /// Last known credential status (metadata only, recomputed on open)
    pub last_known_credential_status: CredentialStatus,
    /// Last known relay status (metadata only, recomputed on open)
    pub last_known_relay_status: RelayConnectionStatus,
}

// ---------------------------------------------------------------------------
// Status enums (metadata only — no secret material)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStatus {
    Unknown,
    Configured,
    Missing,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RelayConnectionStatus {
    Unknown,
    Connected,
    Disconnected,
    NeverConnected,
}

// ---------------------------------------------------------------------------
// Project validation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectValidationResult {
    pub valid: bool,
    pub has_git: bool,
    pub has_roadmap: bool,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Project switch result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSwitchResult {
    pub success: bool,
    pub project_id: String,
    pub name: String,
    pub credential_status: CredentialStatus,
    pub relay_status: RelayConnectionStatus,
    pub previous_relay_disconnected: bool,
}

// ---------------------------------------------------------------------------
// Reset parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetOnboardingRequest {
    /// Clear onboarding completion flag
    pub clear_metadata: bool,
    /// Clear recent projects and last project
    pub clear_project_history: bool,
    // NOTE: OS-keychain credentials are NEVER cleared here.
    // Use a separate credential command for secret deletion.
}

// ---------------------------------------------------------------------------
// Project ID generation
// ---------------------------------------------------------------------------

/// Generate a stable project ID from a canonical path.
/// Uses SHA-256 hash of the canonical root path.
pub fn project_id_from_root(root: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(root.as_bytes());
    format!("proj-{:x}", hasher.finalize())
}

/// Max recent projects to track.
pub const MAX_RECENT_PROJECTS: usize = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_id_deterministic() {
        let id1 = project_id_from_root("C:/Projects/Orqestra");
        let id2 = project_id_from_root("C:/Projects/Orqestra");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_project_id_different_paths() {
        let id1 = project_id_from_root("C:/Projects/A");
        let id2 = project_id_from_root("C:/Projects/B");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_project_id_format() {
        let id = project_id_from_root("/home/user/project");
        assert!(id.starts_with("proj-"));
        assert!(id.len() > 10);
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(!state.onboarding_completed);
        assert!(state.last_project_root.is_none());
        assert!(state.recent_projects.is_empty());
    }

    #[test]
    fn test_credential_status_serialization() {
        let status = CredentialStatus::Configured;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"configured\"");
    }

    #[test]
    fn test_relay_status_serialization() {
        let status = RelayConnectionStatus::Connected;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"connected\"");
    }

    #[test]
    fn test_project_record_no_secrets() {
        let record = ProjectRecord {
            project_id: "proj-abc".to_string(),
            root: "/home/user/project".to_string(),
            name: "My Project".to_string(),
            last_opened_at: "2026-06-08T00:00:00Z".to_string(),
            last_known_credential_status: CredentialStatus::Configured,
            last_known_relay_status: RelayConnectionStatus::Connected,
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(!json.contains("ork_"), "Must not contain tokens");
        assert!(!json.contains("Bearer"), "Must not contain auth headers");
        assert!(!json.contains("password"), "Must not contain passwords");
    }

    #[test]
    fn test_reset_request_no_secret_fields() {
        let req = ResetOnboardingRequest {
            clear_metadata: true,
            clear_project_history: false,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("secret"));
        assert!(!json.contains("credential"));
    }

    #[test]
    fn test_max_recent_projects_constant() {
        assert_eq!(MAX_RECENT_PROJECTS, 10);
    }
}
