//! Patch Application Guard — governs how agent proposals become file writes.
//!
//! v1.7.0: Every agent patch must pass through this module.
//! - Validates path against forbidden patterns (secret, workflow, binary, locks)
//! - Verifies before-content checksum matches current file
//! - Enforces allowed_paths (server-side policy intersected with UI scope)
//! - Writes atomically (temp-then-rename)
//! - Records audit trail (append-only JSONL)
//! - Never auto-commits

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// DTOs — typed, not stringly-typed
// ---------------------------------------------------------------------------

/// Agent type — determines server-side allowed paths policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentType {
    Docs,
    Bugfix,
}

/// A patch proposal with stable ID for audit correlation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposal {
    pub proposal_id: String,
    pub path: String,
    pub before: String,
    pub after: String,
    pub before_checksum: String,
    pub after_checksum: String,
}

/// Durable patch application outcome.
/// "accepted" is a UI state; audit records capture durable outcomes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PatchStatus {
    Proposed,
    Rejected,
    ApplyFailed,
    Applied,
}

/// Result of a patch application attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchApplicationResult {
    pub proposal_id: String,
    pub path: String,
    pub status: PatchStatus,
    pub before_checksum: String,
    pub after_checksum: Option<String>,
    pub verification: String,
    pub reason: Option<String>,
}

/// Audit record — append-only JSONL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub timestamp: String,
    pub agent: AgentType,
    pub proposal_id: String,
    pub path: String,
    pub status: PatchStatus,
    pub before_checksum: String,
    pub after_checksum: Option<String>,
    pub verification: String,
    pub allowed_paths: Vec<String>,
    pub forbidden_check: String,
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Server-side agent path policies
// ---------------------------------------------------------------------------

/// Get the server-side allowed paths for an agent type.
/// The frontend may narrow scope but must not widen it.
fn server_allowed_paths(agent_type: &AgentType) -> Vec<&'static str> {
    match agent_type {
        AgentType::Docs => vec![
            "README.md",
            "docs/",
            "roadmap/",
            "CHANGELOG.md",
        ],
        AgentType::Bugfix => {
            // Bugfix agent allowed paths come from the user-selected file scope.
            // Server-side policy allows source code files but not docs/workflow.
            // The actual allowed set is the intersection of server policy and UI scope.
            vec![] // Empty = use UI scope intersected with non-forbidden paths
        }
    }
}

/// Check if a path is allowed by server-side policy.
fn is_server_allowed(agent_type: &AgentType, path: &str) -> bool {
    let normalized = path.replace("\\", "/");
    let server_paths = server_allowed_paths(agent_type);

    if server_paths.is_empty() {
        // Bugfix agent: allow all non-forbidden paths
        return true;
    }

    server_paths.iter().any(|prefix| {
        normalized.starts_with(prefix) || normalized == prefix.trim_end_matches('/')
    })
}

// ---------------------------------------------------------------------------
// Forbidden path checks
// ---------------------------------------------------------------------------

/// Forbidden dependency lock files.
const FORBIDDEN_LOCK_FILES: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "poetry.lock",
    "uv.lock",
];

/// Check if a path is forbidden for agent writes.
fn is_forbidden_path(path: &str) -> (bool, Option<String>) {
    let normalized = path.replace("\\", "/");
    let filename = normalized.rsplit('/').next().unwrap_or(&normalized);
    let lower = filename.to_lowercase();

    // Secret-risk paths (reuse same patterns as git-bridge risk classification)
    if lower == ".env" || lower.starts_with(".env.") {
        return (true, Some("secret-risk path".into()));
    }
    if lower.ends_with(".pem") || lower.ends_with(".key") || lower.ends_with(".p12")
        || lower.ends_with(".pfx") || lower.ends_with(".p8")
    {
        return (true, Some("secret-risk extension".into()));
    }
    if lower == "id_rsa" || lower == "id_ed25519" || lower == "id_ecdsa" {
        return (true, Some("secret-like filename".into()));
    }
    if lower.starts_with("secrets.") || lower.starts_with("credentials.") {
        return (true, Some("credential file".into()));
    }

    // Workflow-risk paths
    if normalized.contains(".github/workflows/") || normalized.contains(".github/actions/") {
        return (true, Some("workflow-risk path".into()));
    }

    // Binary extensions (non-exhaustive but covers common cases)
    let binary_exts = [
        ".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".ico", ".svg",
        ".exe", ".dll", ".so", ".dylib", ".bin", ".dat",
        ".zip", ".tar", ".gz", ".bz2", ".7z", ".rar",
        ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx",
        ".woff", ".woff2", ".ttf", ".eot", ".otf",
        ".mp3", ".mp4", ".avi", ".mov", ".wav",
    ];
    if binary_exts.iter().any(|ext| lower.ends_with(ext)) {
        return (true, Some("binary file extension".into()));
    }

    // Dependency lock files
    if FORBIDDEN_LOCK_FILES.iter().any(|lock| lower == lock.to_lowercase()) {
        return (true, Some("dependency lock file".into()));
    }

    // CI/CD config
    if normalized.contains("docker-compose") || normalized.contains("Dockerfile") {
        return (true, Some("infrastructure config".into()));
    }

    (false, None)
}

// ---------------------------------------------------------------------------
// Checksum helpers
// ---------------------------------------------------------------------------

fn sha256_hex(data: &str) -> String {
    // Use std::collections::hash_map::DefaultHasher as a stable checksum.
    // Not cryptographic but sufficient for before/after content matching.
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn file_checksum(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(sha256_hex(&content))
}

// ---------------------------------------------------------------------------
// Atomic write
// ---------------------------------------------------------------------------

/// Write file atomically: write to temp file in same directory, then rename.
/// Failed writes leave the original file unchanged.
fn atomic_write(target: &Path, content: &str) -> std::io::Result<()> {
    let parent = target.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "No parent directory")
    })?;

    let mut temp_path = target.to_path_buf();
    let file_name = target.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "No file name")
    })?;
    temp_path.set_file_name(format!(
        ".{}.patch-tmp",
        file_name.to_string_lossy()
    ));

    // Write to temp file
    std::fs::write(&temp_path, content)?;

    // Atomic rename (same filesystem)
    std::fs::rename(&temp_path, target)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Audit trail
// ---------------------------------------------------------------------------

fn append_audit_record(project_root: &Path, record: &AuditRecord) -> Result<(), String> {
    let agent_dir = project_root
        .join(".Orqestra")
        .join("agents")
        .join(match &record.agent {
            AgentType::Docs => "docs",
            AgentType::Bugfix => "bugfix",
        });
    std::fs::create_dir_all(&agent_dir)
        .map_err(|e| format!("Failed to create audit dir: {e}"))?;

    let audit_path = agent_dir.join("audit.jsonl");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&audit_path)
        .map_err(|e| format!("Failed to open audit file: {e}"))?;

    let line = serde_json::to_string(record)
        .map_err(|e| format!("Failed to serialize audit record: {e}"))?;

    use std::io::Write;
    writeln!(file, "{line}")
        .map_err(|e| format!("Failed to write audit record: {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Public commands
// ---------------------------------------------------------------------------

/// Apply an agent patch with full governance.
///
/// Steps:
/// 1. Validate path against forbidden patterns
/// 2. Check path against server-side policy (agent_type)
/// 3. Intersect with UI-provided allowed_paths
/// 4. Verify before-checksum matches current file
/// 5. Write atomically
/// 6. Verify after-checksum matches written file
/// 7. Record audit trail
pub fn apply_agent_patch(
    project_root: &Path,
    patch: &PatchProposal,
    allowed_paths: &[String],
    agent_type: &AgentType,
) -> PatchApplicationResult {
    let normalized = patch.path.replace("\\", "/");
    let full_path = project_root.join(&normalized);

    // 1. Forbidden path check
    let (forbidden, forbidden_reason) = is_forbidden_path(&normalized);
    if forbidden {
        let result = PatchApplicationResult {
            proposal_id: patch.proposal_id.clone(),
            path: patch.path.clone(),
            status: PatchStatus::ApplyFailed,
            before_checksum: patch.before_checksum.clone(),
            after_checksum: None,
            verification: "forbidden".into(),
            reason: forbidden_reason,
        };
        let record = make_audit_record(agent_type, &patch, allowed_paths, &result, "forbidden");
        let _ = append_audit_record(project_root, &record);
        return result;
    }

    // 2. Server-side policy check
    if !is_server_allowed(agent_type, &normalized) {
        let result = PatchApplicationResult {
            proposal_id: patch.proposal_id.clone(),
            path: patch.path.clone(),
            status: PatchStatus::ApplyFailed,
            before_checksum: patch.before_checksum.clone(),
            after_checksum: None,
            verification: "server-policy-blocked".into(),
            reason: Some("Path not allowed by server-side agent policy".into()),
        };
        let record = make_audit_record(agent_type, &patch, allowed_paths, &result, "server-policy-blocked");
        let _ = append_audit_record(project_root, &record);
        return result;
    }

    // 3. UI-provided allowed_paths check (frontend may narrow, not widen)
    if !allowed_paths.is_empty() {
        let in_ui_scope = allowed_paths.iter().any(|p| {
            let p_norm = p.replace("\\", "/");
            normalized.starts_with(&p_norm) || normalized == p_norm.trim_end_matches('/')
        });
        if !in_ui_scope {
            let result = PatchApplicationResult {
                proposal_id: patch.proposal_id.clone(),
                path: patch.path.clone(),
                status: PatchStatus::ApplyFailed,
                before_checksum: patch.before_checksum.clone(),
                after_checksum: None,
                verification: "outside-ui-scope".into(),
                reason: Some("Path outside UI-provided allowed scope".into()),
            };
            let record = make_audit_record(agent_type, &patch, allowed_paths, &result, "outside-ui-scope");
            let _ = append_audit_record(project_root, &record);
            return result;
        }
    }

    // 4. Path traversal check
    let canonical_root = project_root.canonicalize().ok();
    if let Some(root) = &canonical_root {
        if let Ok(canonical_target) = full_path.canonicalize() {
            // For existing files
            if !canonical_target.starts_with(root) {
                let result = PatchApplicationResult {
                    proposal_id: patch.proposal_id.clone(),
                    path: patch.path.clone(),
                    status: PatchStatus::ApplyFailed,
                    before_checksum: patch.before_checksum.clone(),
                    after_checksum: None,
                    verification: "path-traversal".into(),
                    reason: Some("Path traversal blocked".into()),
                };
                let record = make_audit_record(agent_type, &patch, allowed_paths, &result, "path-traversal");
                let _ = append_audit_record(project_root, &record);
                return result;
            }
        } else {
            // New file — check parent
            if let Some(parent) = full_path.parent() {
                if let Ok(canonical_parent) = parent.canonicalize() {
                    if !canonical_parent.starts_with(root) {
                        let result = PatchApplicationResult {
                            proposal_id: patch.proposal_id.clone(),
                            path: patch.path.clone(),
                            status: PatchStatus::ApplyFailed,
                            before_checksum: patch.before_checksum.clone(),
                            after_checksum: None,
                            verification: "path-traversal".into(),
                            reason: Some("Path traversal blocked (new file)".into()),
                        };
                        let record = make_audit_record(agent_type, &patch, allowed_paths, &result, "path-traversal");
                        let _ = append_audit_record(project_root, &record);
                        return result;
                    }
                }
            }
        }
    }

    // 5. Before-checksum verification
    let current_checksum = file_checksum(&full_path).unwrap_or_default();
    if current_checksum != patch.before_checksum {
        let result = PatchApplicationResult {
            proposal_id: patch.proposal_id.clone(),
            path: patch.path.clone(),
            status: PatchStatus::ApplyFailed,
            before_checksum: current_checksum,
            after_checksum: None,
            verification: "before-checksum-mismatch".into(),
            reason: Some("File changed since proposal — stale patch".into()),
        };
        let record = make_audit_record(agent_type, &patch, allowed_paths, &result, "before-checksum-mismatch");
        let _ = append_audit_record(project_root, &record);
        return result;
    }

    // Also verify before content matches (belt and suspenders)
    if let Ok(current_content) = std::fs::read_to_string(&full_path) {
        if current_content != patch.before {
            let result = PatchApplicationResult {
                proposal_id: patch.proposal_id.clone(),
                path: patch.path.clone(),
                status: PatchStatus::ApplyFailed,
                before_checksum: current_checksum,
                after_checksum: None,
                verification: "before-content-mismatch".into(),
                reason: Some("File content changed since proposal".into()),
            };
            let record = make_audit_record(agent_type, &patch, allowed_paths, &result, "before-content-mismatch");
            let _ = append_audit_record(project_root, &record);
            return result;
        }
    }

    // 6. Atomic write
    if let Err(e) = atomic_write(&full_path, &patch.after) {
        let result = PatchApplicationResult {
            proposal_id: patch.proposal_id.clone(),
            path: patch.path.clone(),
            status: PatchStatus::ApplyFailed,
            before_checksum: patch.before_checksum.clone(),
            after_checksum: None,
            verification: "write-failed".into(),
            reason: Some(format!("Atomic write failed: {e}")),
        };
        let record = make_audit_record(agent_type, &patch, allowed_paths, &result, "write-failed");
        let _ = append_audit_record(project_root, &record);
        return result;
    }

    // 7. Post-write verification
    let written_checksum = file_checksum(&full_path).unwrap_or_default();
    let expected_after = sha256_hex(&patch.after);
    if written_checksum != expected_after {
        // Write succeeded but checksum mismatch — should not happen with atomic write
        let result = PatchApplicationResult {
            proposal_id: patch.proposal_id.clone(),
            path: patch.path.clone(),
            status: PatchStatus::ApplyFailed,
            before_checksum: patch.before_checksum.clone(),
            after_checksum: Some(written_checksum),
            verification: "after-checksum-mismatch".into(),
            reason: Some("Post-write verification failed".into()),
        };
        let record = make_audit_record(agent_type, &patch, allowed_paths, &result, "after-checksum-mismatch");
        let _ = append_audit_record(project_root, &record);
        return result;
    }

    // Success
    let result = PatchApplicationResult {
        proposal_id: patch.proposal_id.clone(),
        path: patch.path.clone(),
        status: PatchStatus::Applied,
        before_checksum: patch.before_checksum.clone(),
        after_checksum: Some(expected_after),
        verification: "match".into(),
        reason: None,
    };
    let record = make_audit_record(agent_type, &patch, allowed_paths, &result, "pass");
    let _ = append_audit_record(project_root, &record);
    result
}

/// Record a rejection without modifying any file.
pub fn reject_agent_patch(
    project_root: &Path,
    patch: &PatchProposal,
    agent_type: &AgentType,
    reason: &str,
) -> PatchApplicationResult {
    let result = PatchApplicationResult {
        proposal_id: patch.proposal_id.clone(),
        path: patch.path.clone(),
        status: PatchStatus::Rejected,
        before_checksum: patch.before_checksum.clone(),
        after_checksum: None,
        verification: "rejected".into(),
        reason: Some(reason.to_string()),
    };
    let record = make_audit_record(agent_type, &patch, &[], &result, reason);
    let _ = append_audit_record(project_root, &record);
    result
}

fn make_audit_record(
    agent_type: &AgentType,
    patch: &PatchProposal,
    allowed_paths: &[String],
    result: &PatchApplicationResult,
    forbidden_check: &str,
) -> AuditRecord {
    AuditRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        agent: agent_type.clone(),
        proposal_id: patch.proposal_id.clone(),
        path: patch.path.clone(),
        status: result.status.clone(),
        before_checksum: result.before_checksum.clone(),
        after_checksum: result.after_checksum.clone(),
        verification: result.verification.clone(),
        allowed_paths: allowed_paths.to_vec(),
        forbidden_check: forbidden_check.to_string(),
        reason: result.reason.clone(),
    }
}
