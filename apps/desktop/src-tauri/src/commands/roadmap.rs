use md_indexer::{index_roadmap, IndexerError, Task};
use serde::Serialize;
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
