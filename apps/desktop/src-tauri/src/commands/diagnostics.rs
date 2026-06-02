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
