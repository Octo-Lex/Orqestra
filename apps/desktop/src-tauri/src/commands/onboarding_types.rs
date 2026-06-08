//! Persistent app state DTOs (v2.5.3) + Autonomy settings (v2.6.0).
//!
//! Onboarding state, project records, recent projects, and autonomy policy.
//! Stored in `{app_data_dir}/app-state.json`.
//!
//! Boundaries:
//!   app-state.json: may store local paths (private local state)
//!   diagnostic bundles: must hash/redact all paths
//!
//! No secrets, tokens, PATs, or CRDT data in this file.
//!
//! Autonomy correction: Rust loads persisted autonomy settings;
//! frontend may request auto-apply but may not define policy.

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
    /// v2.6.0: Autonomy settings (persisted, loaded server-side)
    pub autonomy: AutonomySettings,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            onboarding_completed: false,
            last_project_root: None,
            last_project_id: None,
            recent_projects: Vec::new(),
            last_opened_at: None,
            autonomy: AutonomySettings::default(),
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
// Autonomy Settings (v2.6.0)
//
// Stored in AppState, loaded server-side. Frontend never authoritative.
// ---------------------------------------------------------------------------

/// Docs-safe auto-apply paths. Narrower than server-side agent policy.
/// CHANGELOG.md and roadmap/ excluded.
pub const DOCS_AUTO_APPLY_PATHS: &[&str] = &["docs/", "README.md"];

/// Current autonomy policy version.
pub const AUTONOMY_POLICY_VERSION: u32 = 1;

/// Default max auto-apply attempts per session.
pub const DEFAULT_MAX_AUTO_APPLY_PER_SESSION: usize = 5;

/// Default minimum confidence for docs/** paths.
pub const DEFAULT_MIN_CONFIDENCE_DOCS: f64 = 0.80;

/// Minimum confidence for README.md (stricter — public-facing, release-adjacent).
pub const MIN_CONFIDENCE_README: f64 = 0.90;

/// Default max patch bytes (applies to after.len()).
pub const DEFAULT_MAX_PATCH_BYTES: usize = 32_768;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomySettings {
    /// Autonomy disabled by default. User must explicitly enable.
    pub enabled: bool,
    /// Policy version for audit trail.
    pub policy_version: u32,
    /// Only "docs" agent allowed in pilot.
    pub allowed_agent: String,
    /// Only "auto-apply" operation allowed.
    pub allowed_operation: String,
    /// Always false. Auto-apply never commits.
    pub auto_commit: bool,
    /// Docs-safe path allowlist. Must be exactly ["docs/", "README.md"] or narrower.
    pub docs_safe_paths: Vec<String>,
    /// Max patch size in bytes (computed server-side from after content).
    pub max_patch_bytes: usize,
    /// Min confidence for docs/** paths.
    pub min_confidence_docs: f64,
    /// Min confidence for README.md (>= min_confidence_docs).
    pub min_confidence_readme: f64,
    /// Max auto-apply attempts per session.
    pub max_auto_apply_per_session: usize,
    /// Timestamp when autonomy was enabled (audit).
    pub enabled_at: Option<String>,
    /// Who enabled autonomy (audit).
    pub enabled_by: Option<String>,
}

impl Default for AutonomySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            policy_version: AUTONOMY_POLICY_VERSION,
            allowed_agent: "docs".to_string(),
            allowed_operation: "auto-apply".to_string(),
            auto_commit: false,
            docs_safe_paths: DOCS_AUTO_APPLY_PATHS.iter().map(|s| s.to_string()).collect(),
            max_patch_bytes: DEFAULT_MAX_PATCH_BYTES,
            min_confidence_docs: DEFAULT_MIN_CONFIDENCE_DOCS,
            min_confidence_readme: MIN_CONFIDENCE_README,
            max_auto_apply_per_session: DEFAULT_MAX_AUTO_APPLY_PER_SESSION,
            enabled_at: None,
            enabled_by: None,
        }
    }
}

/// Update request for autonomy settings (frontend may request changes,
/// but Rust validates server-side).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomySettingsUpdate {
    pub enabled: Option<bool>,
}

// ---------------------------------------------------------------------------
// Autonomy decision types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AutoApplyDecision {
    Allowed,
    Rejected(AutoApplyRejectReason),
    RequiresReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AutoApplyRejectReason {
    AutonomyDisabled,
    WrongAgent,
    WrongOperation,
    AutoCommitNotFalse,
    PathForbidden,
    PathNotInAllowlist,
    PathExcluded,
    OperationalRiskBlocked,
    CredentialSecretRisk,
    WorkflowRisk,
    DependencyRisk,
    PatchTooLarge,
    ConfidenceBelowThreshold,
    BeforeChecksumMismatch,
    SessionCapExceeded,
    TraversalAttempt,
    BinaryFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoApplyResult {
    pub proposal_id: String,
    pub decision: AutoApplyDecision,
    pub path_class: String,
    pub applied: bool,
    pub auto_commit: bool,
    pub reason_codes: Vec<String>,
    pub before_checksum: String,
    pub after_checksum: Option<String>,
}

/// Current audit schema version.
pub const AUDIT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoApplyAuditRecord {
    pub audit_schema_version: u32,
    pub timestamp: String,
    pub proposal_id: String,
    pub agent: String,
    pub path_class: String,
    pub policy_decision: String,
    pub reason_codes: Vec<String>,
    pub before_checksum: String,
    pub after_checksum: Option<String>,
    pub applied: bool,
    pub auto_commit: bool,
    pub policy_version: u32,
}

// ---------------------------------------------------------------------------
// Autonomy Observability DTOs (v2.7.0)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutonomyMetrics {
    pub total_decisions: usize,
    pub allowed_count: usize,
    pub rejected_count: usize,
    pub requires_review_count: usize,
    pub rejection_reasons: std::collections::HashMap<String, usize>,
    pub path_classes_allowed: std::collections::HashMap<String, usize>,
    pub path_classes_rejected: std::collections::HashMap<String, usize>,
    pub manual_commits_after_auto_apply: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomySummary {
    pub enabled: bool,
    pub policy_version: u32,
    pub session_metrics: AutonomyMetrics,
    pub audit_metrics: AutonomyMetrics,
    pub audit_record_count: usize,
    pub malformed_audit_lines: usize,
    pub recent_decisions: Vec<AutoApplyAuditRecord>,
    pub safety_report: PilotSafetyReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PilotSafetyReport {
    pub report_timestamp: String,
    pub pilot_duration: Option<String>,
    pub total_auto_applied: usize,
    pub total_rejected: usize,
    pub total_requires_review: usize,
    pub rejection_rate: f64,
    pub top_rejection_reasons: Vec<(String, usize)>,
    pub no_secrets_in_audit: bool,
    pub no_auto_commits: bool,
    pub no_source_files_touched: bool,
    pub audit_completeness: f64,
    pub session_cap_hit_count: usize,
    pub manual_follow_up_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyDiagnosticsSection {
    pub enabled: bool,
    pub policy_version: u32,
    pub audit_record_count: usize,
    pub aggregate_metrics: AutonomyMetrics,
    pub safety_report_summary: PilotSafetyReport,
    // No raw proposal IDs, no recent decisions
    // Proposal IDs hashed in diagnostics
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditExportResult {
    pub records: Vec<AutoApplyAuditRecord>,
    pub malformed_line_count: usize,
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
