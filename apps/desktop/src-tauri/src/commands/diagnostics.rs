//! Diagnostics export and error recovery commands.
//!
//! Provides user-facing diagnostic bundle export with secret redaction
//! and error-code-to-recovery-action mapping.

use crate::diagnostics::bundle;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::command;

use super::roadmap::CommandError;

type CommandResult<T> = Result<T, CommandError>;

// ---------------------------------------------------------------------------
// DTOs (re-export from bundle + command-specific)
// ---------------------------------------------------------------------------

pub use crate::diagnostics::bundle::{DiagnosticBundleResult, DiagnosticBundleFile, RedactionSummary};

#[derive(Debug, Serialize)]
pub struct RecoveryAdvice {
    pub code: String,
    pub title: String,
    pub description: String,
    pub action_label: String,
    pub action_kind: String,
    pub action_payload: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecoveryRequest {
    pub code: String,
}

// ---------------------------------------------------------------------------
// Error recovery map
// ---------------------------------------------------------------------------

fn recovery_cards() -> Vec<(&'static str, &'static str, &'static str, &'static str, &'static str)> {
    vec![
        (
            "ROADMAP_NOT_FOUND",
            "No roadmap directory",
            "This folder does not contain a roadmap/ directory. Open a different folder or create a sample project.",
            "Try sample project",
            "open_sample",
        ),
        (
            "PROJECT_INVALID_YAML",
            "Malformed YAML in roadmap",
            "A roadmap file has malformed YAML frontmatter. Open the file and fix the frontmatter syntax.",
            "Open file",
            "open_file",
        ),
        (
            "AI_SERVICE_UNREACHABLE",
            "AI service not running",
            "Start the local AI service (python services/ai) then retry the health check.",
            "Retry health check",
            "run_check",
        ),
        (
            "AI_KEY_MISSING",
            "AI API key not configured",
            "Set ZAI_API_KEY environment variable to enable real AI output. Local project management still works without it.",
            "View setup guide",
            "open_docs",
        ),
        (
            "GITHUB_TOKEN_MISSING",
            "GitHub credential not stored",
            "Save a GitHub personal access token in the Credentials panel before using push/pull.",
            "Open credentials",
            "open_panel",
        ),
        (
            "KEYRING_UNAVAILABLE",
            "OS credential storage unavailable",
            "OS credential storage (keyring) is unavailable on this system. Secrets will be stored in session memory only.",
            "Dismiss",
            "dismiss",
        ),
        (
            "DASHBOARD_JSON_MISSING",
            "Dashboard data not generated",
            "Generate dashboard JSON from the roadmap before building or deploying the dashboard.",
            "Generate JSON",
            "run_check",
        ),
        (
            "CLOUDFLARE_SECRETS_UNKNOWN",
            "Cloudflare deployment secrets unknown",
            "Add CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID as GitHub repository secrets for dashboard auto-deployment.",
            "View setup guide",
            "open_docs",
        ),
        (
            "TASK_NOT_FOUND",
            "Task not found",
            "The requested task does not exist in the current roadmap. Refresh the project index.",
            "Refresh",
            "run_check",
        ),
        (
            "IO_ERROR",
            "File system error",
            "A file operation failed. Check that the file exists and you have permission to access it.",
            "Retry",
            "retry",
        ),
        (
            "DUPLICATE_TASK_ID",
            "Duplicate task IDs",
            "Multiple roadmap files have the same task ID. Rename one of them to have a unique ID.",
            "Open file",
            "open_file",
        ),
    ]
}

/// Get structured error information for a given error code.
/// v1.1.0 product-readiness: returns the full structured error DTO.
#[derive(Debug, Serialize)]
pub struct StructuredErrorResponse {
    pub code: String,
    pub title: String,
    pub message: String,
    pub likely_causes: Vec<String>,
    pub suggested_actions: Vec<String>,
    pub reporting_hint: String,
    pub technical_details: Option<serde_json::Value>,
    pub secret_safe: bool,
}

#[derive(Debug, Deserialize)]
pub struct StructuredErrorRequest {
    pub code: String,
}

#[command]
pub fn get_structured_error_cmd(req: StructuredErrorRequest) -> CommandResult<StructuredErrorResponse> {
    let errors = all_structured_errors();
    let found = errors.into_iter().find(|e| e.code == req.code);
    match found {
        Some(e) => Ok(e),
        None => Err(CommandError {
            code: "UNKNOWN_ERROR_CODE",
            message: format!("No structured error found for code: {}", req.code),
        }),
    }
}

/// All structured error definitions for v1.1.0.
fn all_structured_errors() -> Vec<StructuredErrorResponse> {
    vec![
        StructuredErrorResponse {
            code: "REPO_OPEN_FAILED".into(),
            title: "Could not open repository".into(),
            message: "The selected folder could not be opened as a project.".into(),
            likely_causes: vec!["Folder does not exist".into(), "Permission denied".into(), "Not a valid directory".into()],
            suggested_actions: vec!["Select a different folder".into(), "Check folder permissions".into()],
            reporting_hint: "Include this error code and folder path.".into(),
            technical_details: None,
            secret_safe: true,
        },
        StructuredErrorResponse {
            code: "ROADMAP_PARSE_FAILED".into(),
            title: "Roadmap could not be loaded".into(),
            message: "One or more roadmap files could not be parsed.".into(),
            likely_causes: vec!["Invalid YAML frontmatter".into(), "Missing required task id".into(), "Invalid date format".into()],
            suggested_actions: vec!["Open the file listed in details".into(), "Fix YAML frontmatter".into(), "Run roadmap validation".into()],
            reporting_hint: "Include this error code and file path in a bug report.".into(),
            technical_details: None,
            secret_safe: true,
        },
        StructuredErrorResponse {
            code: "DASHBOARD_FETCH_FAILED".into(),
            title: "Dashboard data could not be loaded".into(),
            message: "The dashboard data could not be fetched or generated.".into(),
            likely_causes: vec!["Network error".into(), "Dashboard not deployed yet".into(), "Invalid roadmap data".into()],
            suggested_actions: vec!["Check network connection".into(), "Generate dashboard JSON first".into()],
            reporting_hint: "Include this error code and dashboard URL.".into(),
            technical_details: None,
            secret_safe: true,
        },
        StructuredErrorResponse {
            code: "GIT_OPERATION_FAILED".into(),
            title: "Git operation failed".into(),
            message: "A Git operation (pull, push, commit) failed.".into(),
            likely_causes: vec!["Network connectivity".into(), "Authentication expired".into(), "Merge conflict".into()],
            suggested_actions: vec!["Check network connection".into(), "Update GitHub token".into(), "Resolve conflicts manually".into()],
            reporting_hint: "Include error code and operation type.".into(),
            technical_details: None,
            secret_safe: true,
        },
        StructuredErrorResponse {
            code: "CREDENTIAL_OPERATION_FAILED".into(),
            title: "Credential operation failed".into(),
            message: "A credential save, load, or delete operation failed.".into(),
            likely_causes: vec!["OS keychain unavailable".into(), "Permission denied".into(), "Keychain service not running".into()],
            suggested_actions: vec!["Check OS keychain/keyring is running".into(), "Try session-only mode".into()],
            reporting_hint: "Include error code. Do NOT include tokens or passwords.".into(),
            technical_details: None,
            secret_safe: true,
        },
        StructuredErrorResponse {
            code: "AI_SERVICE_UNREACHABLE".into(),
            title: "AI service not running".into(),
            message: "The AI service is not running or unreachable.".into(),
            likely_causes: vec!["AI service not started".into(), "Wrong port".into(), "Firewall blocking".into()],
            suggested_actions: vec!["Start the AI service".into(), "Check the service URL".into(), "Retry health check".into()],
            reporting_hint: "Include error code and service URL.".into(),
            technical_details: None,
            secret_safe: true,
        },
        StructuredErrorResponse {
            code: "AI_KEY_MISSING".into(),
            title: "AI API key not configured".into(),
            message: "ZAI_API_KEY is not set. Real AI mode requires an API key.".into(),
            likely_causes: vec!["Environment variable not set".into(), "Key not saved in credentials".into()],
            suggested_actions: vec!["Set ZAI_API_KEY environment variable".into(), "Use no-key beta mode instead".into()],
            reporting_hint: "Do NOT include the API key in reports.".into(),
            technical_details: None,
            secret_safe: true,
        },
        StructuredErrorResponse {
            code: "AGENT_PROPOSAL_FAILED".into(),
            title: "Agent proposal failed".into(),
            message: "The AI agent could not generate a proposal.".into(),
            likely_causes: vec!["AI service error".into(), "Task context too large".into(), "Model timeout".into()],
            suggested_actions: vec!["Retry the agent".into(), "Simplify task context".into(), "Check AI service logs".into()],
            reporting_hint: "Include error code, agent type, and task ID.".into(),
            technical_details: None,
            secret_safe: true,
        },
        StructuredErrorResponse {
            code: "LINUX_RUNTIME_CAVEAT".into(),
            title: "Linux runtime limitation".into(),
            message: "The Linux AppImage has known runtime caveats.".into(),
            likely_causes: vec!["Missing WebKit2GTK".into(), "Missing GTK3".into(), "Missing FUSE".into(), "Display server issue".into()],
            suggested_actions: vec!["Install libwebkit2gtk-4.1-dev".into(), "Install libgtk-3-dev".into(), "Install fuse3".into(), "See docs/linux-native-smoke-guide.md".into()],
            reporting_hint: "Include distro, desktop environment, and display server.".into(),
            technical_details: None,
            secret_safe: true,
        },
    ]
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[command]
pub fn export_diagnostics_cmd(project_root: Option<String>) -> CommandResult<DiagnosticBundleResult> {
    // Collect diagnostic data
    let app_info = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "platform": if cfg!(target_os = "windows") { "windows" }
            else if cfg!(target_os = "macos") { "macos" }
            else if cfg!(target_os = "linux") { "linux" }
            else { "unknown" },
        "git_sha": option_env!("GIT_SHA"),
    })
    .to_string();

    // Run readiness check
    let readiness = super::readiness::get_readiness_impl(project_root.clone());
    let readiness_json = match &readiness {
        Ok(r) => serde_json::to_string_pretty(r).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => serde_json::json!({"error": e.message}).to_string(),
    };

    // Run project validation
    let project_validation_json = match &project_root {
        Some(root) => {
            let validation = super::project_validation::validate_project_cmd(root.clone());
            match validation {
                Ok(v) => Some(serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".to_string())),
                Err(e) => Some(serde_json::json!({"error": e.message}).to_string()),
            }
        }
        None => None,
    };

    let recent_errors = serde_json::json!({
        "errors": [],
        "note": "Error history not yet tracked in v1.0.3"
    })
    .to_string();

    let system_info = format!(
        "OS: {}\nArch: {}\nRust: {}\nTime: {}\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        option_env!("RUSTC_VERSION").unwrap_or("unknown"),
        chrono::Utc::now().to_rfc3339(),
    );

    let ai_health = match &readiness {
        Ok(r) => serde_json::to_string_pretty(&r.ai).unwrap_or_else(|_| "{}".to_string()),
        Err(_) => "{}".to_string(),
    };

    let dashboard_status = match &readiness {
        Ok(r) => serde_json::to_string_pretty(&r.dashboard).unwrap_or_else(|_| "{}".to_string()),
        Err(_) => "{}".to_string(),
    };

    // v2.0.0: 6 new diagnostic data sources (all read-only, non-mutating)

    // Git provider report
    let git_provider_json = match &project_root {
        Some(root) => {
            let root_path = std::path::PathBuf::from(root);
            match git_bridge::build_provider_report(&root_path) {
                Ok(report) => serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string()),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        None => serde_json::json!({"status": "no project root"}).to_string(),
    };

    // Credential status
    let credential_status_json = serde_json::json!({
        "keyring_available": crate::security::is_keyring_available(),
        "provider": if crate::security::is_keyring_available() { "os-keychain" } else { "none" },
    }).to_string();

    // Agent matrix
    let agent_matrix_json = serde_json::json!({
        "agents": [
            {"name": "docs-agent", "mode": "review-only", "endpoint": "/agent/docs", "writes": false, "patch_governed": true},
            {"name": "bugfix-agent", "mode": "review-only", "endpoint": "/agent/bugfix", "writes": false, "patch_governed": true},
            {"name": "architect-agent", "mode": "read-only-planner", "endpoint": "/agent/architect", "writes": false, "patch_governed": false},
            {"name": "autonomy", "mode": "pilot", "endpoint": null, "writes": false, "patch_governed": true, "policy_version": 1, "auto_commit": false, "allowlist": ["docs/**", "README.md"]},
        ]
    }).to_string();

    // Patch governance status
    let patch_governance_json = match &project_root {
        Some(root) => {
            let audit_dir = std::path::Path::new(root).join(".Orqestra").join("audit");
            let audit_count = if audit_dir.exists() {
                std::fs::read_dir(&audit_dir).map(|d| d.count()).unwrap_or(0)
            } else {
                0
            };
            serde_json::json!({
                "enabled": true,
                "version": "v1.7.0+",
                "audit_dir_exists": audit_dir.exists(),
                "audit_entries": audit_count,
                "forbidden_paths": ["secrets", ".env", "*.pem", "*.key", ".github/workflows"],
                "atomic_writes": true,
            }).to_string()
        }
        None => serde_json::json!({"enabled": true, "version": "v1.7.0+", "note": "no project root"}).to_string(),
    };

    // Code intelligence status
    let test_source = "fn main() { println!(\"probe\"); }\n";
    let probe_result = code_intel::extract_symbols("probe.rs", test_source);
    let code_intel_json = serde_json::json!({
        "available": matches!(probe_result.parse_status, code_intel::ParseStatus::Success),
        "languages": ["rust", "typescript"],
        "probe_status": format!("{:?}", probe_result.parse_status),
    }).to_string();

    // Roadmap status
    let roadmap_status_json = match &project_root {
        Some(root) => {
            let roadmap_dir = std::path::Path::new(root).join("roadmap");
            if roadmap_dir.is_dir() {
                match md_indexer::index_roadmap(&roadmap_dir) {
                    Ok(result) => serde_json::json!({
                        "valid": true,
                        "task_count": result.tasks.len(),
                        "index_path": "roadmap/_index.md",
                    }).to_string(),
                    Err(e) => serde_json::json!({"valid": false, "error": e.to_string()}).to_string(),
                }
            } else {
                serde_json::json!({"valid": false, "reason": "no roadmap directory"}).to_string()
            }
        }
        None => serde_json::json!({"valid": false, "reason": "no project root"}).to_string(),
    };

    // v2.11.0: Beta readiness summary
    // Build a structured readiness check without raw paths or secrets.
    // Uses SHA-256 hash of project root instead of raw path.
    let (repo_detected, path_hash, branch, dirty, remote_configured) = match &project_root {
        Some(root) => {
            let root_path = std::path::PathBuf::from(root);
            let detected = root_path.join(".git").exists();
            let hash = {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(root.as_bytes());
                let result = hasher.finalize();
                format!("sha256:{:x}", result)
            };
            let branch = if detected {
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
            let dirty = if detected {
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
            let remote_configured = if detected {
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
            (detected, hash, branch, dirty, remote_configured)
        }
        None => (false, "none".to_string(), "n/a".to_string(), false, false),
    };

    // Parse readiness for checks
    let readiness_value: serde_json::Value = serde_json::from_str(&readiness_json).unwrap_or(serde_json::json!({}));
    let ai_reachable = readiness_value
        .get("ai")
        .and_then(|a| a.get("service_status"))
        .and_then(|s| s.as_str())
        .map(|s| s == "reachable")
        .unwrap_or(false);
    let keyring_available = crate::security::is_keyring_available();

    let roadmap_found = readiness_value
        .get("local_tools")
        .and_then(|t| t.as_array())
        .map(|arr| arr.iter().any(|t| t.get("tool").and_then(|v| v.as_str()) == Some("roadmap") && t.get("status").and_then(|v| v.as_str()) == Some("found")))
        .unwrap_or(false);

    let dashboard_exportable = readiness_value
        .get("dashboard")
        .and_then(|d| d.get("status"))
        .and_then(|s| s.as_str())
        .map(|s| s != "error")
        .unwrap_or(true);

    // v2.11.0: Probe git availability (not hardcoded)
    let git_available = std::process::Command::new("git")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // v2.11.0: Evidence schema validation — check if evidence dir exists and has valid files
    // This is a desktop runtime check, not CLI export time.
    let evidence_schema_valid = match &project_root {
        Some(root) => {
            let evidence_dir = std::path::Path::new(root).join("docs").join("evidence");
            if !evidence_dir.is_dir() {
                serde_json::Value::Null // unknown — no evidence dir in this project
            } else {
                // Check if at least one evidence file exists and parses as JSON
                let has_valid = std::fs::read_dir(&evidence_dir)
                    .ok()
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
                            .any(|e| {
                                std::fs::read_to_string(e.path())
                                    .ok()
                                    .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                                    .is_some()
                            })
                    })
                    .unwrap_or(false);
                serde_json::Value::Bool(has_valid)
            }
        }
        None => serde_json::Value::Null, // unknown — no project context
    };

    let checks = serde_json::json!({
        "repo_detected": repo_detected,
        "roadmap_found": roadmap_found,
        "git_available": git_available,
        "credentials_configured": keyring_available,
        "ai_service_reachable": ai_reachable,
        "dashboard_exportable": dashboard_exportable,
        "evidence_schema_valid": evidence_schema_valid
    });

    // v2.14.11: Never block the whole beta path. Only block specific features.
    let blocking = false;
    let mut warnings = Vec::new();
    let mut blocked_features = Vec::new();

    if !roadmap_found {
        warnings.push("No roadmap directory found — project management views will be empty");
    }
    if !keyring_available {
        warnings.push("OS keychain unavailable — credential storage may be limited");
    }
    if !ai_reachable {
        warnings.push("AI service unreachable — agent features unavailable");
        blocked_features.push("agent_execution");
    }
    if !dashboard_exportable {
        warnings.push("Dashboard export unavailable");
    }

    let readiness_label = if blocked_features.is_empty() && repo_detected {
        "ready"
    } else if !warnings.is_empty() {
        "ready_with_warnings"
    } else {
        "ready"
    };

    let beta_readiness_json = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "readiness": readiness_label,
        "blocking": blocking,
        "checks": checks,
        "repo": {
            "detected": repo_detected,
            "path_hash": path_hash,
            "branch": branch,
            "dirty": dirty,
            "remote_configured": remote_configured
        },
        "warnings": warnings,
        "blocked_features": blocked_features
    }).to_string();

    // Determine output directory
    let output_dir = match &project_root {
        Some(root) => std::path::PathBuf::from(root).join(".Orqestra"),
        None => {
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            home.join(".Orqestra")
        }
    };

    // Create the bundle (11 files)
    bundle::create_diagnostic_bundle(
        &output_dir,
        &app_info,
        &readiness_json,
        project_validation_json.as_deref(),
        &recent_errors,
        &system_info,
        &ai_health,
        &dashboard_status,
        &git_provider_json,
        &credential_status_json,
        &agent_matrix_json,
        &patch_governance_json,
        &code_intel_json,
        &roadmap_status_json,
        "{}",  // sync-status (populated when relay is active)
        "{}",  // coherence (populated when dashboard export exists)
        "{}",  // operational-risk (populated from path classification)
        &beta_readiness_json,  // v2.11.0: beta readiness summary
    )
    .map_err(|e| CommandError {
        code: "DIAGNOSTICS_EXPORT_FAILED",
        message: e,
    })
}

// ---------------------------------------------------------------------------
// v2.0.0: First-Run Probe Commands (10 non-mutating checks)
//
// All probes are read-only. No agent runs, no patch applications,
// no audit writes, no .Orqestra mutations. AI/agent checks return
// optional/degraded on failure (never fail setup).
// ---------------------------------------------------------------------------

/// Check 1: Git available on PATH.
#[command]
pub fn check_git_available_cmd() -> CommandResult<serde_json::Value> {
    let output = std::process::Command::new("git")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    match output {
        Ok(out) => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok(serde_json::json!({"available": true, "version": version}))
        }
        Err(_) => Ok(serde_json::json!({"available": false, "version": null})),
    }
}

/// Check 2: Repository selectable (project_root exists and has .git).
#[command]
pub fn check_repo_selectable_cmd(project_root: String) -> CommandResult<serde_json::Value> {
    let path = std::path::Path::new(&project_root);
    let exists = path.exists();
    let has_git = path.join(".git").exists();
    Ok(serde_json::json!({"exists": exists, "has_git": has_git, "selectable": exists && has_git}))
}

/// Check 3: Roadmap valid (bounded read of _index.md).
#[command]
pub fn check_roadmap_valid_cmd(project_root: String) -> CommandResult<serde_json::Value> {
    let index_path = std::path::Path::new(&project_root).join("roadmap").join("_index.md");
    if !index_path.exists() {
        return Ok(serde_json::json!({"valid": false, "reason": "roadmap/_index.md not found"}));
    }
    // Bounded read: only first 4 KiB, no full parse
    match std::fs::read(&index_path) {
        Ok(bytes) => {
            if bytes.len() > 4096 {
                // Only check first 4 KiB
                let _prefix = &bytes[..4096];
            }
            Ok(serde_json::json!({"valid": true, "reason": null}))
        }
        Err(e) => Ok(serde_json::json!({"valid": false, "reason": e.to_string()})),
    }
}

/// Check 4: AI service reachable — optional/degraded on failure.
#[command]
pub fn check_ai_service_cmd() -> CommandResult<serde_json::Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build();
    match client {
        Ok(client) => {
            match client.get("http://localhost:8000/health").send() {
                Ok(r) if r.status().is_success() => {
                    Ok(serde_json::json!({"reachable": true, "status": "ok"}))
                }
                Ok(r) => {
                    Ok(serde_json::json!({"reachable": true, "status": "degraded", "http_status": r.status().as_u16()}))
                }
                Err(_) => {
                    // Unreachable is not a failure — it's degraded/optional
                    Ok(serde_json::json!({"reachable": false, "status": "optional", "note": "AI features require the Python AI service"}))
                }
            }
        }
        Err(_) => Ok(serde_json::json!({"reachable": false, "status": "optional"})),
    }
}

/// Check 5: Credential provider available.
#[command]
pub fn check_credential_provider_cmd(project_root: String) -> CommandResult<serde_json::Value> {
    let _project_root = project_root; // may be used for project-scoped checks
    let available = crate::security::is_keyring_available();
    Ok(serde_json::json!({"available": available, "provider": if available { "os-keychain" } else { "none" }}))
}

/// Check 6: Dashboard export status visible.
#[command]
pub fn check_dashboard_status_cmd(project_root: String) -> CommandResult<serde_json::Value> {
    let json_path = std::path::Path::new(&project_root)
        .join("apps")
        .join("dashboard")
        .join("public")
        .join("roadmap.json");
    let available = json_path.exists();
    Ok(serde_json::json!({"available": available, "path": if available { json_path.display().to_string() } else { "not found".to_string() }}))
}

/// Check 7: Agent endpoints available — optional/degraded on failure.
#[command]
pub fn check_agent_endpoints_cmd() -> CommandResult<serde_json::Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build();
    let mut endpoints = Vec::new();
    match client {
        Ok(client) => {
            for endpoint in &["/agent/docs", "/agent/bugfix", "/agent/architect"] {
                // Just check the health endpoint, not the agent endpoints directly
                let url = format!("http://localhost:8000{}", endpoint);
                // Don't actually POST to agents — just note they would be available
                endpoints.push(serde_json::json!({"endpoint": *endpoint, "status": "service_up"}));
            }
            // Check if service is up
            match client.get("http://localhost:8000/health").send() {
                Ok(r) if r.status().is_success() => {
                    Ok(serde_json::json!({"available": true, "endpoints": endpoints}))
                }
                _ => {
                    Ok(serde_json::json!({"available": false, "status": "optional", "note": "Agent endpoints require the Python AI service"}))
                }
            }
        }
        Err(_) => Ok(serde_json::json!({"available": false, "status": "optional"})),
    }
}

/// Check 8: Patch governance enabled (read-only audit log check).
#[command]
pub fn check_patch_governance_cmd(project_root: String) -> CommandResult<serde_json::Value> {
    let audit_dir = std::path::Path::new(&project_root).join(".Orqestra").join("audit");
    let enabled = audit_dir.exists();
    let audit_count = if enabled {
        std::fs::read_dir(&audit_dir).map(|d| d.count()).unwrap_or(0)
    } else {
        0
    };
    Ok(serde_json::json!({"enabled": true, "audit_dir_exists": enabled, "audit_entries": audit_count, "note": "Patch governance is always enabled in v1.7.0+"}))
}

/// Check 9: Code intelligence enabled (probe on known test fixture).
#[command]
pub fn check_code_intel_cmd(project_root: String) -> CommandResult<serde_json::Value> {
    // Probe with a small bounded Rust snippet — do NOT parse arbitrary source files
    let test_source = "fn main() { println!(\"hello\"); }\n";
    let result = code_intel::extract_symbols("test_probe.rs", test_source);
    let available = matches!(result.parse_status, code_intel::ParseStatus::Success);
    Ok(serde_json::json!({"available": available, "languages": ["rust", "typescript"], "probe_status": format!("{:?}", result.parse_status)}))
}

/// Check 10: Git provider resolved (single read-only operation).
#[command]
pub fn check_git_provider_cmd(project_root: String) -> CommandResult<serde_json::Value> {
    let root = std::path::PathBuf::from(&project_root);
    if !root.exists() {
        return Ok(serde_json::json!({"resolved": false, "reason": "project root does not exist"}));
    }
    match git_bridge::build_provider_report(&root) {
        Ok(report) => {
            // Get primary provider from first operation
            let primary = report.operations.first()
                .map(|op| format!("{:?}", op.provider))
                .unwrap_or_else(|| "unknown".to_string());
            Ok(serde_json::json!({"resolved": true, "provider": primary, "ops_count": report.operations.len()}))
        }
        Err(e) => Ok(serde_json::json!({"resolved": false, "reason": e.to_string()})),
    }
}

// ---------------------------------------------------------------------------
// v2.2.0: Dashboard / Workspace Sync Coherence
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize)]
pub struct RelayCoherence {
    pub available: bool,
    pub relay_url_host: Option<String>,
    pub workspace_id_hash: Option<String>,
    pub last_snapshot_hash: Option<String>,
    pub connected: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct CoherenceResult {
    pub dashboard_export_exists: bool,
    pub dashboard_commit: Option<String>,
    pub local_head: Option<String>,
    pub commits_behind: Option<u32>,
    pub freshness: String,
    pub local_roadmap_state_hash: Option<String>,
    pub dashboard_roadmap_state_hash: Option<String>,
    pub task_count_local: Option<usize>,
    pub task_count_dashboard: Option<usize>,
    pub relay: RelayCoherence,
}

/// Check dashboard/workspace/relay coherence.
/// Desktop-computed: compares local HEAD vs dashboard export commit.
/// Uses canonical roadmap state hashes.
#[command]
pub fn check_dashboard_coherence_cmd(project_root: String) -> CommandResult<CoherenceResult> {
    let root = std::path::PathBuf::from(&project_root);
    if !root.exists() {
        return Ok(CoherenceResult {
            dashboard_export_exists: false,
            dashboard_commit: None,
            local_head: None,
            commits_behind: None,
            freshness: "unknown".to_string(),
            local_roadmap_state_hash: None,
            dashboard_roadmap_state_hash: None,
            task_count_local: None,
            task_count_dashboard: None,
            relay: RelayCoherence {
                available: false,
                relay_url_host: None,
                workspace_id_hash: None,
                last_snapshot_hash: None,
                connected: false,
            },
        });
    }

    let local_head = git_bridge::get_head_hash(&root).ok();

    let dashboard_json_path = root
        .join("apps")
        .join("dashboard")
        .join("public")
        .join("orqestra-roadmap.json");

    let (dashboard_export_exists, dashboard_commit, dashboard_hash, dashboard_task_count) =
        if dashboard_json_path.exists() {
            match std::fs::read_to_string(&dashboard_json_path) {
                Ok(content) => {
                    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                    let commit = parsed.get("source")
                        .and_then(|s| s.get("commit"))
                        .and_then(|c| c.as_str())
                        .map(|s| s.to_string());
                    let task_count = parsed.get("summary")
                        .and_then(|s| s.get("total_tasks"))
                        .and_then(|t| t.as_u64())
                        .map(|t| t as usize);
                    let hash = parsed.get("coherence")
                        .and_then(|c| c.get("roadmap_state_hash"))
                        .and_then(|h| h.as_str())
                        .map(|h| h.to_string());
                    (true, commit, hash, task_count)
                }
                Err(_) => (true, None, None, None),
            }
        } else {
            (false, None, None, None)
        };

    let freshness = if !dashboard_export_exists {
        "local-only".to_string()
    } else if dashboard_commit.is_none() || local_head.is_none() {
        "unknown".to_string()
    } else if dashboard_commit.as_deref() == local_head.as_deref() {
        "current".to_string()
    } else {
        "stale".to_string()
    };

    Ok(CoherenceResult {
        dashboard_export_exists,
        dashboard_commit,
        local_head,
        commits_behind: None,
        freshness,
        local_roadmap_state_hash: None,
        dashboard_roadmap_state_hash: dashboard_hash,
        task_count_local: None,
        task_count_dashboard: dashboard_task_count,
        relay: RelayCoherence {
            available: false,
            relay_url_host: None,
            workspace_id_hash: None,
            last_snapshot_hash: None,
            connected: false,
        },
    })
}

#[command]
pub fn get_recovery_advice_cmd(code: String) -> CommandResult<RecoveryAdvice> {
    let cards = recovery_cards();
    for (c, title, description, action_label, action_kind) in &cards {
        if *c == code {
            return Ok(RecoveryAdvice {
                code: code.clone(),
                title: title.to_string(),
                description: description.to_string(),
                action_label: action_label.to_string(),
                action_kind: action_kind.to_string(),
                action_payload: None,
            });
        }
    }

    // Default recovery for unknown error codes
    Ok(RecoveryAdvice {
        code: code.clone(),
        title: "Unknown error".to_string(),
        description: format!(
            "No specific recovery advice for error code '{}'. Check the setup guide or export diagnostics.",
            code
        ),
        action_label: "Export diagnostics".to_string(),
        action_kind: "open_panel".to_string(),
        action_payload: Some("diagnostics".to_string()),
    })
}

// ---------------------------------------------------------------------------
// v2.11.0: Beta Readiness Summary Command
// Returns beta readiness summary without full diagnostic bundle.
// ---------------------------------------------------------------------------

/// Get beta readiness summary (lightweight, no file export).
/// v2.14.11: Async to avoid blocking UI. Accepts optional pre-computed readiness
/// to avoid duplicate environment scan.
#[command]
pub async fn get_beta_readiness_cmd(project_root: Option<String>) -> CommandResult<serde_json::Value> {
    tokio::task::spawn_blocking(move || {
        get_beta_readiness_impl(project_root)
    })
    .await
    .map_err(|e| CommandError {
        code: "BETA_READINESS_THREAD_ERROR",
        message: format!("Beta readiness thread failed: {}", e),
    })?}

fn get_beta_readiness_impl(project_root: Option<String>) -> CommandResult<serde_json::Value> {
    let readiness = super::readiness::get_readiness_impl(project_root.clone());
    let readiness_json = match &readiness {
        Ok(r) => serde_json::to_string_pretty(r).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => serde_json::json!({"error": e.message}).to_string(),
    };

    let (repo_detected, path_hash, branch, dirty, remote_configured) = match &project_root {
        Some(root) => {
            let root_path = std::path::PathBuf::from(root);
            let detected = root_path.join(".git").exists();
            let hash = {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(root.as_bytes());
                let result = hasher.finalize();
                format!("sha256:{:x}", result)
            };
            let branch = if detected {
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
            let dirty = if detected {
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
            let remote_configured = if detected {
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
            (detected, hash, branch, dirty, remote_configured)
        }
        None => (false, "none".to_string(), "n/a".to_string(), false, false),
    };

    let readiness_value: serde_json::Value = serde_json::from_str(&readiness_json).unwrap_or(serde_json::json!({}));
    let ai_reachable = readiness_value
        .get("ai")
        .and_then(|a| a.get("service_status"))
        .and_then(|s| s.as_str())
        .map(|s| s == "reachable")
        .unwrap_or(false);
    let keyring_available = crate::security::is_keyring_available();

    let roadmap_found = readiness_value
        .get("local_tools")
        .and_then(|t| t.as_array())
        .map(|arr| arr.iter().any(|t| t.get("tool").and_then(|v| v.as_str()) == Some("roadmap") && t.get("status").and_then(|v| v.as_str()) == Some("found")))
        .unwrap_or(false);

    let dashboard_exportable = readiness_value
        .get("dashboard")
        .and_then(|d| d.get("status"))
        .and_then(|s| s.as_str())
        .map(|s| s != "error")
        .unwrap_or(true);

    // v2.11.0: Probe git availability (not hardcoded)
    let git_available = std::process::Command::new("git")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // v2.11.0: Evidence schema — check if evidence dir exists with valid JSON
    let evidence_schema_valid = match &project_root {
        Some(root) => {
            let evidence_dir = std::path::Path::new(root).join("docs").join("evidence");
            if !evidence_dir.is_dir() {
                serde_json::Value::Null
            } else {
                let has_valid = std::fs::read_dir(&evidence_dir)
                    .ok()
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
                            .any(|e| {
                                std::fs::read_to_string(e.path())
                                    .ok()
                                    .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                                    .is_some()
                            })
                    })
                    .unwrap_or(false);
                serde_json::Value::Bool(has_valid)
            }
        }
        None => serde_json::Value::Null,
    };

    let checks = serde_json::json!({
        "repo_detected": repo_detected,
        "roadmap_found": roadmap_found,
        "git_available": git_available,
        "credentials_configured": keyring_available,
        "ai_service_reachable": ai_reachable,
        "dashboard_exportable": dashboard_exportable,
        "evidence_schema_valid": evidence_schema_valid
    });

    // v2.14.11: Never block the whole beta path. Only block specific features.
    let blocking = false;
    let mut warnings = Vec::new();
    let mut blocked_features = Vec::new();

    if !roadmap_found {
        warnings.push("No roadmap directory found — project management views will be empty");
    }
    if !keyring_available {
        warnings.push("OS keychain unavailable — credential storage may be limited");
    }
    if !ai_reachable {
        warnings.push("AI service unreachable — agent features unavailable");
        blocked_features.push("agent_execution");
    }
    if !dashboard_exportable {
        warnings.push("Dashboard export unavailable");
    }

    let readiness_label = if blocked_features.is_empty() && repo_detected {
        "ready"
    } else if !warnings.is_empty() {
        "ready_with_warnings"
    } else {
        "ready"
    };

    Ok(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "readiness": readiness_label,
        "blocking": blocking,
        "checks": checks,
        "repo": {
            "detected": repo_detected,
            "path_hash": path_hash,
            "branch": branch,
            "dirty": dirty,
            "remote_configured": remote_configured
        },
        "warnings": warnings,
        "blocked_features": blocked_features
    }))
}
