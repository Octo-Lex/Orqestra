//! v2.12.0 — External Beta Evidence Intake
//!
//! Consent-gated, local-only, redacted beta evidence export.
//! No automatic upload. No telemetry. No dashboard writes.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;

use crate::diagnostics::redaction::redact_text;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// Consent payload required for beta evidence export.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BetaEvidenceConsent {
    pub explicit: bool,
    pub timestamp: String,
    pub user_acknowledged_redaction: bool,
    pub user_acknowledged_local_only: bool,
}

/// Failure taxonomy entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureEntry {
    pub code: String,
    pub severity: String,
    pub category: String,
    pub user_recoverable: bool,
    pub blocked_steps: Vec<String>,
}

/// Session step tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSteps {
    pub app_launched: bool,
    pub repo_opened: bool,
    pub roadmap_detected: bool,
    pub pm_views_rendered: bool,
    pub readiness_reviewed: bool,
    pub ai_service_available: bool,
    pub agent_flow_completed: bool,
    pub ai_degraded_mode_understood: bool,
    pub dashboard_evidence_viewed: bool,
    pub diagnostics_exported: bool,
}

/// User-provided structured feedback.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BetaFeedback {
    pub feedback_type: String,
    pub role: String,
    pub experience_level: String,
    pub ratings: BetaRatings,
    pub free_text: BetaFreeText,
    pub share_permission: SharePermission,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BetaRatings {
    pub install_clarity: Option<u8>,
    pub onboarding_clarity: Option<u8>,
    pub readiness_clarity: Option<u8>,
    pub pm_views_usefulness: Option<u8>,
    pub ai_degraded_mode_clarity: Option<u8>,
    pub dashboard_evidence_clarity: Option<u8>,
    pub overall_confidence: Option<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BetaFreeText {
    pub what_worked: Option<String>,
    pub what_confused_you: Option<String>,
    pub what_blocked_you: Option<String>,
    pub what_should_change_before_wider_beta: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SharePermission {
    pub allow_aggregate_use: bool,
    pub allow_quote_use: bool,
}

/// Result of exporting beta evidence.
#[derive(Debug, Serialize)]
pub struct BetaEvidenceExportResult {
    pub ok: bool,
    pub path: Option<String>,
    pub files: Vec<String>,
    pub code: Option<String>,
}

// ---------------------------------------------------------------------------
// Failure taxonomy (16 codes)
// ---------------------------------------------------------------------------

/// All valid failure codes.
pub const FAILURE_CODES: &[&str] = &[
    "INSTALL_BLOCKED",
    "SMARTSCREEN_WARNING",
    "APP_LAUNCH_FAILED",
    "REPO_OPEN_FAILED",
    "ROADMAP_NOT_FOUND",
    "GIT_UNAVAILABLE",
    "KEYCHAIN_UNAVAILABLE",
    "AI_SERVICE_UNAVAILABLE",
    "AGENT_FLOW_FAILED",
    "DIFF_REVIEW_FAILED",
    "DASHBOARD_EXPORT_FAILED",
    "EVIDENCE_SCHEMA_INVALID",
    "DIAGNOSTICS_EXPORT_FAILED",
    "USER_ABANDONED",
    "UNKNOWN_FAILURE",
    "CONSENT_DECLINED",
];

/// Severity levels.
pub const SEVERITY_LEVELS: &[&str] = &["info", "warning", "blocking", "critical"];

/// Categories.
pub const FAILURE_CATEGORIES: &[&str] = &[
    "install",
    "app_launch",
    "repo",
    "roadmap",
    "git",
    "credential",
    "ai_service",
    "agent",
    "diff_review",
    "dashboard",
    "evidence",
    "diagnostics",
    "user_action",
    "consent",
    "unknown",
];

// ---------------------------------------------------------------------------
// Session outcome computation
// ---------------------------------------------------------------------------

/// Determine session outcome from steps and failures.
pub fn compute_session_outcome(
    steps: &SessionSteps,
    failures: &[FailureEntry],
) -> &'static str {
    // If any critical/blocking failure, session is blocked
    let has_blocking = failures.iter().any(|f| f.severity == "blocking" || f.severity == "critical");
    if has_blocking {
        return "blocked";
    }

    // If steps are largely complete but with warnings
    let completed_count = [
        steps.app_launched,
        steps.repo_opened,
        steps.pm_views_rendered,
        steps.readiness_reviewed,
    ].iter().filter(|&&b| b).count();

    if completed_count == 4 && !failures.is_empty() {
        return "completed_with_warnings";
    }

    if completed_count == 4 && failures.is_empty() {
        return "completed";
    }

    // If app launched but few steps completed
    if steps.app_launched && completed_count < 2 {
        return "abandoned";
    }

    "unknown"
}

// ---------------------------------------------------------------------------
// Redaction helpers
// ---------------------------------------------------------------------------

/// Redact a string for beta evidence output.
fn redact_for_evidence(text: &str) -> String {
    redact_text(text).redacted_text
}

/// Hash a path with SHA-256.
fn hash_path(path: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}

// ---------------------------------------------------------------------------
// Export command
// ---------------------------------------------------------------------------

/// Export beta evidence bundle (consent-gated, local-only, redacted).
#[command]
pub fn export_beta_evidence_cmd(
    project_root: Option<String>,
    consent: Option<BetaEvidenceConsent>,
    steps: Option<serde_json::Value>,
    feedback: Option<BetaFeedback>,
    failures: Option<serde_json::Value>,
) -> Result<BetaEvidenceExportResult, String> {
    // Gate: consent required
    let consent = match consent {
        Some(c) if c.explicit && c.user_acknowledged_redaction && c.user_acknowledged_local_only => c,
        _ => {
            return Ok(BetaEvidenceExportResult {
                ok: false,
                path: None,
                files: vec![],
                code: Some("BETA_EVIDENCE_CONSENT_REQUIRED".to_string()),
            });
        }
    };

    let steps: SessionSteps = steps
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or(SessionSteps {
            app_launched: true,
            repo_opened: false,
            roadmap_detected: false,
            pm_views_rendered: false,
            readiness_reviewed: false,
            ai_service_available: false,
            agent_flow_completed: false,
            ai_degraded_mode_understood: false,
            dashboard_evidence_viewed: false,
            diagnostics_exported: false,
        });

    let failures: Vec<FailureEntry> = failures
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let mut failures = failures;
    let outcome = compute_session_outcome(&steps, &failures);

    // v2.12.0 fix: inject default failure for incomplete sessions
    if (outcome == "blocked" || outcome == "abandoned" || outcome == "unknown") && failures.is_empty() {
        failures.push(FailureEntry {
            code: if outcome == "abandoned" { "USER_ABANDONED".to_string() } else { "UNKNOWN_FAILURE".to_string() },
            severity: if outcome == "abandoned" { "info".to_string() } else { "warning".to_string() },
            category: if outcome == "abandoned" { "user_action".to_string() } else { "unknown".to_string() },
            user_recoverable: outcome == "abandoned",
            blocked_steps: vec![],
        });
    }

    // Determine output directory
    let output_dir = match &project_root {
        Some(root) => {
            let root_path = PathBuf::from(root);
            root_path.join(".Orqestra")
        }
        None => {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(".Orqestra")
        }
    };

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S-%3f");
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let bundle_name = format!("beta-evidence-{}-{}", timestamp, unique);
    let bundle_path = output_dir.join(&bundle_name);

    std::fs::create_dir_all(&bundle_path)
        .map_err(|e| format!("Failed to create beta evidence dir: {}", e))?;

    // --- Build and write each file ---

    // 1. beta-session-outcome.json
    let (repo_detected, path_hash, branch, dirty, remote_configured) = match &project_root {
        Some(root) => {
            let root_path = PathBuf::from(root);
            let detected = root_path.join(".git").exists();
            let hash = hash_path(root);
            let br = if detected {
                std::process::Command::new("git")
                    .args(["rev-parse", "--abbrev-ref", "HEAD"])
                    .current_dir(&root_path)
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                "n/a".to_string()
            };
            let d = if detected {
                std::process::Command::new("git")
                    .args(["status", "--porcelain"])
                    .current_dir(&root_path)
                    .output()
                    .ok()
                    .map(|o| !o.stdout.is_empty())
                    .unwrap_or(false)
            } else {
                false
            };
            let remote = if detected {
                std::process::Command::new("git")
                    .args(["remote"])
                    .current_dir(&root_path)
                    .output()
                    .ok()
                    .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
                    .unwrap_or(false)
            } else {
                false
            };
            (detected, hash, br, d, remote)
        }
        None => (false, "none".to_string(), "n/a".to_string(), false, false),
    };

    let session_id = {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("{}-{}", consent.timestamp, chrono::Utc::now().timestamp()).as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    };

    let session_outcome = serde_json::json!({
        "schema_version": 1,
        "session_id": session_id,
        "session_type": "external_beta",
        "started_at": consent.timestamp,
        "completed_at": chrono::Utc::now().to_rfc3339(),
        "outcome": outcome,
        "steps": steps,
        "blocked_features": failures.iter()
            .filter(|f| f.severity == "blocking" || f.severity == "critical")
            .flat_map(|f| f.blocked_steps.clone())
            .collect::<Vec<_>>(),
        "warnings": failures.iter()
            .filter(|f| f.severity == "warning" || f.severity == "info")
            .map(|f| format!("{}: {}", f.code, f.category))
            .collect::<Vec<_>>(),
        "repo": {
            "detected": repo_detected,
            "path_hash": path_hash,
            "branch": branch,
            "dirty": dirty,
            "remote_configured": remote_configured
        },
        "platform": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH
        }
    });

    // 2. beta-feedback.json
    let feedback_json = match &feedback {
        Some(fb) => {
            // Redact free text fields
            let mut redacted_fb = fb.clone();
            if let Some(ref t) = redacted_fb.free_text.what_worked {
                redacted_fb.free_text.what_worked = Some(redact_for_evidence(t));
            }
            if let Some(ref t) = redacted_fb.free_text.what_confused_you {
                redacted_fb.free_text.what_confused_you = Some(redact_for_evidence(t));
            }
            if let Some(ref t) = redacted_fb.free_text.what_blocked_you {
                redacted_fb.free_text.what_blocked_you = Some(redact_for_evidence(t));
            }
            if let Some(ref t) = redacted_fb.free_text.what_should_change_before_wider_beta {
                redacted_fb.free_text.what_should_change_before_wider_beta = Some(redact_for_evidence(t));
            }
            let mut val = serde_json::to_value(&redacted_fb).unwrap_or(serde_json::json!({}));
            // v2.12.0 fix: always include schema_version
            val.as_object_mut().map(|o| o.insert("schema_version".to_string(), serde_json::json!(1)));
            val
        }
        None => serde_json::json!({
            "schema_version": 1,
            "feedback_type": "external_beta",
            "note": "no feedback provided"
        }),
    };

    // 3. beta-failure-taxonomy.json
    let taxonomy_entries: Vec<serde_json::Value> = FAILURE_CODES.iter().map(|code| {
        let (severity, category, recoverable) = match *code {
            "INSTALL_BLOCKED" => ("blocking", "install", false),
            "SMARTSCREEN_WARNING" => ("warning", "install", true),
            "APP_LAUNCH_FAILED" => ("critical", "app_launch", false),
            "REPO_OPEN_FAILED" => ("blocking", "repo", false),
            "ROADMAP_NOT_FOUND" => ("warning", "roadmap", true),
            "GIT_UNAVAILABLE" => ("warning", "git", true),
            "KEYCHAIN_UNAVAILABLE" => ("warning", "credential", true),
            "AI_SERVICE_UNAVAILABLE" => ("warning", "ai_service", true),
            "AGENT_FLOW_FAILED" => ("warning", "agent", true),
            "DIFF_REVIEW_FAILED" => ("warning", "diff_review", true),
            "DASHBOARD_EXPORT_FAILED" => ("warning", "dashboard", true),
            "EVIDENCE_SCHEMA_INVALID" => ("warning", "evidence", true),
            "DIAGNOSTICS_EXPORT_FAILED" => ("warning", "diagnostics", true),
            "USER_ABANDONED" => ("info", "user_action", true),
            "UNKNOWN_FAILURE" => ("warning", "unknown", false),
            "CONSENT_DECLINED" => ("info", "consent", true),
            _ => ("warning", "unknown", false),
        };
        serde_json::json!({
            "code": code,
            "severity": severity,
            "category": category,
            "user_recoverable": recoverable
        })
    }).collect();

    let failure_taxonomy = serde_json::json!({
        "schema_version": 1,
        "taxonomy": taxonomy_entries,
        "observed_failures": failures
    });

    // 4. beta-evidence-manifest.json
    let file_list = vec![
        "beta-session-outcome.json".to_string(),
        "beta-feedback.json".to_string(),
        "beta-failure-taxonomy.json".to_string(),
    ];

    let manifest = serde_json::json!({
        "schema_version": 1,
        "bundle_type": "external-beta-evidence",
        "created_at": chrono::Utc::now().to_rfc3339(),
        "orqestra_version": env!("CARGO_PKG_VERSION"),
        "collection_mode": "local_export",
        "uploaded_automatically": false,
        "consent": {
            "explicit": consent.explicit,
            "timestamp": consent.timestamp
        },
        "redaction": {
            "tokens_removed": true,
            "paths_hashed": true,
            "file_contents_excluded": true,
            "remote_urls_hashed": true
        },
        "files": file_list
    });

    // Write all files with redaction
    let files_to_write: Vec<(&str, String)> = vec![
        ("beta-session-outcome.json", serde_json::to_string_pretty(&session_outcome).unwrap_or_default()),
        ("beta-feedback.json", serde_json::to_string_pretty(&feedback_json).unwrap_or_default()),
        ("beta-failure-taxonomy.json", serde_json::to_string_pretty(&failure_taxonomy).unwrap_or_default()),
        ("beta-evidence-manifest.json", serde_json::to_string_pretty(&manifest).unwrap_or_default()),
    ];

    let mut written_files = Vec::new();
    for (name, content) in &files_to_write {
        let redacted = redact_for_evidence(content);
        let file_path = bundle_path.join(name);
        std::fs::write(&file_path, &redacted)
            .map_err(|e| format!("Failed to write {}: {}", name, e))?;
        written_files.push(name.to_string());
    }

    Ok(BetaEvidenceExportResult {
        ok: true,
        path: Some(bundle_path.display().to_string()),
        files: written_files,
        code: None,
    })
}
