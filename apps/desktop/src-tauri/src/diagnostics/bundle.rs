//! Diagnostic bundle creation.
//!
//! Creates a directory of diagnostic files with all secrets redacted.

use super::redaction::{redact_text, redaction_rule_descriptions};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// Result of exporting a diagnostic bundle.
#[derive(Debug, Serialize)]
pub struct DiagnosticBundleResult {
    pub path: String,
    pub created_at: String,
    pub files: Vec<DiagnosticBundleFile>,
    pub redaction_summary: RedactionSummary,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticBundleFile {
    pub name: String,
    pub description: String,
    pub bytes: usize,
}

#[derive(Debug, Serialize)]
pub struct RedactionSummary {
    pub rules_applied: Vec<String>,
    pub redacted_value_count: usize,
    pub contains_raw_secrets: bool,
}

/// Create a diagnostic bundle at the given output directory.
/// v2.0.0: 11 files total (5 existing + 6 new).
/// All data is read from existing state. Non-mutating.
pub fn create_diagnostic_bundle(
    output_dir: &std::path::Path,
    app_json: &str,
    readiness_json: &str,
    project_validation_json: Option<&str>,
    recent_errors_json: &str,
    system_info: &str,
    ai_health_json: &str,
    dashboard_status_json: &str,
    // v2.0.0: new diagnostic files
    git_provider_json: &str,
    credential_status_json: &str,
    agent_matrix_json: &str,
    patch_governance_json: &str,
    code_intel_json: &str,
    roadmap_status_json: &str,
    sync_status_json: &str,
    coherence_json: &str,
    operational_risk_json: &str,
) -> Result<DiagnosticBundleResult, String> {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let bundle_name = format!("orqestra-diagnostics-{}", timestamp);
    let bundle_path = output_dir.join(&bundle_name);

    fs::create_dir_all(&bundle_path).map_err(|e| format!("Failed to create bundle dir: {}", e))?;

    let mut files = Vec::new();
    let mut total_redacted = 0;
    let mut all_rules: Vec<String> = Vec::new();

    // Write each file with redaction
    let entries: Vec<(&str, &str, &str)> = vec![
        ("app.json", app_json, "App version and platform info"),
        ("readiness.json", readiness_json, "Environment readiness report"),
        (
            "project-validation.json",
            project_validation_json.unwrap_or("{}"),
            "Project validation result",
        ),
        (
            "recent-errors.json",
            recent_errors_json,
            "Recent command errors",
        ),
        ("system.txt", system_info, "System information"),
        ("ai-health.json", ai_health_json, "AI service health check"),
        (
            "dashboard-status.json",
            dashboard_status_json,
            "Dashboard deployment status",
        ),
        // v2.0.0: 6 new diagnostic files
        (
            "git-provider.json",
            git_provider_json,
            "Git provider diagnostics per operation",
        ),
        (
            "credential-status.json",
            credential_status_json,
            "Credential provider availability",
        ),
        (
            "agent-matrix.json",
            agent_matrix_json,
            "Agent mode, endpoint, and availability",
        ),
        (
            "patch-governance.json",
            patch_governance_json,
            "Patch governance policy and audit status",
        ),
        (
            "code-intel.json",
            code_intel_json,
            "Code intelligence languages and parse status",
        ),
        (
            "roadmap-status.json",
            roadmap_status_json,
            "Roadmap parse status and task count",
        ),
        // v2.1.0: Sync relay status (redacted)
        (
            "sync-status.json",
            sync_status_json,
            "Sync relay connection status (redacted)",
        ),
        // v2.2.0: Dashboard/workspace/relay coherence
        (
            "coherence.json",
            coherence_json,
            "Dashboard/workspace/relay coherence (redacted)",
        ),
        // v2.4.0: Operational risk summary
        (
            "operational-risk.json",
            operational_risk_json,
            "Operational risk classification (redacted)",
        ),
    ];

    for (name, content, description) in &entries {
        let result = redact_text(content);
        total_redacted += result.redacted_value_count;
        for rule in result.rules_applied {
            if !all_rules.contains(&rule) {
                all_rules.push(rule);
            }
        }

        let file_path = bundle_path.join(name);
        fs::write(&file_path, &result.redacted_text)
            .map_err(|e| format!("Failed to write {}: {}", name, e))?;
        let bytes = result.redacted_text.len();
        files.push(DiagnosticBundleFile {
            name: name.to_string(),
            description: description.to_string(),
            bytes,
        });
    }

    // Write README.txt
    let readme = format!(
        "Orqestra Diagnostic Bundle\n\
         Generated: {}\n\
         \n\
         This bundle contains diagnostic information with all secrets redacted.\n\
         Redaction rules applied: {}\n\
         Total values redacted: {}\n\
         \n\
         Files:\n{}\n\
         \n\
         Do NOT add raw secrets to this bundle.\n\
         If you need to share this, review the contents first.\n",
        chrono::Utc::now().to_rfc3339(),
        all_rules.len(),
        total_redacted,
        files
            .iter()
            .map(|f| format!("  - {} ({} bytes): {}", f.name, f.bytes, f.description))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let readme_path = bundle_path.join("README.txt");
    fs::write(&readme_path, &readme).map_err(|e| format!("Failed to write README.txt: {}", e))?;
    files.push(DiagnosticBundleFile {
        name: "README.txt".to_string(),
        description: "Bundle overview and redaction summary".to_string(),
        bytes: readme.len(),
    });

    Ok(DiagnosticBundleResult {
        path: bundle_path.display().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        files,
        redaction_summary: RedactionSummary {
            rules_applied: all_rules,
            redacted_value_count: total_redacted,
            contains_raw_secrets: false,
        },
    })
}
