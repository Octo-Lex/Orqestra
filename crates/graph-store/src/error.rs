use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphStoreError {
    #[error("Serialization error: {0}")]
    Serialize(String),

    #[error("IO error on {0}: {1}")]
    Io(PathBuf, #[source] std::io::Error),

    #[error("No triples directory at {0}")]
    NoTriplesDir(PathBuf),

    #[error("No commits directory at {0}")]
    NoCommitsDir(PathBuf),
}

impl From<serde_json::Error> for GraphStoreError {
    fn from(e: serde_json::Error) -> Self {
        GraphStoreError::Serialize(e.to_string())
    }
}

impl Clone for GraphStoreError {
    fn clone(&self) -> Self {
        match self {
            GraphStoreError::Serialize(msg) => GraphStoreError::Serialize(msg.clone()),
            GraphStoreError::Io(path, e) => GraphStoreError::Io(
                path.clone(),
                std::io::Error::new(e.kind(), e.to_string()),
            ),
            GraphStoreError::NoTriplesDir(p) => GraphStoreError::NoTriplesDir(p.clone()),
            GraphStoreError::NoCommitsDir(p) => GraphStoreError::NoCommitsDir(p.clone()),
        }
    }
}
