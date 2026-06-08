use crate::security::patch_guard::{
    AgentType, PatchProposal, PatchApplicationResult,
    apply_agent_patch, reject_agent_patch,
};
use crate::security::auto_apply::{
    decide_auto_apply, build_auto_apply_audit, reset_session_counter,
    session_auto_apply_count, compute_patch_size, validate_autonomy_enable,
    increment_session_counter,
};
use crate::commands::onboarding_types::{
    AutonomySettings, AutoApplyDecision, AutoApplyResult,
    AutonomySettingsUpdate,
};
use crate::commands::onboarding::OnboardingStateManager;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::command;

// ---------------------------------------------------------------------------
// v1.9.0: Architect Agent DTOs
//
// Read-only planner. No patch-shaped fields (no before/after/edits).
// Cannot be passed to apply_agent_patch_cmd.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRef {
    pub name: String,
    pub kind: String,
    pub file: String,
    #[serde(default)]
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskItem {
    pub risk: String,
    pub severity: String,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBreakdownItem {
    pub task: String,
    pub scope: String,
    pub complexity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectPlanResult {
    pub plan_id: String,
    pub schema_version: String,
    pub summary: String,
    pub context_analysis: String,
    pub proposed_approach: Vec<String>,
    pub affected_symbols: Vec<SymbolRef>,
    pub risk_assessment: Vec<RiskItem>,
    pub dependency_warnings: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub test_strategy: Vec<String>,
    pub task_breakdown: Vec<TaskBreakdownItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adr_draft: Option<String>,
    pub confidence: f64,
    // Structural guarantee: this DTO has no before/after/edits/patch fields.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectAgentResult {
    pub plan: ArchitectPlanResult,
    pub agent: String,
    pub mode: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Read a file from disk. Used by SkillLoader and workspace state loading.
#[command]
pub fn read_file_cmd(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path, e))
}

/// Write a file to disk. Used for workspace state persistence and agent file writes.
#[command]
pub fn write_file_cmd(path: String, content: String) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", path, e))
}

/// Run agent: calls the AI service /run-agent endpoint.
/// In production this would HTTP POST to the Python service.
/// For now it returns a structured response indicating the service call.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunAgentResult {
    pub workspace_id: String,
    pub task_id: String,
    pub status: String,
    pub message: String,
}

#[command]
pub fn run_agent_cmd(
    project_root: String,
    workspace_id: String,
    _model: String,
    _prompt: String,
    task_id: String,
) -> Result<String, String> {
    // Write workspace run state to .Orqestra/agents/<workspace_id>/
    let state_dir = std::path::PathBuf::from(&project_root)
        .join(".Orqestra")
        .join("agents")
        .join(&workspace_id);
    fs::create_dir_all(&state_dir).map_err(|e| format!("Failed to create agent dir: {}", e))?;

    let run_state = serde_json::json!({
        "workspaceId": workspace_id,
        "taskId": task_id,
        "status": "running",
        "startedAt": chrono::Utc::now().to_rfc3339(),
    });

    let state_path = state_dir.join("state.json");
    fs::write(&state_path, serde_json::to_string_pretty(&run_state).unwrap())
        .map_err(|e| format!("Failed to write state: {}", e))?;

    // Return structured result — TypeScript side will use mock if service unavailable
    Ok(serde_json::to_string(&RunAgentResult {
        workspace_id,
        task_id,
        status: "dispatched".to_string(),
        message: "Agent dispatched. Use mock response if AI service unavailable.".to_string(),
    })
    .unwrap())
}

/// Run docs agent: calls the real AI service /agent/docs endpoint.
/// This is the first real agent execution path (spec §9).
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocsAgentEdit {
    pub path: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocsAgentResult {
    pub summary: String,
    pub confidence: f64,
    pub has_breaking_change: bool,
    pub edits: Vec<DocsAgentEdit>,
    pub notes: Vec<String>,
}

#[command]
pub fn run_docs_agent_cmd(
    project_root: String,
    task: String, // JSON string of task object
    context_files: String, // JSON string of [{path, content}]
) -> Result<String, String> {
    let task_obj: serde_json::Value = serde_json::from_str(&task)
        .map_err(|e| format!("Invalid task JSON: {}", e))?;
    let files: Vec<serde_json::Value> = serde_json::from_str(&context_files)
        .map_err(|e| format!("Invalid context_files JSON: {}", e))?;

    // Build safe Git context v2 (content-free, schema-versioned)
    let git_context = std::path::PathBuf::from(&project_root);
    let (safe_context, git_context_status, git_context_error_code) =
        match git_bridge::build_agent_context_v2(&git_context) {
            Ok(ctx) => (
                serde_json::to_value(&ctx).unwrap_or(serde_json::json!({})),
                "available".to_string(),
                serde_json::Value::Null,
            ),
            Err(_) => (
                serde_json::json!({}),
                "unavailable".to_string(),
                serde_json::json!("AGENT_CONTEXT_BUILD_FAILED"),
            ),
        };

    // Build request body
    let request_body = serde_json::json!({
        "task": task_obj,
        "context_files": files,
        "git_context": safe_context,
        "git_context_status": git_context_status,
        "git_context_error_code": git_context_error_code,
        "constraints": {
            "allowed_paths": ["README.md", "docs/", "roadmap/", "CHANGELOG.md"],
            "max_files_changed": 3,
            "review_only": true,
            "auto_commit": false,
            "auto_apply": false
        }
    });

    // Call the AI service synchronously (blocking reqwest, runs on Tauri threadpool)
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("http://localhost:8000/agent/docs")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .map_err(|e| format!("AI service unreachable: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("AI service error {}: {}", status, body));
    }

    let result: serde_json::Value = response
        .json()
        .map_err(|e| format!("Invalid AI response: {}", e))?;

    // Validate edits are within allowed paths
    let allowed = ["README.md", "docs/", "roadmap/", "CHANGELOG.md"];
    let mut filtered_edits = Vec::new();

    if let Some(edits) = result.get("edits").and_then(|e| e.as_array()) {
        for edit in edits {
            if let Some(path) = edit.get("path").and_then(|p| p.as_str()) {
                let normalized = path.replace("\\", "/");
                let is_allowed = allowed.iter().any(|prefix| {
                    normalized.starts_with(prefix) || normalized == prefix.trim_end_matches('/')
                });
                if is_allowed {
                    filtered_edits.push(DocsAgentEdit {
                        path: path.to_string(),
                        before: edit.get("before").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        after: edit.get("after").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    });
                }
            }
        }
    }

    let agent_result = DocsAgentResult {
        summary: result.get("summary").and_then(|v| v.as_str()).unwrap_or("No summary").to_string(),
        confidence: result.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0),
        has_breaking_change: result.get("has_breaking_change").and_then(|v| v.as_bool()).unwrap_or(false),
        edits: filtered_edits,
        notes: result.get("notes")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    };

    // Write agent run state
    let state_dir = std::path::PathBuf::from(&project_root)
        .join(".Orqestra")
        .join("agents")
        .join("docs");
    std::fs::create_dir_all(&state_dir).ok();
    let run_state = serde_json::json!({
        "workspaceId": "docs",
        "summary": agent_result.summary,
        "confidence": agent_result.confidence,
        "editCount": agent_result.edits.len(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    std::fs::write(state_dir.join("state.json"), serde_json::to_string_pretty(&run_state).unwrap()).ok();

    Ok(serde_json::to_string(&agent_result).unwrap())
}

// ---------------------------------------------------------------------------
// v1.0.2: Bugfix agent commands (Workstream E)
// ---------------------------------------------------------------------------

/// Read a project file for the bugfix agent file scope selector.
#[command]
pub fn read_project_file_cmd(
    project_root: String,
    path: String,
) -> Result<String, String> {
    let full_path = std::path::PathBuf::from(&project_root).join(&path);

    // Security: must be within project root
    let canonical_root = std::path::PathBuf::from(&project_root).canonicalize()
        .map_err(|e| format!("Invalid project root: {e}"))?;
    let canonical_file = full_path.canonicalize()
        .map_err(|e| format!("Invalid file path: {e}"))?;
    if !canonical_file.starts_with(&canonical_root) {
        return Err("Path traversal blocked".into());
    }

    fs::read_to_string(&full_path)
        .map_err(|e| format!("Failed to read {}: {}", path, e))
}

/// Bugfix agent result DTO
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BugfixAgentEdit {
    pub path: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BugfixAgentResult {
    pub summary: String,
    pub confidence: f64,
    pub has_breaking_change: bool,
    pub edits: Vec<BugfixAgentEdit>,
    pub needs_more_files: bool,
    pub requested_files: Vec<RequestedFile>,
    pub notes: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestedFile {
    pub path: String,
    pub reason: String,
}

/// Run bugfix agent: calls the AI service /agent/bugfix endpoint.
/// Only user-selected files are included in the request.
#[command]
pub fn run_bugfix_agent_cmd(
    project_root: String,
    task: String,
    allowed_files: String,
) -> Result<String, String> {
    let task_obj: serde_json::Value = serde_json::from_str(&task)
        .map_err(|e| format!("Invalid task JSON: {e}"))?;
    let files: Vec<serde_json::Value> = serde_json::from_str(&allowed_files)
        .map_err(|e| format!("Invalid allowed_files JSON: {e}"))?;

    // Extract allowed paths for validation
    let allowed_paths: Vec<String> = files.iter()
        .filter_map(|f| f.get("path").and_then(|p| p.as_str()).map(String::from))
        .collect();

    // Build safe Git context v2 (content-free, schema-versioned)
    let git_ctx_path = std::path::PathBuf::from(&project_root);
    let (safe_context, git_context_status, git_context_error_code) =
        match git_bridge::build_agent_context_v2(&git_ctx_path) {
            Ok(ctx) => (
                serde_json::to_value(&ctx).unwrap_or(serde_json::json!({})),
                "available".to_string(),
                serde_json::Value::Null,
            ),
            Err(_) => (
                serde_json::json!({}),
                "unavailable".to_string(),
                serde_json::json!("AGENT_CONTEXT_BUILD_FAILED"),
            ),
        };

    let request_body = serde_json::json!({
        "task": task_obj,
        "allowed_files": files,
        "git_context": safe_context,
        "git_context_status": git_context_status,
        "git_context_error_code": git_context_error_code,
        "constraints": {
            "allowed_paths": allowed_paths,
            "max_files_changed": allowed_paths.len(),
            "review_only": true,
            "auto_commit": false,
            "auto_apply": false,
            "may_request_more_files": true
        }
    });

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("http://localhost:8000/agent/bugfix")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(45))
        .send()
        .map_err(|e| format!("AI service unreachable: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("AI service error {status}: {body}"));
    }

    let result: serde_json::Value = response
        .json()
        .map_err(|e| format!("Invalid AI response: {e}"))?;

    // Validate edits are within allowed paths
    let mut filtered_edits = Vec::new();
    if let Some(edits) = result.get("edits").and_then(|e| e.as_array()) {
        for edit in edits {
            if let Some(path) = edit.get("path").and_then(|p| p.as_str()) {
                if allowed_paths.contains(&path.to_string()) {
                    filtered_edits.push(BugfixAgentEdit {
                        path: path.to_string(),
                        before: edit.get("before").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        after: edit.get("after").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    });
                }
                // Silently drop edits to non-allowed files
            }
        }
    }

    let requested_files: Vec<RequestedFile> = result.get("requested_files")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| {
            Some(RequestedFile {
                path: v.get("path")?.as_str()?.to_string(),
                reason: v.get("reason")?.as_str()?.to_string(),
            })
        }).collect())
        .unwrap_or_default();

    let agent_result = BugfixAgentResult {
        summary: result.get("summary").and_then(|v| v.as_str()).unwrap_or("No summary").to_string(),
        confidence: result.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0),
        has_breaking_change: result.get("has_breaking_change").and_then(|v| v.as_bool()).unwrap_or(false),
        edits: filtered_edits,
        needs_more_files: result.get("needs_more_files").and_then(|v| v.as_bool()).unwrap_or(false),
        requested_files,
        notes: result.get("notes")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    };

    // Write agent run state
    let state_dir = std::path::PathBuf::from(&project_root)
        .join(".Orqestra")
        .join("agents")
        .join("bugfix");
    std::fs::create_dir_all(&state_dir).ok();
    let run_state = serde_json::json!({
        "workspaceId": "bugfix",
        "summary": agent_result.summary,
        "confidence": agent_result.confidence,
        "editCount": agent_result.edits.len(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    std::fs::write(state_dir.join("state.json"), serde_json::to_string_pretty(&run_state).unwrap()).ok();

    Ok(serde_json::to_string(&agent_result).unwrap())
}

/// List all workspace directories under agents/workspaces/
#[derive(Debug, serde::Serialize)]
pub struct WorkspaceEntry {
    pub dir: String,
    pub id: String,
}

#[command]
pub fn list_workspaces_cmd(project_root: String) -> Result<Vec<WorkspaceEntry>, String> {
    let ws_dir = std::path::PathBuf::from(&project_root)
        .join("agents")
        .join("workspaces");

    if !ws_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    let read_dir = fs::read_dir(&ws_dir).map_err(|e| format!("Failed to read workspaces: {}", e))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            let yaml_path = path.join("workspace.yml");
            let id = if yaml_path.exists() {
                let content = fs::read_to_string(&yaml_path).unwrap_or_default();
                content
                    .lines()
                    .find(|l| l.starts_with("id:"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy().to_string())
            } else {
                path.file_name().unwrap().to_string_lossy().to_string()
            };
            entries.push(WorkspaceEntry {
                dir: path.file_name().unwrap().to_string_lossy().to_string(),
                id,
            });
        }
    }

    Ok(entries)
}

// ---------------------------------------------------------------------------
// v1.7.0: Patch Application Governance
//
// Typed DTOs — no JSON string arguments.
// Server-side agent policy enforced; frontend may narrow but not widen.
// Accepted is a UI state; audit records capture durable outcomes.
// ---------------------------------------------------------------------------

/// Apply an agent patch with full governance.
/// Validated, atomic, audited. No auto-commit.
#[command]
pub fn apply_agent_patch_cmd(
    project_root: String,
    patch: PatchProposal,
    allowed_paths: Vec<String>,
    agent_type: AgentType,
) -> Result<PatchApplicationResult, String> {
    let root = std::path::PathBuf::from(&project_root);
    if !root.exists() {
        return Err("Project root does not exist".into());
    }
    Ok(apply_agent_patch(&root, &patch, &allowed_paths, &agent_type))
}

/// Record a patch rejection without modifying any file.
#[command]
pub fn reject_agent_patch_cmd(
    project_root: String,
    patch: PatchProposal,
    agent_type: AgentType,
    reason: String,
) -> Result<PatchApplicationResult, String> {
    let root = std::path::PathBuf::from(&project_root);
    if !root.exists() {
        return Err("Project root does not exist".into());
    }
    Ok(reject_agent_patch(&root, &patch, &agent_type, &reason))
}

// ---------------------------------------------------------------------------
// v1.9.0: Architect Agent — Read-Only Planner
//
// Calls /agent/architect endpoint. Never writes files.
// ArchitectPlanResult has no patch-shaped fields.
// ---------------------------------------------------------------------------

/// Run the architect agent — read-only planning.
/// Builds context (Agent Context v2 + symbols + risks + ADRs), calls AI service.
/// Returns a structured plan. Never mutates the repository.
#[command]
pub fn run_architect_agent_cmd(
    project_root: String,
    task: String,
) -> Result<String, String> {
    let root = std::path::PathBuf::from(&project_root);
    if !root.exists() {
        return Err("Project root does not exist".into());
    }

    let task_obj: serde_json::Value = serde_json::from_str(&task)
        .map_err(|e| format!("Invalid task JSON: {e}"))?;

    // Build Agent Context v2 (content-free, schema-versioned)
    let (safe_context, git_context_status, git_context_error_code) =
        match git_bridge::build_agent_context_v2(&root) {
            Ok(ctx) => (
                serde_json::to_value(&ctx).unwrap_or(serde_json::json!({})),
                "available".to_string(),
                serde_json::Value::Null,
            ),
            Err(_) => (
                serde_json::json!({}),
                "unavailable".to_string(),
                serde_json::json!("AGENT_CONTEXT_BUILD_FAILED"),
            ),
        };

    // Extract symbol summaries for changed files
    let symbol_summaries = _extract_symbols_for_context(&root, &safe_context);

    // Find existing ADRs (bounded metadata only)
    let existing_adrs = _find_existing_adrs(&root);

    // Build constraints
    let constraints = serde_json::json!({
        "read_only": true,
        "may_edit_files": false,
        "may_create_adrs": false,
        "max_plan_sections": 8
    });

    let request_body = serde_json::json!({
        "task": task_obj,
        "git_context": safe_context,
        "git_context_status": git_context_status,
        "symbol_summaries": symbol_summaries,
        "risk_summary": null,
        "existing_adrs": existing_adrs,
        "constraints": constraints
    });

    // Call AI service — raises on failure, no runtime mock
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("http://localhost:8000/agent/architect")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(45))
        .send()
        .map_err(|e| format!("AI service unreachable: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("AI service error {status}: {body}"));
    }

    let result: serde_json::Value = response
        .json()
        .map_err(|e| format!("Invalid AI response: {e}"))?;

    serde_json::to_string(&result)
        .map_err(|e| format!("Failed to serialize architect result: {e}"))
}

/// Extract symbol summaries for changed files in the git context.
fn _extract_symbols_for_context(
    project_root: &std::path::PathBuf,
    git_context: &serde_json::Value,
) -> Vec<serde_json::Value> {
    let changed_files = git_context
        .get("changed_files")
        .and_then(|f| f.as_array())
        .cloned()
        .unwrap_or_default();

    let mut summaries = Vec::new();
    for file in changed_files.iter().take(20) {
        let path = file.get("path").and_then(|p| p.as_str()).unwrap_or("");
        let risk = file.get("risk").and_then(|r| r.as_str()).unwrap_or("normal");

        // Only extract for normal-risk supported files
        if risk != "normal" {
            continue;
        }

        let full_path = project_root.join(path);
        if let Ok(source) = std::fs::read_to_string(&full_path) {
            let summary = code_intel::extract_symbols(path, &source);
            if matches!(summary.parse_status, code_intel::ParseStatus::Success) {
                summaries.push(serde_json::to_value(&summary).unwrap_or_default());
            }
        }
    }
    summaries
}

/// Find existing ADRs with bounded metadata.
fn _find_existing_adrs(project_root: &std::path::PathBuf) -> Vec<serde_json::Value> {
    let mut adrs = Vec::new();
    let roadmap_dir = project_root.join("roadmap");
    if !roadmap_dir.exists() {
        return adrs;
    }

    if let Ok(entries) = std::fs::read_dir(&roadmap_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if name.starts_with("ADR-") {
                    // Read only first 500 bytes for metadata
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let title = content
                            .lines()
                            .find(|l| l.starts_with("title:"))
                            .map(|l| l.splitn(2, ':').nth(1).unwrap_or("").trim().to_string())
                            .unwrap_or_default();
                        let status = content
                            .lines()
                            .find(|l| l.starts_with("status:"))
                            .map(|l| l.splitn(2, ':').nth(1).unwrap_or("").trim().to_string())
                            .unwrap_or_default();
                        let excerpt: String = content
                            .lines()
                            .skip_while(|l| !l.starts_with("---") || l.starts_with("---"))
                            .skip(1)
                            .take(5)
                            .collect::<Vec<_>>()
                            .join(" ");
                        adrs.push(serde_json::json!({
                            "path": format!("roadmap/{}", name),
                            "title": title,
                            "status": status,
                            "excerpt": excerpt.chars().take(500).collect::<String>()
                        }));
                    }
                }
            }
            if adrs.len() >= 10 {
                break;
            }
        }
    }
    adrs
}

// ---------------------------------------------------------------------------
// v2.3.0: Hunk-to-Symbol Impact Mapping
// ---------------------------------------------------------------------------

/// Bounded symbol impact summary for architect agent context.
/// Max 20 individual impacts; counts only beyond that.
#[derive(Debug, serde::Serialize)]
pub struct ArchitectSymbolSummary {
    pub total_hunks: usize,
    pub impacted_symbols: usize,
    pub by_overlap_type: std::collections::HashMap<String, usize>,
    pub high_confidence_count: usize,
    pub truncated: bool,
}

/// v2.4.0: Bounded operational risk summary for architect agent context.
#[derive(Debug, serde::Serialize)]
pub struct ArchitectRiskSummary {
    pub total_risks: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub by_category: std::collections::HashMap<String, usize>,
    pub files_requiring_review: usize,
}

// ---------------------------------------------------------------------------
// v2.6.0: Autonomy Commands
//
// Auto-apply: Rust loads persisted settings. Frontend never authoritative.
// RequiresReview never writes files.
// ---------------------------------------------------------------------------

/// Enable or update autonomy settings.
/// Server-side validation ensures policy is safe.
/// Records when and by whom autonomy was enabled.
#[command]
pub fn set_autonomy_settings_cmd(
    app: tauri::AppHandle,
    update: AutonomySettingsUpdate,
    manager: tauri::State<'_, OnboardingStateManager>,
) -> Result<AutonomySettings, String> {
    let settings = manager.update(&app, |state| {
        if let Some(enabled) = update.enabled {
            if enabled {
                // Validate before enabling
                if let Err(e) = validate_autonomy_enable(&state.autonomy) {
                    // Don't enable — return current settings unchanged
                    return;
                }
                state.autonomy.enabled = true;
                state.autonomy.enabled_at = Some(chrono::Utc::now().to_rfc3339());
                state.autonomy.enabled_by = Some("local-user".to_string());
                state.autonomy.policy_version = crate::commands::onboarding_types::AUTONOMY_POLICY_VERSION;
            } else {
                state.autonomy.enabled = false;
                state.autonomy.enabled_at = None;
                state.autonomy.enabled_by = None;
            }
        }
    }).map_err(|e| format!("Failed to update autonomy settings: {:?}", e))?;

    Ok(settings.autonomy)
}

/// Get current autonomy settings (read-only).
#[command]
pub fn get_autonomy_settings_cmd(
    app: tauri::AppHandle,
    manager: tauri::State<'_, OnboardingStateManager>,
) -> Result<AutonomySettings, String> {
    let state = manager.get_or_load(&app);
    Ok(state.autonomy)
}

/// Auto-apply a patch if all gates pass.
/// Rust loads persisted settings — frontend never supplies policy.
/// RequiresReview never writes files, records audit only.
/// Auto-apply never commits.
#[command]
pub fn auto_apply_patch_cmd(
    app: tauri::AppHandle,
    project_root: String,
    patch: PatchProposal,
    confidence: f64,
    manager: tauri::State<'_, OnboardingStateManager>,
) -> Result<AutoApplyResult, String> {
    use crate::security::patch_guard::AgentType;
    use std::path::Path;

    // Load persisted settings — frontend is NOT authoritative
    let state = manager.get_or_load(&app);
    let settings = &state.autonomy;

    // Get current file checksum (server-side)
    let full_path = Path::new(&project_root).join(&patch.path);
    let current_checksum = if full_path.exists() {
        use sha2::{Sha256, Digest};
        if let Ok(content) = std::fs::read_to_string(&full_path) {
            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            Some(format!("{:x}", hasher.finalize()))
        } else {
            None
        }
    } else {
        None
    };

    // Decide
    let decision = decide_auto_apply(
        settings,
        &AgentType::Docs,
        &patch,
        confidence,
        current_checksum.as_deref(),
    );

    let path_class = crate::security::auto_apply::classify_path_for_audit(&patch.path);

    match &decision {
        AutoApplyDecision::Allowed => {
            // Increment session counter
            let prev = session_auto_apply_count();

            // Apply through existing PatchApplicationGuard
            let root = Path::new(&project_root);
            let result = apply_agent_patch(
                root,
                &patch,
                &settings.docs_safe_paths,
                &AgentType::Docs,
            );

            if result.status == crate::security::patch_guard::PatchStatus::Applied {
                // Increment counter only on success
                crate::security::auto_apply::increment_session_counter()
                    
            }

            let audit = build_auto_apply_audit(
                &patch.proposal_id,
                &patch,
                &decision,
                settings.policy_version,
            );

            Ok(AutoApplyResult {
                proposal_id: patch.proposal_id.clone(),
                decision: AutoApplyDecision::Allowed,
                path_class,
                applied: result.status == crate::security::patch_guard::PatchStatus::Applied,
                auto_commit: false,
                reason_codes: vec![],
                before_checksum: patch.before_checksum.clone(),
                after_checksum: result.after_checksum,
            })
        }
        AutoApplyDecision::Rejected(reason) => {
            let audit = build_auto_apply_audit(
                &patch.proposal_id,
                &patch,
                &decision,
                settings.policy_version,
            );

            Ok(AutoApplyResult {
                proposal_id: patch.proposal_id.clone(),
                decision: decision.clone(),
                path_class,
                applied: false,
                auto_commit: false,
                reason_codes: vec![format!("{:?}", reason).to_lowercase().replace('_', "-")],
                before_checksum: patch.before_checksum.clone(),
                after_checksum: None,
            })
        }
        AutoApplyDecision::RequiresReview => {
            // RequiresReview never writes files, records audit only
            let audit = build_auto_apply_audit(
                &patch.proposal_id,
                &patch,
                &decision,
                settings.policy_version,
            );

            Ok(AutoApplyResult {
                proposal_id: patch.proposal_id.clone(),
                decision: AutoApplyDecision::RequiresReview,
                path_class,
                applied: false,
                auto_commit: false,
                reason_codes: vec!["session-cap-exceeded".to_string()],
                before_checksum: patch.before_checksum.clone(),
                after_checksum: None,
            })
        }
    }
}
