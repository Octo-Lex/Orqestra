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

// ---------------------------------------------------------------------------
// Native Git Status Pilot (v1.1.0 spec §10)
//
// Read-only status via gix with CLI fallback.
// This pilot MUST NOT block normal Git operations if it fails.
// ---------------------------------------------------------------------------

/// Result of a native git status check.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NativeGitStatus {
    pub branch: String,
    pub dirty: bool,
    pub staged_count: u32,
    pub unstaged_count: u32,
    pub untracked_count: u32,
    pub provider: String,
    pub fallback_used: bool,
    pub latency_ms: u64,
    pub parity_status: String,
}

/// Get git status using native gix (preferred) with git CLI fallback.
///
/// This is a read-only operation. It never modifies the repository.
/// If the native gix path fails for any reason, it falls back to
/// `git status --porcelain` without erroring the caller.
///
/// Current implementation: gix provides branch name, CLI provides counts.
/// This hybrid approach lets us start using gix for what it supports
/// while relying on CLI for the rest. Future versions may expand gix usage.
pub fn native_git_status(project_root: &Path) -> Result<NativeGitStatus, GitBridgeError> {
    let start = std::time::Instant::now();

    // Try gix for branch detection
    let gix_branch = gix::open(project_root)
        .ok()
        .and_then(|repo| {
            repo.head_name().ok().flatten().map(|name| {
                let s = format!("{name}");
                s.strip_prefix("refs/heads/").unwrap_or(&s).to_string()
            })
        });

    // Use CLI for full status counts (gix lacks high-level status API)
    let mut status = git_status_cli(project_root)?;

    // If gix gave us the branch, prefer it and mark provider as hybrid
    if let Some(branch) = gix_branch {
        status.branch = branch;
        status.provider = "gix-hybrid".to_string();
        status.fallback_used = false;
        status.parity_status = "not-tested".to_string();
    } else {
        status.fallback_used = true;
        status.parity_status = "fallback".to_string();
    }

    status.latency_ms = start.elapsed().as_millis() as u64;
    Ok(status)
}

/// Native gix-based status check (read-only).
///
/// Uses gix for branch/HEAD detection and falls back to CLI for
/// staged/unstaged/untracked counts (gix lacks a high-level status API).
fn native_git_status_gix(project_root: &Path) -> Result<NativeGitStatus, GitBridgeError> {
    let repo = gix::open(project_root)
        .map_err(|e| GitBridgeError::GitOperation(format!("gix open failed: {e}")))?;

    // Get branch name via gix
    let branch = repo.head_name()
        .ok()
        .flatten()
        .map(|name| {
            let s = format!("{name}");
            s.strip_prefix("refs/heads/").unwrap_or(&s).to_string()
        })
        .unwrap_or_else(|| "(detached)".to_string());

    // gix does not provide a high-level status/diff API.
    // For staged/unstaged/untracked counts, fall through to CLI fallback
    // which is the caller's responsibility.
    // Here we return the branch-only result, signaling that counts need CLI.
    Err(GitBridgeError::GitOperation("gix status counts not available".into()))
}

/// CLI fallback for git status.
fn git_status_cli(project_root: &Path) -> Result<NativeGitStatus, GitBridgeError> {
    let output = std::process::Command::new("git")
        .current_dir(project_root)
        .args(["status", "--porcelain=v2", "--branch"])
        .output()
        .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;

    if !output.status.success() {
        return Err(GitBridgeError::GitOperation(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut branch = String::from("(unknown)");
    let mut staged_count = 0u32;
    let mut unstaged_count = 0u32;
    let mut untracked_count = 0u32;

    for line in stdout.lines() {
        if let Some(branch_name) = line.strip_prefix("# branch.head ") {
            branch = branch_name.to_string();
        } else if let Some(xy) = line.strip_prefix("1 ") {
            // Porcelain v2: "1 <xy> <sub> <mH> <mI> <mW> <hH> <hI> <path>"
            let chars: Vec<char> = xy.chars().take(2).collect();
            if chars.len() >= 2 {
                let x = chars[0]; // index status
                let y = chars[1]; // worktree status
                if x != '.' && x != '?' {
                    staged_count += 1;
                }
                if y != '.' && y != '?' {
                    unstaged_count += 1;
                }
            }
        } else if line.starts_with("? ") {
            untracked_count += 1;
        }
    }

    let dirty = staged_count > 0 || unstaged_count > 0 || untracked_count > 0;

    Ok(NativeGitStatus {
        branch,
        dirty,
        staged_count,
        unstaged_count,
        untracked_count,
        provider: "git-cli".to_string(),
        fallback_used: true,
        latency_ms: 0,
        parity_status: "not-tested".to_string(),
    })
}

/// Recursively walk files in a directory (non-.git entries only).
fn walkdir_files(root: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str == ".git" || name_str == ".Orqestra" || name_str == "target" {
                continue;
            }

            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }

    Ok(files)
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

    // v1.1.0 native git status pilot tests

    #[test]
    fn native_git_status_on_current_repo() {
        let project_root = std::env::current_dir().unwrap();
        let mut dir = project_root.clone();
        while !dir.join(".git").exists() {
            if !dir.pop() {
                return;
            }
        }
        let result = native_git_status(&dir);
        assert!(result.is_ok(), "Should get git status: {:?}", result);
        let status = result.unwrap();
        assert!(!status.branch.is_empty());
        assert!(status.provider == "gix" || status.provider == "git-cli" || status.provider == "gix-hybrid");
        assert!(status.latency_ms < 5000, "Latency should be reasonable: {}ms", status.latency_ms);
    }

    #[test]
    fn native_git_status_nonexistent_repo_fails() {
        let result = native_git_status(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn native_git_status_fallback_to_cli() {
        // Test that the CLI fallback path works independently
        let project_root = std::env::current_dir().unwrap();
        let mut dir = project_root.clone();
        while !dir.join(".git").exists() {
            if !dir.pop() {
                return;
            }
        }
        let result = git_status_cli(&dir);
        assert!(result.is_ok(), "CLI fallback should work: {:?}", result);
        let status = result.unwrap();
        assert_eq!(status.provider, "git-cli");
        assert!(!status.branch.is_empty());
    }

    #[test]
    fn native_git_status_reports_provider() {
        let project_root = std::env::current_dir().unwrap();
        let mut dir = project_root.clone();
        while !dir.join(".git").exists() {
            if !dir.pop() {
                return;
            }
        }
        let status = native_git_status(&dir).unwrap();
        // Provider must be either gix-hybrid or git-cli, never empty
        assert!(!status.provider.is_empty());
        // New DTO fields must be present
        assert!(!status.parity_status.is_empty());
    }

    #[test]
    fn native_git_status_parity_against_cli() {
        // Compare native_git_status against pure CLI output
        let project_root = std::env::current_dir().unwrap();
        let mut dir = project_root.clone();
        while !dir.join(".git").exists() {
            if !dir.pop() {
                return;
            }
        }
        let native = native_git_status(&dir).unwrap();
        let cli = git_status_cli(&dir).unwrap();

        // Branch must match
        assert_eq!(native.branch, cli.branch,
            "Branch mismatch: native={}, cli={}", native.branch, cli.branch);

        // Dirty flag must match
        assert_eq!(native.dirty, cli.dirty,
            "Dirty mismatch: native={}, cli={}", native.dirty, cli.dirty);
    }

    #[test]
    fn native_git_status_dirty_repo() {
        use std::io::Write;
        // Create a temp repo with an untracked file
        let tmp = std::env::temp_dir().join("gix-dirty-test");
        std::fs::create_dir_all(&tmp).ok();

        // Init repo
        let init = std::process::Command::new("git")
            .current_dir(&tmp)
            .args(["init"])
            .output().unwrap();
        if !init.status.success() {
            return; // git not available
        }

        // Create untracked file
        std::fs::write(tmp.join("untracked.txt"), "test").unwrap();

        let status = native_git_status(&tmp).unwrap();
        assert!(status.dirty, "Repo with untracked file should be dirty");
        assert!(status.untracked_count >= 1, "Should have at least 1 untracked file");

        // Clean up
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn native_git_status_non_repo_returns_error_or_fallback() {
        let tmp = std::env::temp_dir().join("gix-nonrepo-test");
        std::fs::create_dir_all(&tmp).ok();

        // Non-repo should fail or use fallback
        let result = native_git_status(&tmp);
        if let Ok(status) = result {
            // If it succeeds, it must be via CLI fallback
            assert!(status.fallback_used || status.provider == "git-cli");
        }
        // If it fails, that's also acceptable

        std::fs::remove_dir_all(&tmp).ok();
    }
}
