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
