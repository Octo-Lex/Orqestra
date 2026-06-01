use md_indexer::{index_roadmap, IndexerError, Task};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::command;

/// Serializable error for the frontend.
/// Never expose internal IndexerError variants directly to TypeScript.
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

impl From<IndexerError> for CommandError {
    fn from(e: IndexerError) -> Self {
        match e {
            IndexerError::DirectoryNotFound(_) => CommandError {
                code: "ROADMAP_NOT_FOUND",
                message: e.to_string(),
            },
            IndexerError::Io(_, _) => CommandError {
                code: "IO_ERROR",
                message: e.to_string(),
            },
            _ => CommandError {
                code: "PARSE_ERROR",
                message: e.to_string(),
            },
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Serialize)]
pub struct IndexRoadmapResult {
    pub tasks: Vec<Task>,
    pub warnings: Vec<String>,
}

/// Index the roadmap/ directory relative to the given project root.
/// Called from TypeScript as: invoke('index_roadmap_cmd', { projectRoot: '/path/to/project' })
#[command]
pub fn index_roadmap_cmd(project_root: String) -> CommandResult<IndexRoadmapResult> {
    let roadmap_dir = PathBuf::from(&project_root).join("roadmap");
    let result = index_roadmap(&roadmap_dir).map_err(CommandError::from)?;

    let warnings = result
        .errors
        .iter()
        .map(|(path, err)| format!("{}: {}", path.display(), err))
        .collect();

    Ok(IndexRoadmapResult {
        tasks: result.tasks,
        warnings,
    })
}

#[command]
pub fn get_task(project_root: String, task_id: String) -> CommandResult<Option<Task>> {
    let roadmap_dir = PathBuf::from(&project_root).join("roadmap");
    let result = index_roadmap(&roadmap_dir).map_err(CommandError::from)?;
    let task = result.tasks.into_iter().find(|t| t.frontmatter.id == task_id);
    Ok(task)
}

#[derive(Debug, Serialize)]
pub struct UpdateTaskStatusResult {
    pub success: bool,
    pub new_status: String,
}

/// Update a task's status by rewriting its frontmatter in the .md file.
/// Reads the file, updates the `status:` line, writes back atomically.
#[command]
pub fn update_task_status_cmd(
    project_root: String,
    task_id: String,
    new_status: String,
) -> CommandResult<UpdateTaskStatusResult> {
    let roadmap_dir = PathBuf::from(&project_root).join("roadmap");
    let result = index_roadmap(&roadmap_dir).map_err(CommandError::from)?;
    let task = result
        .tasks
        .into_iter()
        .find(|t| t.frontmatter.id == task_id)
        .ok_or_else(|| CommandError {
            code: "TASK_NOT_FOUND",
            message: format!("Task {} not found", task_id),
        })?;

    let file_path = PathBuf::from(&project_root).join(&task.source_path);
    let content = fs::read_to_string(&file_path).map_err(|e| CommandError {
        code: "IO_ERROR",
        message: format!("Failed to read {}: {}", file_path.display(), e),
    })?;

    // Replace the status line in frontmatter
    let updated = content;
    let mut lines: Vec<String> = updated.lines().map(String::from).collect();
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
        return Err(CommandError {
            code: "STATUS_NOT_FOUND",
            message: format!("No status field in frontmatter for {}", task_id),
        });
    }

    // Write the updated content back
    let new_content = lines.join("\n");
    // Ensure trailing newline
    let new_content = if new_content.ends_with('\n') {
        new_content
    } else {
        format!("{}\n", new_content)
    };

    // Atomic write: write to temp file, then rename
    let tmp_path = file_path.with_extension("md.tmp");
    fs::write(&tmp_path, &new_content).map_err(|e| CommandError {
        code: "IO_ERROR",
        message: format!("Failed to write temp file: {}", e),
    })?;
    fs::rename(&tmp_path, &file_path).map_err(|e| CommandError {
        code: "IO_ERROR",
        message: format!("Failed to rename temp file: {}", e),
    })?;

    Ok(UpdateTaskStatusResult {
        success: true,
        new_status,
    })
}
