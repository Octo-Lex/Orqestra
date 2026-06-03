//! Diagnostics export and error recovery commands.
//!
//! Provides user-facing diagnostic bundle export with secret redaction
//! and error-code-to-recovery-action mapping.

use crate::diagnostics::bundle;
use serde::{Deserialize, Serialize};
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
    let readiness = super::readiness::get_readiness_cmd(project_root.clone());
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

    // Determine output directory
    let output_dir = match &project_root {
        Some(root) => std::path::PathBuf::from(root).join(".Orqestra"),
        None => {
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            home.join(".Orqestra")
        }
    };

    // Create the bundle
    bundle::create_diagnostic_bundle(
        &output_dir,
        &app_info,
        &readiness_json,
        project_validation_json.as_deref(),
        &recent_errors,
        &system_info,
        &ai_health,
        &dashboard_status,
    )
    .map_err(|e| CommandError {
        code: "DIAGNOSTICS_EXPORT_FAILED",
        message: e,
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
