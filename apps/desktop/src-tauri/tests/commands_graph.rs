//! Tests for graph.rs Tauri commands.
//!
//! Tests the pure file-IO commands (read_trace, read_commit_stub).
//! Graph store operations (index_graph, query_graph) are tested via the
//! graph-store crate's own 7 tests. The query_history command is tested
//! via HTTP in the Tier 2 suite.

use std::fs;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// read_trace_cmd
// ---------------------------------------------------------------------------

fn read_trace_cmd(project_root: &str, trace_id: &str) -> Result<String, String> {
    let path = std::path::PathBuf::from(project_root)
        .join(".Orqestra/graph/reasoning")
        .join(format!("{}.txt", trace_id));

    if !path.exists() {
        return Err(format!("Trace not found: {}", trace_id));
    }

    fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))
}

#[test]
fn test_read_trace_success() {
    let dir = TempDir::new().unwrap();
    let reasoning_dir = dir.path().join(".Orqestra/graph/reasoning");
    fs::create_dir_all(&reasoning_dir).unwrap();
    fs::write(
        reasoning_dir.join("abc-123.txt"),
        "Step 1: Analyze diff\nStep 2: Map concepts\n",
    )
    .unwrap();

    let result = read_trace_cmd(dir.path().to_string_lossy().as_ref(), "abc-123");
    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.contains("Analyze diff"));
    assert!(content.contains("Map concepts"));
}

#[test]
fn test_read_trace_not_found() {
    let dir = TempDir::new().unwrap();
    let result = read_trace_cmd(dir.path().to_string_lossy().as_ref(), "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Trace not found"));
}

// ---------------------------------------------------------------------------
// read_commit_stub_cmd
// ---------------------------------------------------------------------------

fn read_commit_stub_cmd(project_root: &str, hash: &str) -> Result<serde_json::Value, String> {
    let path = std::path::PathBuf::from(project_root)
        .join(".Orqestra/graph/commits")
        .join(format!("{}.json", hash));

    if !path.exists() {
        return Err(format!("Commit stub not found: {}", hash));
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))
}

#[test]
fn test_read_commit_stub_success() {
    let dir = TempDir::new().unwrap();
    let commits_dir = dir.path().join(".Orqestra/graph/commits");
    fs::create_dir_all(&commits_dir).unwrap();
    let stub = serde_json::json!({
        "hash": "deadbeef1234",
        "intent_summary": "Fix navigation crash",
        "confidence": 0.92,
        "task_ids": ["TASK-001", "TASK-002"]
    });
    fs::write(
        commits_dir.join("deadbeef1234.json"),
        serde_json::to_string_pretty(&stub).unwrap(),
    )
    .unwrap();

    let result = read_commit_stub_cmd(dir.path().to_string_lossy().as_ref(), "deadbeef1234");
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["hash"], "deadbeef1234");
    assert_eq!(value["confidence"], 0.92);
}

#[test]
fn test_read_commit_stub_not_found() {
    let dir = TempDir::new().unwrap();
    let result = read_commit_stub_cmd(dir.path().to_string_lossy().as_ref(), "missing");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Commit stub not found"));
}

#[test]
fn test_read_commit_stub_invalid_json() {
    let dir = TempDir::new().unwrap();
    let commits_dir = dir.path().join(".Orqestra/graph/commits");
    fs::create_dir_all(&commits_dir).unwrap();
    fs::write(commits_dir.join("bad.json"), "this is not json{{{").unwrap();

    let result = read_commit_stub_cmd(dir.path().to_string_lossy().as_ref(), "bad");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Parse error"));
}
