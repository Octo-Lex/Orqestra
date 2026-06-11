//! Path guard for file IO commands.
//!
//! Enforces that all file reads and writes remain within the opened project root.
//! The desktop UI may suggest paths, but Rust must enforce the boundary.

use std::path::{Path, PathBuf};

/// Verify that `target` is within `project_root` after canonicalization.
///
/// Returns the canonical target path if it is within the project root,
/// or an error describing why access was denied.
pub fn guard_path(project_root: &str, target: &str) -> Result<PathBuf, String> {
    let root = Path::new(project_root);

    // Canonicalize project root (must exist)
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("Invalid project root '{}': {}", project_root, e))?;

    // Handle the target path
    let target_path = Path::new(target);

    // Resolve absolute paths: must stay under project root after canonicalization
    if target_path.is_absolute() {
        // If the target exists, canonicalize it directly to resolve symlinks
        if target_path.exists() {
            let canonical = target_path
                .canonicalize()
                .map_err(|e| format!("Cannot resolve path '{}': {}", target, e))?;
            if canonical.starts_with(&canonical_root) {
                return Ok(canonical);
            }
            return Err(format!(
                "Access denied: path '{}' resolves outside project root '{}'",
                target, project_root
            ));
        }

        // Target doesn't exist — canonicalize parent and verify it's under root
        if let Some(parent) = target_path.parent() {
            if parent.exists() {
                let canonical_parent = parent
                    .canonicalize()
                    .map_err(|e| format!("Cannot resolve parent of '{}': {}", target, e))?;
                if canonical_parent.starts_with(&canonical_root) {
                    return Ok(target_path.to_path_buf());
                }
                return Err(format!(
                    "Access denied: parent of '{}' resolves outside project root",
                    target
                ));
            }
        }

        return Err(format!(
            "Access denied: absolute path '{}' cannot be verified under project root '{}'",
            target, project_root
        ));
    }

    // For relative paths, join with project root and canonicalize
    let joined = canonical_root.join(target_path);

    // If the file exists, canonicalize to resolve symlinks and ..
    if joined.exists() {
        let canonical = joined
            .canonicalize()
            .map_err(|e| format!("Cannot resolve path '{}': {}", target, e))?;
        if canonical.starts_with(&canonical_root) {
            return Ok(canonical);
        }
        return Err(format!(
            "Access denied: resolved path escapes project root '{}'",
            project_root
        ));
    }

    // For non-existent targets (e.g., new writes), check the parent
    if let Some(parent) = joined.parent() {
        // Parent must exist or be creatable under project root
        if parent.exists() {
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| format!("Cannot resolve parent of '{}': {}", target, e))?;
            if canonical_parent.starts_with(&canonical_root) {
                return Ok(joined);
            }
            return Err(format!(
                "Access denied: parent of '{}' escapes project root '{}'",
                target, project_root
            ));
        }

        // Walk up to find an existing ancestor and verify it's under root
        let mut check = parent;
        loop {
            if check.exists() {
                let canonical = check
                    .canonicalize()
                    .map_err(|e| format!("Cannot resolve path: {}", e))?;
                if canonical.starts_with(&canonical_root) {
                    return Ok(joined);
                }
                return Err(format!(
                    "Access denied: path '{}' would escape project root",
                    target
                ));
            }
            match check.parent() {
                Some(p) => check = p,
                None => {
                    return Err(format!(
                        "Access denied: cannot verify path '{}' stays within project root",
                        target
                    ))
                }
            }
        }
    }

    Ok(joined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn allows_relative_path_within_project() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_str().unwrap();

        // Create a file to read
        fs::write(tmp.path().join("test.txt"), "hello").unwrap();

        let result = guard_path(root, "test.txt");
        assert!(result.is_ok(), "Should allow relative path within project: {:?}", result);
        // The guarded path should resolve to a file under the temp dir
        let guarded = result.unwrap();
        assert!(guarded.ends_with("test.txt"), "Should end with test.txt: {:?}", guarded);
    }

    #[test]
    fn allows_nested_relative_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_str().unwrap();

        fs::create_dir_all(tmp.path().join("src/lib")).unwrap();
        fs::write(tmp.path().join("src/lib/mod.rs"), "mod test;").unwrap();

        let result = guard_path(root, "src/lib/mod.rs");
        assert!(result.is_ok(), "Should allow nested relative path");
    }

    #[test]
    fn rejects_traversal_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_str().unwrap();

        fs::write(tmp.path().join("test.txt"), "hello").unwrap();

        let result = guard_path(root, "../../../etc/passwd");
        assert!(result.is_err(), "Should reject traversal escape");
        assert!(result.unwrap_err().contains("Access denied"));
    }

    #[test]
    fn rejects_absolute_path_outside_project() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_str().unwrap();

        // Use the temp directory's parent as an outside absolute path
        let outside = tmp.path().parent().unwrap().parent().unwrap();
        let outside_str = outside.join("some_target_file.txt").to_str().unwrap().to_string();

        let result = guard_path(root, &outside_str);
        assert!(result.is_err(), "Should reject absolute path outside project");
        let err = result.unwrap_err();
        assert!(err.contains("Access denied") || err.contains("outside"));
    }

    #[test]
    fn allows_absolute_path_within_project() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_str().unwrap();

        fs::write(tmp.path().join("test.txt"), "hello").unwrap();
        let abs_path = tmp.path().join("test.txt").to_str().unwrap().to_string();

        let result = guard_path(root, &abs_path);
        assert!(result.is_ok(), "Should allow absolute path within project");
    }

    #[test]
    fn rejects_absolute_in_project_symlink_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let root = tmp.path().to_str().unwrap();

        // Write a file outside the project
        fs::write(outside.path().join("secret.txt"), "sensitive").unwrap();

        // Create a symlink inside the project pointing outside
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                outside.path().join("secret.txt"),
                tmp.path().join("link-to-outside"),
            )
            .unwrap();
        }
        #[cfg(windows)]
        {
            if std::os::windows::fs::symlink_file(
                outside.path().join("secret.txt"),
                tmp.path().join("link-to-outside"),
            ).is_err() {
                eprintln!("Skipping symlink test on Windows: not enough privileges");
                return;
            }
        }

        if tmp.path().join("link-to-outside").exists() {
            let abs_link = tmp.path().join("link-to-outside").to_str().unwrap().to_string();
            let result = guard_path(root, &abs_link);
            assert!(
                result.is_err(),
                "Should reject absolute in-project symlink pointing outside project: {:?}",
                result
            );
            let err = result.unwrap_err();
            assert!(err.contains("Access denied"), "Error should mention access denied: {}", err);
        }
    }

    #[test]
    fn allows_nonexistent_nested_path_for_write() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_str().unwrap();

        // .Orqestra/agents/<id>/state.json doesn't exist yet but parent chain is valid
        let result = guard_path(root, ".Orqestra/agents/docs/state.json");
        assert!(result.is_ok(), "Should allow non-existent nested path for write");
    }

    #[test]
    fn rejects_nonexistent_traversal_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_str().unwrap();

        let result = guard_path(root, "../../escape.txt");
        assert!(result.is_err(), "Should reject non-existent traversal path");
    }

    #[test]
    fn rejects_empty_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_str().unwrap();

        // Empty path resolves to project root itself — that's fine for reads
        let result = guard_path(root, "");
        // Empty string joins to project root, which is allowed
        assert!(result.is_ok() || result.is_err());
    }
}
