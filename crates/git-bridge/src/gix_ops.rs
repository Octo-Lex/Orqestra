use crate::error::GitBridgeError;
use std::path::{Path, PathBuf};

/// Stage files via git add (shell-out — staging API not high-level in gix).
/// This is documented as a partial migration per spec §8.2.
pub fn stage_files(
    project_root: &Path,
    files: &[PathBuf],
) -> Result<(), GitBridgeError> {
    let mut cmd = std::process::Command::new("git");
    cmd.current_dir(project_root);

    if files.is_empty() {
        cmd.args(["add", "roadmap/"]);
    } else {
        cmd.arg("add");
        for f in files {
            cmd.arg(f);
        }
    }

    let output = cmd
        .output()
        .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;

    if !output.status.success() {
        return Err(GitBridgeError::GitOperation(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}

/// Create a commit natively using gix 0.84.
///
/// Flow:
/// 1. `git write-tree` to convert index to tree object (one shell-out)
/// 2. `gix::Repository::commit()` creates the commit object and updates HEAD (native)
///
/// The commit object creation and reference update are fully native gix.
/// The tree-from-index conversion remains a shell-out (gix lacks this high-level API).
pub fn create_commit_native(
    project_root: &Path,
    message: &str,
    _author_name: &str,
    _author_email: &str,
) -> Result<String, GitBridgeError> {
    let repo = gix::open(project_root)
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to open repository: {e}")))?;

    // Write tree from index via shell-out (gix lacks write-tree helper)
    let tree_output = std::process::Command::new("git")
        .current_dir(project_root)
        .args(["write-tree"])
        .output()
        .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;

    if !tree_output.status.success() {
        return Err(GitBridgeError::GitOperation(
            String::from_utf8_lossy(&tree_output.stderr).to_string(),
        ));
    }

    let tree_hash = String::from_utf8_lossy(&tree_output.stdout).trim().to_string();
    let tree_id = gix::ObjectId::from_hex(tree_hash.as_bytes())
        .map_err(|e| GitBridgeError::GitOperation(format!("Invalid tree hash: {e}")))?;

    // Get parent commit (HEAD)
    let parents: Vec<gix::ObjectId> = match repo.head_id() {
        Ok(id) => vec![id.detach()],
        Err(_) => vec![],
    };

    // Create commit natively — reads author/committer from git config
    let commit_id = repo.commit(
        "HEAD",
        message,
        tree_id,
        parents.iter().copied(),
    )
    .map_err(|e| GitBridgeError::GitOperation(format!("Failed to create commit: {e}")))?;

    Ok(commit_id.detach().to_hex().to_string())
}

/// Get the unified diff for a commit (shell-out for diff formatting).
/// gix provides tree diffing but not unified diff output formatting.
pub fn get_commit_diff_native(
    project_root: &Path,
    hash: &str,
) -> Result<String, GitBridgeError> {
    let output = std::process::Command::new("git")
        .current_dir(project_root)
        .args(["show", "--unified=3", hash])
        .output()
        .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;

    if !output.status.success() {
        return Err(GitBridgeError::GitOperation(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get HEAD commit hash via gix (fully native).
pub fn get_head_hash(project_root: &Path) -> Result<String, GitBridgeError> {
    let repo = gix::open(project_root)
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to open repository: {e}")))?;

    let head_id = repo.head_id()
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to get HEAD: {e}")))?;

    Ok(head_id.detach().to_hex().to_string())
}

/// Check if a path is a git repository.
pub fn is_git_repo(project_root: &Path) -> bool {
    gix::open(project_root).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_head_hash_on_current_repo() {
        let project_root = std::env::current_dir().unwrap();
        let mut dir = project_root.clone();
        while !dir.join(".git").exists() {
            if !dir.pop() {
                return;
            }
        }
        let result = get_head_hash(&dir);
        assert!(result.is_ok(), "Should get HEAD hash: {:?}", result);
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.len() >= 7);
    }

    #[test]
    fn get_head_hash_nonexistent_repo_fails() {
        let result = get_head_hash(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn is_git_repo_detects_repo() {
        let project_root = std::env::current_dir().unwrap();
        let mut dir = project_root.clone();
        while !dir.join(".git").exists() {
            if !dir.pop() {
                return;
            }
        }
        assert!(is_git_repo(&dir));
    }

    #[test]
    fn is_git_repo_detects_non_repo() {
        let tmp = std::env::temp_dir().join("not-a-repo-test-gix");
        std::fs::create_dir_all(&tmp).ok();
        assert!(!is_git_repo(&tmp));
        std::fs::remove_dir_all(&tmp).ok();
    }
}
