//! Domain types for the md-indexer.
//!
//! Field names and JSON shapes are pinned to the TypeScript contract in
//! Appendix E.3 of the Orqestra specification. Do not rename fields without
//! updating the TypeScript interfaces.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::duration::Duration;
use crate::error::IndexerError;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Backlog,
    Ready,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Task,
    Subtask,
    Milestone,
    Epic,
}

// ---------------------------------------------------------------------------
// Frontmatter
// ---------------------------------------------------------------------------

/// Parsed YAML frontmatter from a task `.md` file.
///
/// JSON shape must match `TaskFrontmatter` in `apps/desktop/src/lib/orqestra.ts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFrontmatter {
    /// Discriminator: `true` for task files.
    #[serde(rename = "pm-task")]
    pub pm_task: bool,

    pub id: String,
    pub title: String,

    /// Optional — omitted from the TypeScript contract but present in the spec.
    /// Parsed from YAML, not exposed to TS (handled at the Tauri command layer).
    #[serde(default, skip_serializing)]
    pub r#type: Option<TaskType>,

    pub status: TaskStatus,
    pub priority: Priority,

    #[serde(default)]
    pub sprint: Option<String>,

    #[serde(default)]
    pub epic: Option<String>,

    #[serde(default)]
    pub assignee: Option<String>,

    /// 0–100
    #[serde(default)]
    pub progress: u8,

    #[serde(default)]
    pub dependencies: Vec<String>,

    #[serde(default)]
    pub blocks: Vec<String>,

    #[serde(default)]
    pub labels: Vec<String>,

    /// Duration parsed from strings like `"8h"`, `"30m"`, `"1h30m"`, or bare integers.
    /// Serializes as `number | null` (integer minutes).
    #[serde(default)]
    pub time_estimate: Option<Duration>,

    #[serde(default)]
    pub time_logged: Option<Duration>,

    #[serde(default)]
    pub due_date: Option<String>,

    #[serde(default)]
    pub start_date: Option<String>,

    pub created: String,
    pub updated: String,
}

// ---------------------------------------------------------------------------
// Task body (parsed Markdown)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub text: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBody {
    pub context: Option<String>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub agent_notes: Option<String>,
    /// Raw Markdown body (everything after the closing `---`), unparsed.
    pub raw: String,
}

// ---------------------------------------------------------------------------
// Task (frontmatter + body + source location)
// ---------------------------------------------------------------------------

/// A fully parsed task file.
///
/// JSON shape must match `Task` in `apps/desktop/src/lib/orqestra.ts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub frontmatter: TaskFrontmatter,
    pub body: TaskBody,
    pub source_path: PathBuf,
}

// ---------------------------------------------------------------------------
// Index result
// ---------------------------------------------------------------------------

/// Result of indexing a `roadmap/` directory.
///
/// `errors` contains per-file parse failures so the caller can report warnings
/// without aborting the entire index.
#[derive(Debug, Clone)]
pub struct IndexResult {
    pub tasks: Vec<Task>,
    /// (file path, error description)
    pub errors: Vec<(PathBuf, IndexerError)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_roundtrip() {
        let statuses = [
            TaskStatus::Backlog,
            TaskStatus::Ready,
            TaskStatus::InProgress,
            TaskStatus::InReview,
            TaskStatus::Done,
            TaskStatus::Cancelled,
        ];
        for status in &statuses {
            let json = serde_json::to_string(status).unwrap();
            let back: TaskStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*status, back, "roundtrip failed for {:?}", status);
        }
    }

    #[test]
    fn status_serializes_kebab_case() {
        assert_eq!(
            serde_json::to_string(&TaskStatus::InProgress).unwrap(),
            r#""in-progress""#
        );
        assert_eq!(
            serde_json::to_string(&TaskStatus::InReview).unwrap(),
            r#""in-review""#
        );
    }

    #[test]
    fn priority_roundtrip() {
        let priorities = [
            Priority::Critical,
            Priority::High,
            Priority::Medium,
            Priority::Low,
        ];
        for p in &priorities {
            let json = serde_json::to_string(p).unwrap();
            let back: Priority = serde_json::from_str(&json).unwrap();
            assert_eq!(*p, back, "roundtrip failed for {:?}", p);
        }
    }

    #[test]
    fn priority_serializes_pascal_case() {
        // Spec uses PascalCase: "Critical", "High", "Medium", "Low"
        assert_eq!(
            serde_json::to_string(&Priority::Critical).unwrap(),
            r#""Critical""#
        );
        assert_eq!(
            serde_json::to_string(&Priority::High).unwrap(),
            r#""High""#
        );
    }

    #[test]
    fn task_type_roundtrip() {
        let types = [
            TaskType::Task,
            TaskType::Subtask,
            TaskType::Milestone,
            TaskType::Epic,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            let back: TaskType = serde_json::from_str(&json).unwrap();
            assert_eq!(*t, back, "roundtrip failed for {:?}", t);
        }
    }

    #[test]
    fn frontmatter_optional_fields_default_to_none() {
        let yaml = r#"
pm-task: true
id: TASK-001
title: "Minimal task"
status: backlog
priority: Low
progress: 0
created: "2026-01-01T00:00:00Z"
updated: "2026-01-01T00:00:00Z"
"#;
        let fm: TaskFrontmatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.id, "TASK-001");
        assert!(fm.sprint.is_none());
        assert!(fm.epic.is_none());
        assert!(fm.assignee.is_none());
        assert!(fm.time_estimate.is_none());
        assert!(fm.time_logged.is_none());
        assert!(fm.due_date.is_none());
        assert!(fm.start_date.is_none());
        assert!(fm.dependencies.is_empty());
        assert!(fm.blocks.is_empty());
        assert!(fm.labels.is_empty());
    }
}
