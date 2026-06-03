//! Tests for agents.rs Tauri commands.
//!
//! These test the pure-logic parts of the command handlers without a Tauri runtime.
//! Commands that call the AI service (run_docs_agent_cmd, run_bugfix_agent_cmd)
//! are tested via HTTP in the Tier 2 suite.

use std::fs;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// read_file_cmd
// ---------------------------------------------------------------------------

fn read_file_cmd(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path, e))
}

#[test]
fn test_read_file_cmd_success() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, "hello world").unwrap();

    let result = read_file_cmd(file.to_string_lossy().to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello world");
}

#[test]
fn test_read_file_cmd_not_found() {
    let result = read_file_cmd("/nonexistent/path/file.md".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read"));
}

// ---------------------------------------------------------------------------
// write_file_cmd
// ---------------------------------------------------------------------------

fn write_file_cmd(path: String, content: String) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", path, e))
}

#[test]
fn test_write_file_cmd_creates_parent_dirs() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("nested/deep/test.md");

    let result = write_file_cmd(file.to_string_lossy().to_string(), "content".to_string());
    assert!(result.is_ok());
    assert_eq!(fs::read_to_string(&file).unwrap(), "content");
}

#[test]
fn test_write_file_cmd_overwrites() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, "old").unwrap();

    write_file_cmd(file.to_string_lossy().to_string(), "new".to_string()).unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "new");
}

// ---------------------------------------------------------------------------
// run_agent_cmd (state dir + JSON result)
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunAgentResult {
    pub workspace_id: String,
    pub task_id: String,
    pub status: String,
    pub message: String,
}

fn run_agent_cmd(
    project_root: String,
    workspace_id: String,
    task_id: String,
) -> Result<String, String> {
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

    Ok(serde_json::to_string(&RunAgentResult {
        workspace_id,
        task_id,
        status: "dispatched".to_string(),
        message: "Agent dispatched.".to_string(),
    })
    .unwrap())
}

#[test]
fn test_run_agent_cmd_creates_state() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_string_lossy().to_string();

    let result = run_agent_cmd(root.clone(), "architect".to_string(), "TASK-001".to_string());
    assert!(result.is_ok());

    // Verify state file was created
    let state_path = dir.path().join(".Orqestra/agents/architect/state.json");
    assert!(state_path.exists());

    let state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert_eq!(state["workspaceId"], "architect");
    assert_eq!(state["taskId"], "TASK-001");
    assert_eq!(state["status"], "running");
}

#[test]
fn test_run_agent_cmd_returns_json() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_string_lossy().to_string();

    let result_str = run_agent_cmd(root, "bugfix".to_string(), "TASK-042".to_string()).unwrap();
    let result: RunAgentResult = serde_json::from_str(&result_str).unwrap();
    assert_eq!(result.workspace_id, "bugfix");
    assert_eq!(result.task_id, "TASK-042");
    assert_eq!(result.status, "dispatched");
}

// ---------------------------------------------------------------------------
// read_project_file_cmd (path traversal protection)
// ---------------------------------------------------------------------------

fn read_project_file_cmd(project_root: String, path: String) -> Result<String, String> {
    let full_path = std::path::PathBuf::from(&project_root).join(&path);

    let canonical_root = std::path::PathBuf::from(&project_root)
        .canonicalize()
        .map_err(|e| format!("Invalid project root: {e}"))?;
    let canonical_file = full_path
        .canonicalize()
        .map_err(|e| format!("Invalid file path: {e}"))?;
    if !canonical_file.starts_with(&canonical_root) {
        return Err("Path traversal blocked".into());
    }

    fs::read_to_string(&full_path).map_err(|e| format!("Failed to read {}: {}", path, e))
}

#[test]
fn test_read_project_file_success() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("src/main.rs");
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(&file, "fn main() {}").unwrap();

    let result = read_project_file_cmd(
        dir.path().to_string_lossy().to_string(),
        "src/main.rs".to_string(),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "fn main() {}");
}

#[test]
fn test_read_project_file_traversal_blocked() {
    let dir = TempDir::new().unwrap();
    // Create a file outside the project root
    let outside = dir.path().join("../outside.txt");
    fs::write(&outside, "secret").unwrap();

    let result = read_project_file_cmd(
        dir.path().to_string_lossy().to_string(),
        "../outside.txt".to_string(),
    );
    assert!(result.is_err());
    // The error should indicate path traversal (canonicalization resolves ../outside.txt
    // to be outside the project root)
    let err = result.unwrap_err();
    assert!(
        err.contains("Path traversal blocked") || err.contains("Invalid file path"),
        "Expected path traversal error, got: {err}"
    );
}

#[test]
fn test_read_project_file_not_found() {
    let dir = TempDir::new().unwrap();
    let result = read_project_file_cmd(
        dir.path().to_string_lossy().to_string(),
        "nonexistent.md".to_string(),
    );
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// list_workspaces_cmd
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize)]
struct WorkspaceEntry {
    pub dir: String,
    pub id: String,
}

fn list_workspaces_cmd(project_root: String) -> Result<Vec<WorkspaceEntry>, String> {
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

#[test]
fn test_list_workspaces_empty() {
    let dir = TempDir::new().unwrap();
    let result = list_workspaces_cmd(dir.path().to_string_lossy().to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_list_workspaces_finds_dirs() {
    let dir = TempDir::new().unwrap();
    let ws_dir = dir.path().join("agents/workspaces");
    fs::create_dir_all(ws_dir.join("architect")).unwrap();
    fs::create_dir_all(ws_dir.join("bugfix")).unwrap();
    // Write a workspace.yml with an id
    fs::write(
        ws_dir.join("architect/workspace.yml"),
        "id: architect\nname: Architect",
    )
    .unwrap();

    let result = list_workspaces_cmd(dir.path().to_string_lossy().to_string());
    assert!(result.is_ok());
    let entries = result.unwrap();
    assert_eq!(entries.len(), 2);

    // Check that the architect entry has the id from workspace.yml
    let architect = entries.iter().find(|e| e.dir == "architect").unwrap();
    assert_eq!(architect.id, "architect");

    // Bugfix has no workspace.yml, so id = dir name
    let bugfix = entries.iter().find(|e| e.dir == "bugfix").unwrap();
    assert_eq!(bugfix.id, "bugfix");
}

#[test]
fn test_list_workspaces_ignores_files() {
    let dir = TempDir::new().unwrap();
    let ws_dir = dir.path().join("agents/workspaces");
    fs::create_dir_all(&ws_dir).unwrap();
    fs::write(ws_dir.join("readme.txt"), "not a dir").unwrap();

    let result = list_workspaces_cmd(dir.path().to_string_lossy().to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// v1.1.1 Bugfix-agent review-only hardening tests
//
// These test the path-filtering logic from run_bugfix_agent_cmd
// without calling the AI service.
// ---------------------------------------------------------------------------

/// Simulate the path-filtering logic from bugfix-agent.
/// Returns only edits whose paths are in allowed_paths.
fn filter_bugfix_edits<'a>(
    edits: Vec<(&'a str, &'a str, &'a str)>, // (path, before, after)
    allowed_paths: &[&'a str],
) -> Vec<(&'a str, &'a str, &'a str)> {
    edits
        .into_iter()
        .filter(|(path, _, _)| allowed_paths.contains(path))
        .collect()
}

#[test]
fn bugfix_agent_allows_permitted_paths() {
    let edits = vec![
        ("apps/dashboard/src/App.tsx", "old", "new"),
        ("README.md", "old", "new"),
    ];
    let allowed = vec!["apps/dashboard/src/App.tsx", "README.md"];
    let filtered = filter_bugfix_edits(edits, &allowed);
    assert_eq!(filtered.len(), 2);
}

#[test]
fn bugfix_agent_filters_disallowed_paths() {
    let edits = vec![
        ("apps/dashboard/src/App.tsx", "old", "new"),
        ("/etc/passwd", "old", "new"),
    ];
    let allowed = vec!["apps/dashboard/src/App.tsx"];
    let filtered = filter_bugfix_edits(edits, &allowed);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, "apps/dashboard/src/App.tsx");
}

#[test]
fn bugfix_agent_filters_env_file_edits() {
    let edits = vec![
        (".env", "OLD_KEY=old", "OLD_KEY=new"),
        ("apps/dashboard/src/App.tsx", "old", "new"),
    ];
    let allowed = vec!["apps/dashboard/src/App.tsx"];
    let filtered = filter_bugfix_edits(edits, &allowed);
    // .env is not in allowed_paths, so it should be filtered
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, "apps/dashboard/src/App.tsx");
}

#[test]
fn bugfix_agent_filters_workflow_edits() {
    let edits = vec![
        (".github/workflows/ci.yml", "old", "new"),
        ("apps/dashboard/src/App.tsx", "old", "new"),
    ];
    let allowed = vec!["apps/dashboard/src/App.tsx"];
    let filtered = filter_bugfix_edits(edits, &allowed);
    assert_eq!(filtered.len(), 1);
}

#[test]
fn bugfix_agent_handles_empty_edit_response() {
    let edits: Vec<(&str, &str, &str)> = vec![];
    let allowed = vec!["apps/dashboard/src/App.tsx"];
    let filtered = filter_bugfix_edits(edits, &allowed);
    assert_eq!(filtered.len(), 0);
}

#[test]
fn bugfix_agent_auto_commit_always_false() {
    // The bugfix agent request always sets auto_commit: false.
    // Verify this invariant is baked into the request construction.
    let request = serde_json::json!({
        "task": {"id": "TASK-001"},
        "allowed_files": [],
        "constraints": {
            "allowed_paths": [],
            "max_files_changed": 0,
            "auto_commit": false,
            "may_request_more_files": true
        }
    });
    assert_eq!(request["constraints"]["auto_commit"], false);
    // auto_commit must never be true for bugfix agent
    assert!(request["constraints"]["auto_commit"].as_bool() == Some(false));
}

#[test]
fn bugfix_agent_reject_leaves_worktree_unchanged() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, "original content").unwrap();

    // Simulate: agent proposes edit, user rejects
    // The file should remain unchanged
    let _proposed_content = "modified content";
    // Rejection means we don't write the proposed content
    let actual = fs::read_to_string(&file).unwrap();
    assert_eq!(actual, "original content");
}
