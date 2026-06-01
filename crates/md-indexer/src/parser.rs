//! Parser for task `.md` files.
//!
//! Extracts YAML frontmatter between `---` delimiters and parses the Markdown
//! body into structured sections (context, acceptance criteria, agent notes).

use std::path::Path;

use crate::error::IndexerError;
use crate::types::{AcceptanceCriterion, Task, TaskBody, TaskFrontmatter};

/// Parse a complete task `.md` file into a [`Task`].
pub fn parse_task_content(raw: &str, source_path: &Path) -> Result<Task, IndexerError> {
    let (frontmatter_yaml, body_md) = split_frontmatter(raw, source_path)?;

    let frontmatter: TaskFrontmatter = serde_yaml::from_str(frontmatter_yaml).map_err(|e| {
        IndexerError::InvalidFrontmatter(
            source_path.to_path_buf(),
            format!("{}", e),
        )
    })?;

    // Skip files that aren't task files
    if !frontmatter.pm_task {
        return Err(IndexerError::ParseError(
            source_path.to_path_buf(),
            "not a task file (pm-task is not true)".into(),
        ));
    }

    let body = parse_body(body_md);

    Ok(Task {
        frontmatter,
        body,
        source_path: source_path.to_path_buf(),
    })
}

/// Split a file into `(frontmatter_yaml, body_markdown)`.
///
/// Expects the file to start with `---\n` and have a closing `---\n`.
fn split_frontmatter<'a>(
    content: &'a str,
    path: &Path,
) -> Result<(&'a str, &'a str), IndexerError> {
    // Must start with "---\n"
    if !content.starts_with("---") {
        return Err(IndexerError::ParseError(
            path.to_path_buf(),
            "file does not start with --- frontmatter delimiter".into(),
        ));
    }

    // Find the closing "---"
    // Skip the opening "---" and optional newline, then search for the next "---"
    let after_opening = &content[3..];
    let newline_after_opening = after_opening
        .find(|c: char| c != '\n' && c != '\r')
        .unwrap_or(0);

    let search_start = 3 + newline_after_opening;

    // Look for "\n---" to find the closing delimiter
    let closing = content[search_start..]
        .find("\n---")
        .map(|pos| search_start + pos)
        .or_else(|| {
            // Handle case where closing --- is at end without trailing newline
            content[search_start..]
                .find("---")
                .map(|pos| search_start + pos)
                .filter(|&pos| pos > 3) // Must be after opening
        });

    let closing = closing.ok_or_else(|| {
        IndexerError::ParseError(
            path.to_path_buf(),
            "no closing --- delimiter found".into(),
        )
    })?;

    let yaml_content = &content[3..closing].trim_start_matches('\n').trim_start_matches('\r');
    let body_start = content[closing..]
        .find('\n')
        .map(|offset| closing + offset + 1)
        .unwrap_or(content.len());

    let body = &content[body_start..];

    Ok((yaml_content, body))
}

/// Parse the Markdown body into structured sections.
fn parse_body(md: &str) -> TaskBody {
    let mut acceptance_criteria: Vec<AcceptanceCriterion> = Vec::new();

    let mut current_section: Option<&str> = None;
    let mut context_lines: Vec<&str> = Vec::new();
    let mut note_lines: Vec<&str> = Vec::new();

    for line in md.lines() {
        let trimmed = line.trim();

        // Detect section headings
        if let Some(heading) = trimmed.strip_prefix("## ") {
            match heading.trim() {
                "Context" => {
                    current_section = Some("context");
                    continue;
                }
                "Acceptance Criteria" => {
                    current_section = Some("criteria");
                    continue;
                }
                "Agent Notes" => {
                    current_section = Some("notes");
                    continue;
                }
                other => {
                    // Unknown heading — treat as a new generic section
                    current_section = Some(other);
                    continue;
                }
            }
        }

        match current_section {
            Some("context") => {
                context_lines.push(line);
            }
            Some("criteria") => {
                // Parse checkbox lines: "- [ ] text" or "- [x] text"
                if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
                    acceptance_criteria.push(AcceptanceCriterion {
                        text: rest.to_string(),
                        completed: false,
                    });
                } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
                    acceptance_criteria.push(AcceptanceCriterion {
                        text: rest.to_string(),
                        completed: true,
                    });
                }
                // Ignore non-checkbox lines in criteria section
            }
            Some("notes") => {
                note_lines.push(line);
            }
            _ => {
                // Content before any heading goes into context
                if !trimmed.is_empty() {
                    context_lines.push(line);
                    current_section = Some("context");
                }
            }
        }
    }

    // Assemble context
    let context = {
        let text = context_lines.join("\n").trim().to_string();
        if text.is_empty() { None } else { Some(text) }
    };

    // Assemble agent notes
    let agent_notes = {
        let text = note_lines.join("\n").trim().to_string();
        if text.is_empty() { None } else { Some(text) }
    };

    TaskBody {
        context,
        acceptance_criteria,
        agent_notes,
        raw: md.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// The exact fixture from Section 3.2 of the spec.
    const TASK_2026_042: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/roadmap/TASK-2026-042.md"
    ));

    #[test]
    fn parse_task_2026_042() {
        let task = parse_task_content(TASK_2026_042, Path::new("roadmap/TASK-2026-042.md"))
            .expect("parse should succeed");
        let fm = &task.frontmatter;

        assert_eq!(fm.id, "TASK-2026-042");
        assert_eq!(fm.title, "Refactor auth middleware to use JWT");
        assert_eq!(fm.status, crate::types::TaskStatus::InProgress);
        assert_eq!(fm.priority, crate::types::Priority::Critical);
        assert_eq!(fm.sprint.as_deref(), Some("Sprint 14"));
        assert_eq!(fm.epic.as_deref(), Some("Security Hardening"));
        assert_eq!(fm.assignee.as_deref(), Some("agent-architect"));
        assert_eq!(fm.progress, 37);

        // Duration parsing
        assert_eq!(fm.time_estimate.map(|d| d.minutes()), Some(480));
        assert_eq!(fm.time_logged.map(|d| d.minutes()), Some(180));

        // Dependencies & blocks
        assert_eq!(fm.dependencies, vec!["TASK-2026-038", "TASK-2026-040"]);
        assert_eq!(fm.blocks, vec!["TASK-2026-045"]);
        assert_eq!(fm.labels, vec!["backend", "auth", "refactor"]);

        // Dates
        assert_eq!(fm.due_date.as_deref(), Some("2026-06-15"));
        assert_eq!(fm.start_date.as_deref(), Some("2026-06-01"));
        assert_eq!(fm.created, "2026-05-28T09:00:00Z");
        assert_eq!(fm.updated, "2026-06-01T14:30:00Z");

        // Body
        assert!(task.body.context.is_some());
        assert_eq!(task.body.acceptance_criteria.len(), 3);
        assert_eq!(
            task.body.acceptance_criteria[0].text,
            "All routes protected by JWT middleware"
        );
        assert!(!task.body.acceptance_criteria[0].completed);
        assert!(task.body.agent_notes.is_some());
    }

    /// E.6 serialization contract test — mandatory before Tauri UI.
    #[test]
    fn serializes_to_expected_json_shape() {
        let task = parse_task_content(TASK_2026_042, Path::new("test.md")).unwrap();
        let json = serde_json::to_value(&task).unwrap();

        assert_eq!(json["frontmatter"]["id"], "TASK-2026-042");
        assert_eq!(json["frontmatter"]["status"], "in-progress");
        assert_eq!(json["frontmatter"]["time_estimate"], 480);
        assert!(json["frontmatter"]["dependencies"].is_array());

        // Verify JSON field names match TS contract (snake_case)
        assert!(json["frontmatter"].get("time_estimate").is_some());
        assert!(json["frontmatter"].get("due_date").is_some());
        assert!(json["frontmatter"].get("start_date").is_some());

        // Body shape
        assert!(json["body"]["context"].is_string());
        assert!(json["body"]["acceptance_criteria"].is_array());
        assert!(json["body"]["agent_notes"].is_string());
        assert!(json["body"]["raw"].is_string());
        assert!(json["source_path"].is_string());
    }

    #[test]
    fn parse_minimal_task() {
        let minimal = r#"---
pm-task: true
id: TASK-001
title: "Do the thing"
status: backlog
priority: Low
progress: 0
created: "2026-01-01T00:00:00Z"
updated: "2026-01-01T00:00:00Z"
---
Some context here.
"#;
        let task = parse_task_content(minimal, Path::new("roadmap/TASK-001.md")).unwrap();
        assert_eq!(task.frontmatter.id, "TASK-001");
        assert_eq!(task.frontmatter.status, crate::types::TaskStatus::Backlog);
        assert!(task.frontmatter.sprint.is_none());
        assert!(task.frontmatter.time_estimate.is_none());
        assert_eq!(task.body.acceptance_criteria.len(), 0);
    }

    #[test]
    fn missing_frontmatter_delimiter_returns_error() {
        let no_delimiter = "Just some markdown\nno frontmatter here";
        let result = parse_task_content(no_delimiter, Path::new("bad.md"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            IndexerError::ParseError(path, msg) => {
                assert_eq!(path, PathBuf::from("bad.md"));
                assert!(msg.contains("---"));
            }
            other => panic!("expected ParseError, got {:?}", other),
        }
    }

    #[test]
    fn no_closing_delimiter_returns_error() {
        let no_close = "---\nfoo: bar\nno closing delimiter";
        let result = parse_task_content(no_close, Path::new("bad.md"));
        assert!(result.is_err());
    }

    #[test]
    fn non_task_file_returns_error() {
        let not_task = "---\npm-epic: true\nid: epic-1\n---\nContent";
        let result = parse_task_content(not_task, Path::new("epic.md"));
        assert!(result.is_err());
        // serde_yaml fails on missing required field `pm-task` before we reach the pm_task check
        match result.unwrap_err() {
            IndexerError::InvalidFrontmatter(_, msg) => {
                assert!(msg.contains("pm-task"));
            }
            other => panic!("expected InvalidFrontmatter, got {:?}", other),
        }
    }

    #[test]
    fn acceptance_criteria_with_completed_items() {
        let md = r#"---
pm-task: true
id: T-1
title: "Test"
status: done
priority: Low
progress: 100
created: "2026-01-01T00:00:00Z"
updated: "2026-01-01T00:00:00Z"
---
## Acceptance Criteria
- [x] First thing done
- [ ] Second thing pending
- [x] Third thing done
"#;
        let task = parse_task_content(md, Path::new("test.md")).unwrap();
        assert_eq!(task.body.acceptance_criteria.len(), 3);
        assert!(task.body.acceptance_criteria[0].completed);
        assert!(!task.body.acceptance_criteria[1].completed);
        assert!(task.body.acceptance_criteria[2].completed);
    }
}
