use crate::error::GitBridgeError;
use serde::Serialize;
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

// ---------------------------------------------------------------------------
// v2.5.0: Fully native commit path (no CLI shell-outs)
// ---------------------------------------------------------------------------

/// Method used for each commit-path step.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GitWriteMethod {
    Native,
    CliFallback,
}

/// Diagnostic for the commit path used.
#[derive(Debug, Clone, Serialize)]
pub struct CommitPathDiagnostic {
    pub tree_method: GitWriteMethod,
    pub commit_method: GitWriteMethod,
    pub head_update_method: GitWriteMethod,
    pub provider_label: String,
    pub fallback_reason: Option<String>,
}

/// Result of a fully native commit attempt.
#[derive(Debug, Clone, Serialize)]
pub struct NativeWriteCommitResult {
    pub hash: String,
    pub parent_hashes: Vec<String>,
    pub diagnostic: CommitPathDiagnostic,
    pub elapsed_ms: u64,
}

/// Build a tree object from the current index using gix (no shell-out).
///
/// Constructs tree entries from index entries, builds nested trees for
/// subdirectories, and writes the root tree object to the ODB.
fn native_write_tree_from_index(
    repo: &gix::Repository,
) -> Result<gix::ObjectId, GitBridgeError> {
    let index = repo.open_index()
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to open index: {e}")))?;

    use std::collections::BTreeMap;
    use gix::bstr::BString;
    use gix::objs::tree::{Entry, EntryMode, EntryKind};

    enum Node {
        Leaf(EntryMode, gix::ObjectId),
        Dir(BTreeMap<BString, Node>),
    }

    let mut root: BTreeMap<BString, Node> = BTreeMap::new();

    for entry in index.entries() {
        let path = entry.path_in(&index.path_backing());
        let path_str = std::str::from_utf8(path)
            .map_err(|e| GitBridgeError::GitOperation(format!("Invalid path in index: {e}")))?;
        let parts: Vec<&str> = path_str.split('/').collect();

        let mode = EntryMode::try_from(entry.mode.bits())
            .unwrap_or(EntryKind::Blob.into());
        let id = entry.id;

        let mut current = &mut root;
        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;
            let key = BString::from(part.to_string());

            if is_last {
                current.insert(key, Node::Leaf(mode, id));
                break;
            } else {
                let next = current.entry(key).or_insert_with(|| {
                    Node::Dir(BTreeMap::new())
                });
                current = match next {
                    Node::Dir(ref mut m) => m,
                    Node::Leaf(_, _) => break,
                };
            }
        }
    }

    fn write_tree(
        repo: &gix::Repository,
        nodes: &BTreeMap<BString, Node>,
    ) -> Result<gix::ObjectId, GitBridgeError> {
        let mut entries: Vec<Entry> = Vec::new();

        for (name, node) in nodes {
            match node {
                Node::Leaf(mode, oid) => {
                    entries.push(Entry {
                        mode: *mode,
                        filename: name.clone(),
                        oid: *oid,
                    });
                }
                Node::Dir(children) => {
                    let subtree_id = write_tree(repo, children)?;
                    entries.push(Entry {
                        mode: EntryKind::Tree.into(),
                        filename: name.clone(),
                        oid: subtree_id,
                    });
                }
            }
        }

        entries.sort_by(|a, b| a.filename.cmp(&b.filename));

        let tree = gix::objs::Tree { entries };
        let id = repo.write_object(tree)
            .map_err(|e| GitBridgeError::GitOperation(format!("Failed to write tree: {e}")))?;

        Ok(id.detach())
    }

    write_tree(repo, &root)
}

/// Fully native commit path. All-or-nothing: if any step fails,
/// the caller must use the CLI fallback path instead.
///
/// Compare-and-swap HEAD: aborts if HEAD != expected_parent.
pub fn native_commit_full(
    project_root: &Path,
    message: &str,
    expected_parent: &str,
    reviewed_proposal_id: &str,
) -> Result<NativeWriteCommitResult, GitBridgeError> {
    let start = std::time::Instant::now();

    // Validate message
    if message.trim().is_empty() {
        return Err(GitBridgeError::GitOperation(
            "GIT_COMMIT_ERROR: empty message".into(),
        ));
    }

    // Validate reviewed proposal
    if reviewed_proposal_id.trim().is_empty() {
        return Err(GitBridgeError::GitOperation(
            "GIT_COMMIT_ERROR: reviewed proposal ID required".into(),
        ));
    }

    let repo = gix::open(project_root)
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to open repository: {e}")))?;

    // Compare-and-swap: verify HEAD matches expected_parent
    let current_head = repo.head_id()
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to get HEAD: {e}")))?
        .detach();

    let expected_oid = gix::ObjectId::from_hex(expected_parent.as_bytes())
        .map_err(|e| GitBridgeError::GitOperation(format!("Invalid expected_parent: {e}")))?;

    if current_head != expected_oid {
        return Err(GitBridgeError::GitOperation(
            format!("HEAD_CHANGED: expected {} but found {}",
                expected_parent,
                current_head.to_hex())
        ));
    }

    // Step 1: Write tree from index (native)
    let tree_id = native_write_tree_from_index(&repo)?;

    // Step 2: Create commit (native)
    let parents: Vec<gix::ObjectId> = vec![current_head];
    let commit_id = repo.commit(
        "HEAD",
        message,
        tree_id,
        parents.iter().copied(),
    )
    .map_err(|e| GitBridgeError::GitOperation(format!("Failed to create commit: {e}")))?;

    // Step 3: HEAD update is done by repo.commit() above (native)
    // Verify HEAD now points to our commit
    let new_head = repo.head_id()
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to verify HEAD: {e}")))?
        .detach();

    if new_head != commit_id.detach() {
        return Err(GitBridgeError::GitOperation(
            format!("HEAD_RACE: commit created {} but HEAD is {}",
                commit_id.detach().to_hex(),
                new_head.to_hex())
        ));
    }

    let elapsed = start.elapsed();

    Ok(NativeWriteCommitResult {
        hash: commit_id.detach().to_hex().to_string(),
        parent_hashes: vec![current_head.to_hex().to_string()],
        diagnostic: CommitPathDiagnostic {
            tree_method: GitWriteMethod::Native,
            commit_method: GitWriteMethod::Native,
            head_update_method: GitWriteMethod::Native,
            provider_label: "gix".to_string(),
            fallback_reason: None,
        },
        elapsed_ms: elapsed.as_millis() as u64,
    })
}

/// Fallback commit path using CLI (gix-hybrid-fallback).
/// All-or-nothing: uses CLI for tree write, gix for commit creation if possible,
/// or full CLI if needed.
pub fn fallback_commit(
    project_root: &Path,
    message: &str,
) -> Result<NativeWriteCommitResult, GitBridgeError> {
    let start = std::time::Instant::now();

    // Use existing hybrid path
    let hash = create_commit_native(project_root, message, "orqestra", "orqestra@local")?;

    let repo = gix::open(project_root)
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to open repo: {e}")))?;

    let parent_hashes = match repo.head_id() {
        Ok(id) => vec![id.detach().to_hex().to_string()],
        Err(_) => vec![],
    };

    Ok(NativeWriteCommitResult {
        hash,
        parent_hashes,
        diagnostic: CommitPathDiagnostic {
            tree_method: GitWriteMethod::CliFallback,
            commit_method: GitWriteMethod::Native,
            head_update_method: GitWriteMethod::Native,
            provider_label: "gix-hybrid-fallback".to_string(),
            fallback_reason: Some("write-tree uses CLI git write-tree".to_string()),
        },
        elapsed_ms: start.elapsed().as_millis() as u64,
    })
}
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

/// Get raw `git status --porcelain=v2` output for per-file parsing.
/// Used by the snapshot module to build changed-file lists.
pub fn git_status_porcelain_output(project_root: &Path) -> Result<String, GitBridgeError> {
    let output = std::process::Command::new("git")
        .current_dir(project_root)
        .args(["status", "--porcelain=v2", "-u"]) // -u: show individual untracked files
        .output()
        .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;

    if !output.status.success() {
        return Err(GitBridgeError::GitOperation(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ---------------------------------------------------------------------------
// Git Provider Diagnostics (v1.6.0)
//
// Canonical provider enum and per-operation diagnostics.
// Read-only diagnostics only — mutating operations are reported from
// a static registry, never executed during diagnostics.
// ---------------------------------------------------------------------------

/// Canonical provider label — enum-backed to prevent drift.
/// This is the sole source of provider strings; no ad-hoc labels allowed.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum GitProvider {
    Gix,
    GixHybrid,
    GitCliFallback,
    DeterministicHeuristic,
    NotImplemented,
}

/// Per-operation provider metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GitOperationProvider {
    pub operation: String,
    pub provider: GitProvider,
    pub native: bool,
    pub fallback_available: bool,
    pub read_only: bool,
    pub mutates_repository: bool,
    pub executed_in_diagnostics: bool,
    pub latency_ms: Option<u64>,  // None for mutating ops not executed
}

/// Full provider diagnostics report.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GitProviderReport {
    pub operations: Vec<GitOperationProvider>,
    pub snapshot_time: String,
    pub repository_valid: bool,
}

/// Static registry of all Git operations with their declared providers.
/// Mutating operations are listed but never executed during diagnostics.
fn static_provider_registry() -> Vec<GitOperationProvider> {
    vec![
        GitOperationProvider {
            operation: "head_hash".into(),
            provider: GitProvider::Gix,
            native: true,
            fallback_available: false,
            read_only: true,
            mutates_repository: false,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "branch_name".into(),
            provider: GitProvider::Gix,
            native: true,
            fallback_available: false,
            read_only: true,
            mutates_repository: false,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "recent_commits".into(),
            provider: GitProvider::Gix,
            native: true,
            fallback_available: true,
            read_only: true,
            mutates_repository: false,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "repository_snapshot".into(),
            provider: GitProvider::GixHybrid,
            native: false,
            fallback_available: true,
            read_only: true,
            mutates_repository: false,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "changed_file_summary".into(),
            provider: GitProvider::GixHybrid,
            native: false,
            fallback_available: true,
            read_only: true,
            mutates_repository: false,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "diff_stat".into(),
            provider: GitProvider::GitCliFallback,
            native: false,
            fallback_available: false,
            read_only: true,
            mutates_repository: false,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "safe_diff_context".into(),
            provider: GitProvider::GitCliFallback,
            native: false,
            fallback_available: false,
            read_only: true,
            mutates_repository: false,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "semantic_commit_prep".into(),
            provider: GitProvider::DeterministicHeuristic,
            native: false,
            fallback_available: false,
            read_only: true,
            mutates_repository: false,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        // Mutating operations — never executed in diagnostics
        GitOperationProvider {
            operation: "staging".into(),
            provider: GitProvider::GitCliFallback,
            native: false,
            fallback_available: false,
            read_only: false,
            mutates_repository: true,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "commit_creation".into(),
            provider: GitProvider::GixHybrid,
            native: false,
            fallback_available: false,
            read_only: false,
            mutates_repository: true,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "push".into(),
            provider: GitProvider::NotImplemented,
            native: false,
            fallback_available: false,
            read_only: false,
            mutates_repository: true,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "pull".into(),
            provider: GitProvider::NotImplemented,
            native: false,
            fallback_available: false,
            read_only: false,
            mutates_repository: true,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
        GitOperationProvider {
            operation: "merge".into(),
            provider: GitProvider::NotImplemented,
            native: false,
            fallback_available: false,
            read_only: false,
            mutates_repository: true,
            executed_in_diagnostics: false,
            latency_ms: None,
        },
    ]
}

/// Build a provider diagnostics report.
///
/// Executes only read-only operations to measure latency.
/// Mutating operations are reported from the static registry only
/// (executed_in_diagnostics: false, latency_ms: null).
pub fn build_provider_report(project_root: &Path) -> Result<GitProviderReport, GitBridgeError> {
    let snapshot_time = chrono::Utc::now().to_rfc3339();
    let mut registry = static_provider_registry();

    // Execute read-only operations and record latency
    for op in &mut registry {
        if op.mutates_repository {
            // Never execute mutating operations
            continue;
        }

        let start = std::time::Instant::now();
        let succeeded = match op.operation.as_str() {
            "head_hash" => get_head_hash(project_root).is_ok(),
            "branch_name" => gix::open(project_root)
                .ok()
                .and_then(|repo| repo.head_name().ok().flatten())
                .is_some(),
            "recent_commits" => crate::commits::recent_commits(project_root, Some(1)).is_ok(),
            "repository_snapshot" => crate::snapshot::repository_snapshot(project_root).is_ok(),
            "changed_file_summary" => {
                crate::snapshot::repository_snapshot(project_root)
                    .map(|s| !s.changed_files.is_empty() || true) // empty is valid
                    .unwrap_or(false)
            }
            "diff_stat" => crate::diff::diff_stat(project_root).is_ok(),
            "safe_diff_context" => {
                // Check env var — if not enabled, report as not executed but still available
                std::env::var("ORQESTRA_SAFE_DIFF_CONTEXT").is_ok()
            }
            "semantic_commit_prep" => crate::semantic_prep::prepare_semantic_commit(project_root).is_ok(),
            _ => false,
        };

        op.executed_in_diagnostics = true;
        op.latency_ms = Some(start.elapsed().as_millis() as u64);

        // If operation failed, report actual provider as cli-fallback if available
        if !succeeded && op.fallback_available {
            op.provider = GitProvider::GitCliFallback;
        }
    }

    Ok(GitProviderReport {
        operations: registry,
        snapshot_time,
        repository_valid: is_git_repo(project_root),
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
