//! Error types for md-indexer.
//!
//! The variants here are matched explicitly in the Tauri command layer
//! (`apps/desktop/src-tauri/src/commands/roadmap.rs`), which maps them to
//! `CommandError { code, message }`. New variants must be added to both places.

use std::path::PathBuf;

/// Errors that can occur during roadmap indexing.
#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    #[error("directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("IO error on {0}: {1}")]
    Io(PathBuf, #[source] std::io::Error),

    #[error("parse error in {0}: {1}")]
    ParseError(PathBuf, String),

    #[error("invalid frontmatter in {0}: {1}")]
    InvalidFrontmatter(PathBuf, String),
}

// Equality is useful in tests but cannot be derived because std::io::Error
// doesn't implement Eq. We implement a best-effort comparison.
impl PartialEq for IndexerError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DirectoryNotFound(a), Self::DirectoryNotFound(b)) => a == b,
            (Self::ParseError(a1, a2), Self::ParseError(b1, b2)) => a1 == b1 && a2 == b2,
            (Self::InvalidFrontmatter(a1, a2), Self::InvalidFrontmatter(b1, b2)) => {
                a1 == b1 && a2 == b2
            }
            (Self::Io(a, _), Self::Io(b, _)) => a == b,
            _ => false,
        }
    }
}

impl Eq for IndexerError {}

impl Clone for IndexerError {
    fn clone(&self) -> Self {
        match self {
            Self::DirectoryNotFound(p) => Self::DirectoryNotFound(p.clone()),
            Self::Io(p, e) => Self::Io(p.clone(), std::io::Error::new(e.kind(), e.to_string())),
            Self::ParseError(p, msg) => Self::ParseError(p.clone(), msg.clone()),
            Self::InvalidFrontmatter(p, msg) => Self::InvalidFrontmatter(p.clone(), msg.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn error_display_messages() {
        let path = PathBuf::from("roadmap/bad.md");
        let e = IndexerError::DirectoryNotFound(path.clone());
        assert_eq!(e.to_string(), format!("directory not found: {}", path.display()));

        let e = IndexerError::ParseError(path.clone(), "missing ---".into());
        assert!(e.to_string().contains("missing ---"));

        let e = IndexerError::InvalidFrontmatter(path.clone(), "bad yaml".into());
        assert!(e.to_string().contains("bad yaml"));
    }

    #[test]
    fn partial_eq_on_io_variants() {
        let path = PathBuf::from("roadmap/x.md");
        let e1 = IndexerError::Io(path.clone(), std::io::Error::new(std::io::ErrorKind::NotFound, "not found"));
        let e2 = IndexerError::Io(path.clone(), std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"));
        // Same path, different IO errors — still equal by path
        assert_eq!(e1, e2);
    }
}
