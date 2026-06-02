//! Tests for roadmap.rs Tauri commands.
//!
//! Tests the update_task_status_cmd frontmatter rewriting logic
//! and index_roadmap_cmd integration with md-indexer.

use std::fs;
use tempfile::TempDir;

/// Create a minimal roadmap task file with frontmatter.
fn create_task_file(dir: &std::path::Path, filename: &str, id: &str, status: &str) -> std::path::PathBuf {
    let roadmap_dir = dir.join("roadmap");
    fs::create_dir_all(&roadmap_dir).unwrap();
    let file_path = roadmap_dir.join(filename);
    let content = format!(
        "---\npm-task: true\nid: {}\ntitle: Test Task\nstatus: {}\npriority: High\ncreated: \"2026-01-01T00:00:00Z\"\nupdated: \"2026-01-01T00:00:00Z\"\n---\n\n# Test Task\n\nSome body text.\n",
        id, status
    );
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Simulate update_task_status_cmd logic (pure function, no Tauri runtime).
fn update_task_status(
    project_root: &str,
    task_id: &str,
    new_status: &str,
) -> Result<String, String> {
    let roadmap_dir = std::path::PathBuf::from(project_root).join("roadmap");
    let result = md_indexer::index_roadmap(&roadmap_dir).map_err(|e| e.to_string())?;

    let task = result
        .tasks
        .into_iter()
        .find(|t| t.frontmatter.id == task_id)
        .ok_or_else(|| format!("Task {} not found", task_id))?;

    let file_path = std::path::PathBuf::from(project_root).join(&task.source_path);
    let content = fs::read_to_string(&file_path).map_err(|e| format!("Read error: {}", e))?;

    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut in_frontmatter = false;
    let mut found = false;

    for line in lines.iter_mut() {
        if line.trim() == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter && line.starts_with("status:") {
            *line = format!("status: {}", new_status);
            found = true;
            break;
        }
    }

    if !found {
        return Err(format!("No status field in frontmatter for {}", task_id));
    }

    let new_content = if lines.join("\n").ends_with('\n') {
        lines.join("\n")
    } else {
        format!("{}\n", lines.join("\n"))
    };

    // Atomic write
    let tmp_path = file_path.with_extension("md.tmp");
    fs::write(&tmp_path, &new_content).map_err(|e| format!("Write error: {}", e))?;
    fs::rename(&tmp_path, &file_path).map_err(|e| format!("Rename error: {}", e))?;

    Ok(new_status.to_string())
}

// ---------------------------------------------------------------------------
// index_roadmap_cmd
// ---------------------------------------------------------------------------

#[test]
fn test_index_roadmap_finds_task() {
    let dir = TempDir::new().unwrap();
    create_task_file(dir.path(), "TASK-2026-100.md", "TASK-2026-100", "backlog");

    let roadmap_dir = dir.path().join("roadmap");
    let result = md_indexer::index_roadmap(&roadmap_dir).unwrap();
    assert_eq!(result.tasks.len(), 1);
    assert_eq!(result.tasks[0].frontmatter.id, "TASK-2026-100");
    assert_eq!(result.tasks[0].frontmatter.status, md_indexer::types::TaskStatus::Backlog);
}

#[test]
fn test_index_roadmap_empty_dir() {
    let dir = TempDir::new().unwrap();
    let roadmap_dir = dir.path().join("roadmap");
    fs::create_dir_all(&roadmap_dir).unwrap();

    let result = md_indexer::index_roadmap(&roadmap_dir).unwrap();
    assert_eq!(result.tasks.len(), 0);
}

// ---------------------------------------------------------------------------
// update_task_status_cmd
// ---------------------------------------------------------------------------

#[test]
fn test_update_task_status_todo_to_done() {
    let dir = TempDir::new().unwrap();
    create_task_file(dir.path(), "TASK-2026-100.md", "TASK-2026-100", "backlog");

    let result = update_task_status(
        dir.path().to_string_lossy().as_ref(),
        "TASK-2026-100",
        "done",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "done");

    // Verify file was actually updated
    let content = fs::read_to_string(dir.path().join("roadmap/TASK-2026-100.md")).unwrap();
    assert!(content.contains("status: done"));
    assert!(!content.contains("status: backlog"));
    // Other frontmatter preserved
    assert!(content.contains("priority: High"));
    // Body preserved
    assert!(content.contains("Some body text."));
}

#[test]
fn test_update_task_status_preserves_trailing_newline() {
    let dir = TempDir::new().unwrap();
    create_task_file(dir.path(), "TASK-2026-101.md", "TASK-2026-101", "in-progress");

    update_task_status(
        dir.path().to_string_lossy().as_ref(),
        "TASK-2026-101",
        "done",
    )
    .unwrap();

    let content = fs::read_to_string(dir.path().join("roadmap/TASK-2026-101.md")).unwrap();
    assert!(content.ends_with('\n'));
}

#[test]
fn test_update_task_status_not_found() {
    let dir = TempDir::new().unwrap();
    // Empty roadmap directory
    fs::create_dir_all(dir.path().join("roadmap")).unwrap();

    let result = update_task_status(
        dir.path().to_string_lossy().as_ref(),
        "TASK-999",
        "done",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_update_task_status_idempotent() {
    let dir = TempDir::new().unwrap();
    create_task_file(dir.path(), "TASK-2026-102.md", "TASK-2026-102", "done");

    // Set to same status
    update_task_status(
        dir.path().to_string_lossy().as_ref(),
        "TASK-2026-102",
        "done",
    )
    .unwrap();

    let content = fs::read_to_string(dir.path().join("roadmap/TASK-2026-102.md")).unwrap();
    assert!(content.contains("status: done"));
}

// ---------------------------------------------------------------------------
// get_task
// ---------------------------------------------------------------------------

#[test]
fn test_get_task_found() {
    let dir = TempDir::new().unwrap();
    create_task_file(dir.path(), "TASK-2026-100.md", "TASK-2026-100", "backlog");

    let roadmap_dir = dir.path().join("roadmap");
    let result = md_indexer::index_roadmap(&roadmap_dir).unwrap();
    let task = result.tasks.into_iter().find(|t| t.frontmatter.id == "TASK-2026-100");
    assert!(task.is_some());
    assert_eq!(task.unwrap().frontmatter.title, "Test Task");
}

#[test]
fn test_get_task_not_found() {
    let dir = TempDir::new().unwrap();
    create_task_file(dir.path(), "TASK-2026-100.md", "TASK-2026-100", "backlog");

    let roadmap_dir = dir.path().join("roadmap");
    let result = md_indexer::index_roadmap(&roadmap_dir).unwrap();
    let task = result.tasks.into_iter().find(|t| t.frontmatter.id == "TASK-999");
    assert!(task.is_none());
}
