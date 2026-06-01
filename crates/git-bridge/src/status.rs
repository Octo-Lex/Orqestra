use crate::error::GitBridgeError;
use md_indexer::TaskStatus;
use std::path::Path;

/// Update the `status:` field in a task's YAML frontmatter.
/// Uses string replacement — does not re-parse and re-serialize the whole file,
/// which would lose comments and formatting.
pub fn update_task_status(
    task_path: &Path,
    new_status: TaskStatus,
) -> Result<(), GitBridgeError> {
    let content = std::fs::read_to_string(task_path)
        .map_err(|e| GitBridgeError::Io(task_path.to_owned(), e))?;

    let status_str = status_to_yaml_str(&new_status);
    let updated = replace_status_line(&content, status_str).ok_or_else(|| {
        GitBridgeError::YamlParse(format!("No status: line found in {:?}", task_path))
    })?;

    // Write atomically
    let tmp = task_path.with_extension("md.tmp");
    std::fs::write(&tmp, &updated)
        .map_err(|e| GitBridgeError::Io(tmp.clone(), e))?;
    std::fs::rename(&tmp, task_path)
        .map_err(|e| GitBridgeError::Io(task_path.to_owned(), e))?;

    Ok(())
}

fn status_to_yaml_str(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Backlog => "backlog",
        TaskStatus::Ready => "ready",
        TaskStatus::InProgress => "in-progress",
        TaskStatus::InReview => "in-review",
        TaskStatus::Done => "done",
        TaskStatus::Cancelled => "cancelled",
    }
}

fn replace_status_line(content: &str, new_status: &str) -> Option<String> {
    let mut found = false;
    let lines: Vec<String> = content
        .lines()
        .map(|line| {
            if !found && line.trim_start().starts_with("status:") {
                found = true;
                format!("status: {new_status}")
            } else {
                line.to_string()
            }
        })
        .collect();

    if found {
        Some(lines.join("\n") + if content.ends_with('\n') { "\n" } else { "" })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn replaces_status_in_progress_to_done() {
        let content = "---\nstatus: in-progress\npriority: High\n---\nBody\n";
        let result = replace_status_line(content, "done").unwrap();
        assert!(result.contains("status: done"));
        assert!(!result.contains("in-progress"));
        assert!(result.contains("priority: High"));
    }

    #[test]
    fn preserves_rest_of_frontmatter() {
        let content = "---\nstatus: backlog\nid: TASK-001\ntitle: \"Test\"\nlabels:\n  - backend\n---\n";
        let result = replace_status_line(content, "ready").unwrap();
        assert!(result.contains("id: TASK-001"));
        assert!(result.contains("title: \"Test\""));
        assert!(result.contains("  - backend"));
        assert!(result.contains("status: ready"));
    }

    #[test]
    fn returns_none_when_no_status_line() {
        let content = "---\nid: TASK-001\n---\nBody\n";
        assert!(replace_status_line(content, "done").is_none());
    }

    #[test]
    fn preserves_trailing_newline() {
        let content_with = "---\nstatus: backlog\n---\nBody\n";
        let content_without = "---\nstatus: backlog\n---\nBody";
        assert!(replace_status_line(content_with, "done").unwrap().ends_with('\n'));
        assert!(!replace_status_line(content_without, "done").unwrap().ends_with('\n'));
    }

    #[test]
    fn update_task_status_writes_to_file() {
        let dir = std::env::temp_dir().join("git-bridge-test-status");
        std::fs::create_dir_all(&dir).unwrap();

        let task_path = dir.join("TASK-TEST.md");
        let mut f = std::fs::File::create(&task_path).unwrap();
        write!(
            f,
            "---\npm-task: true\nid: TASK-TEST\nstatus: backlog\npriority: Low\nprogress: 0\ncreated: \"2026-01-01T00:00:00Z\"\nupdated: \"2026-01-01T00:00:00Z\"\n---\nBody\n"
        )
        .unwrap();

        update_task_status(&task_path, TaskStatus::InProgress).unwrap();

        let updated = std::fs::read_to_string(&task_path).unwrap();
        assert!(updated.contains("status: in-progress"));
        assert!(!updated.contains("status: backlog"));
        assert!(updated.contains("id: TASK-TEST")); // rest preserved

        // No temp file left behind
        assert!(!task_path.with_extension("md.tmp").exists());

        std::fs::remove_dir_all(&dir).ok();
    }
}
