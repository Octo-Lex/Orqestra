use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitBridgeError {
    #[error("Repository not found at {0}")]
    RepoNotFound(PathBuf),

    #[error("Git operation failed: {0}")]
    GitOperation(String),

    #[error("IO error on {0}: {1}")]
    Io(PathBuf, #[source] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Task file not found: {0}")]
    TaskNotFound(String),

    #[error("YAML parse error: {0}")]
    YamlParse(String),
}
