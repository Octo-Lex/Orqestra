use crate::security::patch_guard::{
    AgentType, PatchProposal, PatchApplicationResult,
    apply_agent_patch, reject_agent_patch,
};
use std::fs;
use tauri::command;

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
