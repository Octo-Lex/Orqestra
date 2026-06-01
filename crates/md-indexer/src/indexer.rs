//! Roadmap directory indexer.
//!
//! Walks a `roadmap/` directory, parses each `.md` file as a task, and returns
//! an [`IndexResult`] containing successfully parsed tasks and per-file errors.

use std::path::Path;

use walkdir::WalkDir;

use crate::error::IndexerError;
use crate::parser::parse_task_content;
use crate::types::{IndexResult, Task};

/// Index all `.md` files in the given directory as tasks.
///
/// Returns an [`IndexResult`] with:
/// - `tasks`: successfully parsed tasks
/// - `errors`: per-file parse failures (caller can surface as warnings)
///
/// The directory must exist. Files that fail to parse are skipped but recorded
/// in `errors` — the function never returns `Err` for individual file failures.
pub fn index_roadmap(dir: &Path) -> Result<IndexResult, IndexerError> {
    if !dir.exists() {
        return Err(IndexerError::DirectoryNotFound(dir.to_path_buf()));
    }

    if !dir.is_dir() {
        return Err(IndexerError::DirectoryNotFound(dir.to_path_buf()));
    }

    let mut tasks: Vec<Task> = Vec::new();
    let mut errors: Vec<(std::path::PathBuf, IndexerError)> = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only process .md files
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        // Skip directories (shouldn't happen after filter_map, but be safe)
        if !path.is_file() {
            continue;
        }

        match std::fs::read_to_string(path) {
            Ok(content) => match parse_task_content(&content, path) {
                Ok(task) => tasks.push(task),
                Err(e) => errors.push((path.to_path_buf(), e)),
            },
            Err(io_err) => {
                errors.push((path.to_path_buf(), IndexerError::Io(path.to_path_buf(), io_err)));
            }
        }
    }

    // Deterministic ordering: sort by source path
    tasks.sort_by(|a, b| a.source_path.cmp(&b.source_path));
    errors.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(IndexResult { tasks, errors })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixtures_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("roadmap")
    }

    #[test]
    fn index_sample_roadmap() {
        let dir = fixtures_dir();
        let result = index_roadmap(&dir).expect("indexing should succeed");

        assert_eq!(result.tasks.len(), 1, "fixture dir has exactly one task");
        assert_eq!(result.tasks[0].frontmatter.id, "TASK-2026-042");
        assert!(
            result.errors.is_empty(),
            "fixture should have no parse errors, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn index_empty_dir() {
        let tmp = tempfile_path("orqestra_test_empty");
        fs::create_dir_all(&tmp).unwrap();

        let result = index_roadmap(&tmp).expect("empty dir should succeed");
        assert!(result.tasks.is_empty());
        assert!(result.errors.is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn index_nonexistent_dir_returns_error() {
        let result = index_roadmap(Path::new("/no/such/directory/orqestra_test"));
        assert!(result.is_err());
        match result.unwrap_err() {
            IndexerError::DirectoryNotFound(_) => {}
            other => panic!("expected DirectoryNotFound, got {:?}", other),
        }
    }

    #[test]
    fn index_with_corrupt_file() {
        let tmp = tempfile_path("orqestra_test_corrupt");
        fs::create_dir_all(&tmp).unwrap();

        // Write a valid task
        fs::write(
            tmp.join("good.md"),
            r#"---
pm-task: true
id: TASK-GOOD
title: "Good task"
status: backlog
priority: Low
progress: 0
created: "2026-01-01T00:00:00Z"
updated: "2026-01-01T00:00:00Z"
---
Good content
"#,
        )
        .unwrap();

        // Write a file with invalid YAML
        fs::write(
            tmp.join("bad.md"),
            "---\nnot: valid: yaml: [[[---\nSome body",
        )
        .unwrap();

        // Write a non-task .md (should be an error, not silently skipped)
        fs::write(
            tmp.join("not_task.md"),
            "---\npm-epic: true\nid: E-1\n---\nEpic content",
        )
        .unwrap();

        let result = index_roadmap(&tmp).expect("indexing should succeed even with errors");

        assert_eq!(
            result.tasks.len(),
            1,
            "only the good task should parse: tasks = {:?}",
            result.tasks.iter().map(|t| &t.frontmatter.id).collect::<Vec<_>>()
        );
        assert_eq!(
            result.errors.len(),
            2,
            "bad.md and not_task.md should produce errors: errors = {:?}",
            result.errors
                .iter()
                .map(|(p, _)| p.file_name().unwrap().to_string_lossy())
                .collect::<Vec<_>>()
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    /// Helper: create a deterministic temp path that doesn't collide.
    fn tempfile_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{}_{}", name, std::process::id()))
    }
}
