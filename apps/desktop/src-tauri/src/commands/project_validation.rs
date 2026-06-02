//! Project validation and sample project generation.
//!
//! Validates that a selected folder is a loadable Orqestra repository and can
//! generate a built-in sample project for external reviewers.

use md_indexer::index_roadmap;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::command;

use super::roadmap::CommandError;

type CommandResult<T> = Result<T, CommandError>;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ProjectValidationResult {
    pub project_root: String,
    pub status: &'static str,
    pub detected: ProjectDetectedState,
    pub errors: Vec<ProjectValidationIssue>,
    pub warnings: Vec<ProjectValidationIssue>,
    pub suggested_actions: Vec<SuggestedAction>,
}

#[derive(Debug, Serialize)]
pub struct ProjectDetectedState {
    pub is_git_repo: bool,
    pub has_roadmap_dir: bool,
    pub has_index_md: bool,
    pub task_count: usize,
    pub malformed_task_count: usize,
    pub has_orqestra_dir: bool,
    pub has_orqestra_toml: bool,
    pub has_dashboard_json: bool,
}

#[derive(Debug, Serialize)]
pub struct ProjectValidationIssue {
    pub code: String,
    pub path: Option<String>,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Serialize)]
pub struct SuggestedAction {
    pub id: String,
    pub label: String,
    pub description: String,
    pub kind: String,
    pub safe: bool,
}

#[derive(Debug, Serialize)]
pub struct SampleProjectResult {
    pub path: String,
    pub created: bool,
    pub task_count: usize,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate whether a folder is a loadable Orqestra repository.
#[command]
pub fn validate_project_cmd(project_root: String) -> CommandResult<ProjectValidationResult> {
    let root = Path::new(&project_root);

    // Check accessibility
    if !root.exists() {
        return Ok(ProjectValidationResult {
            project_root: project_root.clone(),
            status: "inaccessible",
            detected: empty_detected(),
            errors: vec![ProjectValidationIssue {
                code: "PATH_NOT_FOUND".into(),
                path: Some(project_root.clone()),
                message: "The specified path does not exist".into(),
                severity: "error".into(),
            }],
            warnings: vec![],
            suggested_actions: vec![SuggestedAction {
                id: "choose_different".into(),
                label: "Choose a different folder".into(),
                description: "Select an existing directory".into(),
                kind: "retry".into(),
                safe: true,
            }],
        });
    }

    let metadata = match fs::metadata(root) {
        Ok(m) => m,
        Err(e) => {
            return Ok(ProjectValidationResult {
                project_root: project_root.clone(),
                status: "inaccessible",
                detected: empty_detected(),
                errors: vec![ProjectValidationIssue {
                    code: "PATH_INACCESSIBLE".into(),
                    path: Some(project_root.clone()),
                    message: format!("Cannot read directory: {}", e),
                    severity: "error".into(),
                }],
                warnings: vec![],
                suggested_actions: vec![],
            });
        }
    };

    if !metadata.is_dir() {
        return Ok(ProjectValidationResult {
            project_root: project_root.clone(),
            status: "inaccessible",
            detected: empty_detected(),
            errors: vec![ProjectValidationIssue {
                code: "NOT_A_DIRECTORY".into(),
                path: Some(project_root.clone()),
                message: "The specified path is not a directory".into(),
                severity: "error".into(),
            }],
            warnings: vec![],
            suggested_actions: vec![],
        });
    }

    // Detect state
    let is_git_repo = root.join(".git").exists();
    let has_roadmap_dir = root.join("roadmap").is_dir();
    let has_index_md = root.join("roadmap").join("_index.md").exists();
    let has_orqestra_dir = root.join(".Orqestra").is_dir();
    let has_orqestra_toml = root.join("Orqestra.toml").exists();

    // Check for dashboard JSON — look in common locations
    let has_dashboard_json = root.join("apps").join("dashboard").join("public").join("roadmap.json").exists()
        || root.join("dashboard").join("roadmap.json").exists()
        || root.join("public").join("roadmap.json").exists();

    // If no roadmap dir, it's not an Orqestra project
    if !has_roadmap_dir {
        return Ok(ProjectValidationResult {
            project_root: project_root.clone(),
            status: "not_orqestra",
            detected: ProjectDetectedState {
                is_git_repo,
                has_roadmap_dir: false,
                has_index_md: false,
                task_count: 0,
                malformed_task_count: 0,
                has_orqestra_dir,
                has_orqestra_toml,
                has_dashboard_json,
            },
            errors: vec![ProjectValidationIssue {
                code: "ROADMAP_NOT_FOUND".into(),
                path: Some(project_root.clone()),
                message: "This folder does not contain a roadmap/ directory".into(),
                severity: "error".into(),
            }],
            warnings: vec![],
            suggested_actions: vec![
                SuggestedAction {
                    id: "open_sample".into(),
                    label: "Try sample project".into(),
                    description: "Create a sample Orqestra project to explore features".into(),
                    kind: "open_sample".into(),
                    safe: true,
                },
                SuggestedAction {
                    id: "initialize".into(),
                    label: "Initialize project".into(),
                    description: "Create a roadmap/ directory in this folder".into(),
                    kind: "initialize_project".into(),
                    safe: true,
                },
            ],
        });
    }

    // Try to index the roadmap
    let roadmap_dir = root.join("roadmap");
    let index_result = index_roadmap(&roadmap_dir);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let (task_count, malformed_task_count) = match &index_result {
        Ok(result) => {
            let tc = result.tasks.len();
            // Count malformed as index errors
            for (path, err) in &result.errors {
                warnings.push(ProjectValidationIssue {
                    code: "TASK_PARSE_WARNING".into(),
                    path: Some(path.display().to_string()),
                    message: err.to_string(),
                    severity: "warning".into(),
                });
            }
            (tc, result.errors.len())
        }
        Err(e) => {
            errors.push(ProjectValidationIssue {
                code: "ROADMAP_INDEX_FAILED".into(),
                path: Some(roadmap_dir.display().to_string()),
                message: e.to_string(),
                severity: "error".into(),
            });
            (0, 0)
        }
    };

    // Check for duplicate task IDs
    if let Ok(result) = &index_result {
        let mut seen_ids = std::collections::HashSet::new();
        let mut duplicate_ids = std::collections::HashSet::new();
        for task in &result.tasks {
            let id = &task.frontmatter.id;
            if !seen_ids.insert(id.clone()) {
                duplicate_ids.insert(id.clone());
            }
        }
        for dup_id in duplicate_ids {
            errors.push(ProjectValidationIssue {
                code: "DUPLICATE_TASK_ID".into(),
                path: None,
                message: format!("Duplicate task ID: {}", dup_id),
                severity: "error".into(),
            });
        }
    }

    // Missing optional files → warnings
    if !has_orqestra_dir {
        warnings.push(ProjectValidationIssue {
            code: "MISSING_ORQESTRA_DIR".into(),
            path: None,
            message: "No .Orqestra/ directory (optional, local metadata)".into(),
            severity: "info".into(),
        });
    }
    if !has_orqestra_toml {
        warnings.push(ProjectValidationIssue {
            code: "MISSING_ORQESTRA_TOML".into(),
            path: None,
            message: "No Orqestra.toml config file (optional)".into(),
            severity: "info".into(),
        });
    }
    if !has_index_md && has_roadmap_dir {
        warnings.push(ProjectValidationIssue {
            code: "MISSING_INDEX_MD".into(),
            path: None,
            message: "No roadmap/_index.md coordinator file (optional)".into(),
            severity: "info".into(),
        });
    }

    // Determine status
    let status = if errors.is_empty() {
        if task_count > 0 {
            "valid"
        } else {
            "repairable"
        }
    } else if has_roadmap_dir && task_count > 0 && errors.iter().all(|e| e.code != "DUPLICATE_TASK_ID" && e.code != "ROADMAP_INDEX_FAILED") {
        "repairable"
    } else {
        "invalid"
    };

    let mut suggested_actions = Vec::new();
    if status == "repairable" && !has_orqestra_dir {
        suggested_actions.push(SuggestedAction {
            id: "create_orqestra_dir".into(),
            label: "Create .Orqestra/ directory".into(),
            description: "Add local metadata directory".into(),
            kind: "create_file".into(),
            safe: true,
        });
    }
    if status == "repairable" && !has_orqestra_toml {
        suggested_actions.push(SuggestedAction {
            id: "create_orqestra_toml".into(),
            label: "Create Orqestra.toml".into(),
            description: "Add project configuration file".into(),
            kind: "create_file".into(),
            safe: true,
        });
    }
    if status == "not_orqestra" || status == "invalid" {
        suggested_actions.push(SuggestedAction {
            id: "open_sample".into(),
            label: "Try sample project".into(),
            description: "Create a sample Orqestra project to explore features".into(),
            kind: "open_sample".into(),
            safe: true,
        });
    }

    Ok(ProjectValidationResult {
        project_root: project_root.clone(),
        status,
        detected: ProjectDetectedState {
            is_git_repo,
            has_roadmap_dir,
            has_index_md,
            task_count,
            malformed_task_count,
            has_orqestra_dir,
            has_orqestra_toml,
            has_dashboard_json,
        },
        errors,
        warnings,
        suggested_actions,
    })
}

fn empty_detected() -> ProjectDetectedState {
    ProjectDetectedState {
        is_git_repo: false,
        has_roadmap_dir: false,
        has_index_md: false,
        task_count: 0,
        malformed_task_count: 0,
        has_orqestra_dir: false,
        has_orqestra_toml: false,
        has_dashboard_json: false,
    }
}

// ---------------------------------------------------------------------------
// Sample Project
// ---------------------------------------------------------------------------

/// Create a sample Orqestra project at the given destination, or in a temp
/// location if destination is None.
#[command]
pub fn create_sample_project_cmd(destination: Option<String>) -> CommandResult<SampleProjectResult> {
    let dest = match destination {
        Some(d) => PathBuf::from(d),
        None => {
            // Default: next to the app in a standard location
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join("Orqestra-Sample")
        }
    };

    let created = !dest.exists();

    // Create directory structure
    fs::create_dir_all(&dest).map_err(|e| CommandError {
        code: "IO_ERROR",
        message: format!("Failed to create directory: {}", e),
    })?;

    // README.md
    write_if_missing(&dest.join("README.md"), r#"# Sample Orqestra Project

This is a sample project demonstrating Orqestra's project management capabilities.

## Structure

- `roadmap/` — Task files with YAML frontmatter
- `roadmap/_index.md` — Sprint/epic coordinator
- `src/` — Sample source files

## Views

Open this project in Orqestra to see Table, Gantt, and Kanban views.
"#)?;

    // Orqestra.toml
    write_if_missing(&dest.join("Orqestra.toml"), r#"[project]
name = "sample-orqestra-project"
version = "0.1.0"

[roadmap]
task_prefix = "SAMPLE"

[agents]
docs = { enabled = true, mode = "propose" }
bugfix = { enabled = true, mode = "propose" }
"#)?;

    // roadmap/ directory
    let roadmap_dir = dest.join("roadmap");
    fs::create_dir_all(&roadmap_dir).map_err(|e| CommandError {
        code: "IO_ERROR",
        message: format!("Failed to create roadmap dir: {}", e),
    })?;

    // _index.md coordinator
    write_if_missing(&roadmap_dir.join("_index.md"), r#"---
pm-index: true
project: "Sample Orqestra Project"
sprints:
  - name: "Sprint 1"
    start: "2026-06-01"
    end: "2026-06-14"
  - name: "Sprint 2"
    start: "2026-06-15"
    end: "2026-06-28"
epics:
  - name: "Core Features"
    description: "Essential feature development"
team:
  - name: "Alex"
    role: "developer"
  - name: "Sam"
    role: "designer"
---
# Sample Project Roadmap

This roadmap demonstrates Orqestra's task management features.
"#)?;

    // Task 1 — Backlog
    write_if_missing(&roadmap_dir.join("TASK-2026-SAMPLE-001.md"), r#"---
pm-task: true
id: TASK-2026-SAMPLE-001
title: "Design landing page"
type: Task
status: backlog
priority: High
sprint: "Sprint 2"
epic: "Core Features"
assignee: "Sam"
labels:
  - design
  - frontend
time_estimate: "4h"
dependencies: []
created: "2026-06-01T10:00:00Z"
updated: "2026-06-01T10:00:00Z"
---
## Context

Design the main landing page for the application.

## Acceptance Criteria

- [ ] Wireframe approved
- [ ] High-fidelity mockup created
- [ ] Responsive breakpoints defined

## Agent Notes

This task has no dependencies and can start in Sprint 2.
"#)?;

    // Task 2 — In Progress (depends on Task 1)
    write_if_missing(&roadmap_dir.join("TASK-2026-SAMPLE-002.md"), r#"---
pm-task: true
id: TASK-2026-SAMPLE-002
title: "Implement user authentication"
type: Task
status: in-progress
priority: Critical
sprint: "Sprint 1"
epic: "Core Features"
assignee: "Alex"
labels:
  - backend
  - security
time_estimate: "8h"
time_logged: "3h"
dependencies:
  - TASK-2026-SAMPLE-001
blocks:
  - TASK-2026-SAMPLE-003
progress: 40
start_date: "2026-06-02"
due_date: "2026-06-10"
created: "2026-06-01T09:00:00Z"
updated: "2026-06-02T14:00:00Z"
---
## Context

Implement JWT-based user authentication with refresh tokens.

## Acceptance Criteria

- [x] Login endpoint functional
- [ ] Token refresh working
- [ ] Session management complete
- [ ] Password reset flow

## Agent Notes

Login endpoint is complete. Token refresh is next.
"#)?;

    // Task 3 — Done
    write_if_missing(&roadmap_dir.join("TASK-2026-SAMPLE-003.md"), r#"---
pm-task: true
id: TASK-2026-SAMPLE-003
title: "Set up project repository"
type: Task
status: done
priority: Critical
sprint: "Sprint 1"
epic: "Core Features"
assignee: "Alex"
labels:
  - devops
time_estimate: "2h"
time_logged: "1h30m"
progress: 100
start_date: "2026-05-28"
due_date: "2026-05-30"
created: "2026-05-28T08:00:00Z"
updated: "2026-05-29T16:00:00Z"
---
## Context

Initialize the project with Git, CI/CD pipeline, and base configuration.

## Acceptance Criteria

- [x] Git repository initialized
- [x] CI pipeline configured
- [x] Base project structure created

## Agent Notes

Completed ahead of schedule.
"#)?;

    // ADR file
    write_if_missing(&roadmap_dir.join("ADR-001.md"), r#"---
pm-task: true
id: ADR-001
title: "Use JWT for authentication"
type: Task
status: done
priority: Medium
labels:
  - architecture
  - documentation
created: "2026-05-27T10:00:00Z"
updated: "2026-05-27T10:00:00Z"
---
## Decision

Use JSON Web Tokens (JWT) for stateless authentication.

## Rationale

JWT tokens work well with our microservices architecture and don't require
server-side session storage.
"#)?;

    // src/ directory with sample file
    let src_dir = dest.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| CommandError {
        code: "IO_ERROR",
        message: format!("Failed to create src dir: {}", e),
    })?;
    write_if_missing(&src_dir.join("sample.ts"), r#"// Sample source file for Orqestra demo
// This file can be used as a scope target for the bugfix agent demo

export function greet(name: string): string {
  return `Hello, ${name}! Welcome to Orqestra.`;
}

export function formatDate(date: Date): string {
  return date.toISOString().split('T')[0];
}

export function calculateEstimate(hours: number, complexity: number): number {
  return hours * complexity;
}
"#)?;

    // .Orqestra/ directory with .gitignore
    let orq_dir = dest.join(".Orqestra");
    fs::create_dir_all(&orq_dir).map_err(|e| CommandError {
        code: "IO_ERROR",
        message: format!("Failed to create .Orqestra dir: {}", e),
    })?;
    write_if_missing(&orq_dir.join(".gitignore"), r#"# Orqestra local data — do not commit
graph/
agents/
*.log
"#)?;

    // Count tasks
    let task_count = 4; // SAMPLE-001, SAMPLE-002, SAMPLE-003, ADR-001

    Ok(SampleProjectResult {
        path: dest.display().to_string(),
        created,
        task_count,
    })
}

fn write_if_missing(path: &Path, content: &str) -> CommandResult<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| CommandError {
            code: "IO_ERROR",
            message: format!("Failed to create parent dir: {}", e),
        })?;
    }
    fs::write(path, content).map_err(|e| CommandError {
        code: "IO_ERROR",
        message: format!("Failed to write {}: {}", path.display(), e),
    })
}
