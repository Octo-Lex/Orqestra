//! Git sync commands for roadmap/ directory.
//!
//! v1.0.2: Uses OS-keychain-backed credential store internally.
//! The raw PAT is never returned to TypeScript — it's retrieved in Rust
//! and passed directly to the git operation.

use crate::commands::credentials;
use serde::Serialize;
use tauri::command;
use tauri_plugin_shell::ShellExt;

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct GitResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

// ---------------------------------------------------------------------------
use std::path::{Path, PathBuf};
// ---------------------------------------------------------------------------
// v2.5.1: RAII askpass guard — guaranteed cleanup
// ---------------------------------------------------------------------------

struct AskpassGuard {
    dir: PathBuf,
    path: PathBuf,
}

impl Drop for AskpassGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_dir(&self.dir);
    }
}

// ---------------------------------------------------------------------------
// Git helpers
// ---------------------------------------------------------------------------

/// Build a credential helper batch script for Windows.
/// v2.5.1: Uses unique temp directory, create_new, RAII cleanup.
/// Interim mitigation — v2.5.2+ should use credential helper / stdin-safe flow.
fn write_askpass_script(pat: &str) -> Result<AskpassGuard, String> {
    let tmp_root = std::env::temp_dir();
    let unique_id = uuid::Uuid::new_v4().to_string();
    let dir = tmp_root.join(format!("orqestra-askpass-{}", unique_id));
    std::fs::create_dir(&dir)
        .map_err(|e| format!("failed to create askpass dir: {}", e))?;

    let script_path = dir.join("askpass.bat");

    // Use OpenOptions with create_new to never overwrite existing file
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600); // Owner-only
    }
    let mut file = opts.open(&script_path)
        .map_err(|e| format!("failed to create askpass script: {}", e))?;
    std::io::Write::write_all(&mut file, format!("@echo {}\n", pat).as_bytes())
        .map_err(|e| format!("failed to write askpass script: {}", e))?;

    Ok(AskpassGuard { dir, path: script_path })
}

/// Run a git command in the project root with credential helper set.
async fn run_git(
    app: &tauri::AppHandle,
    project_root: &str,
    args: &[&str],
    pat: &str,
) -> Result<GitResult, String> {
    let askpass = write_askpass_script(pat)?;
    let askpass_path = askpass.path.clone(); // Copy path before moving guard

    let shell = app.shell();
    let cmd = shell
        .command("git")
        .args(args)
        .current_dir(PathBuf::from(project_root))
        .env("GIT_ASKPASS", &askpass_path)
        .env("GIT_TERMINAL_PROMPT", "0");

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("failed to execute git: {}", e))?;

    // RAII: askpass guard drops here, cleaning up temp dir and file

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(GitResult {
        success: output.status.success(),
        stdout,
        stderr,
    })
}

// ---------------------------------------------------------------------------
// Commands — PAT retrieved internally from OS keychain
// ---------------------------------------------------------------------------

/// Pull latest changes from origin.
#[command]
pub async fn git_pull_roadmap(
    app: tauri::AppHandle,
    project_root: String,
    _pat: String, // Ignored — PAT comes from keychain now
) -> Result<GitResult, String> {
    let pat = credentials::get_stored_pat(&app)?;
    run_git(&app, &project_root, &["pull", "origin", "HEAD"], &pat).await
}

/// Stage all changes in roadmap/, commit, and push to origin.
#[command]
pub async fn git_push_roadmap(
    app: tauri::AppHandle,
    project_root: String,
    _pat: String, // Ignored — PAT comes from keychain now
) -> Result<GitResult, String> {
    let pat = credentials::get_stored_pat(&app)?;

    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let message = format!("orqestra: sync roadmap [{}]", timestamp);

    // Stage roadmap/ changes
    let stage_result = run_git(&app, &project_root, &["add", "roadmap/"], &pat).await?;
    if !stage_result.success {
        return Ok(stage_result);
    }

    // Commit
    let commit_result = run_git(
        &app,
        &project_root,
        &["commit", "-m", &message],
        &pat,
    )
    .await?;

    // "nothing to commit" is not an error
    if !commit_result.success {
        let combined = format!("{}{}", commit_result.stdout, commit_result.stderr);
        if combined.contains("nothing to commit") {
            return Ok(GitResult {
                success: true,
                stdout: "Nothing to commit — already up to date.".into(),
                stderr: String::new(),
            });
        }
        return Ok(commit_result);
    }

    // Push
    run_git(&app, &project_root, &["push", "origin", "HEAD"], &pat).await
}

// ---------------------------------------------------------------------------
// Native Git Status (v1.1.0 pilot)
// ---------------------------------------------------------------------------

/// Get git status using native gix with CLI fallback.
/// Read-only — never modifies the repository.
#[command]
pub fn git_status_cmd(project_root: String) -> Result<String, String> {
    use git_bridge::NativeGitStatus;
    let path = std::path::PathBuf::from(&project_root);
    let status = git_bridge::native_git_status(&path)
        .map_err(|e| format!("Git status failed: {e}"))?;
    serde_json::to_string(&status)
        .map_err(|e| format!("Failed to serialize status: {e}"))
}

// ---------------------------------------------------------------------------
// v1.2.0: Native Git Operations (read-only)
// ---------------------------------------------------------------------------

/// Get a repository snapshot — branch, HEAD, status, changed files.
/// Read-only — never modifies the repository.
#[command]
pub fn git_repository_snapshot_cmd(project_root: String) -> Result<String, String> {
    let path = std::path::PathBuf::from(&project_root);
    let snapshot = git_bridge::repository_snapshot(&path)
        .map_err(|e| format!("Repository snapshot failed: {e}"))?;
    serde_json::to_string(&snapshot)
        .map_err(|e| format!("Failed to serialize snapshot: {e}"))
}

/// Get recent commit metadata.
/// Read-only — bounded to 100 max.
#[command]
pub fn git_recent_commits_cmd(project_root: String, limit: Option<usize>) -> Result<String, String> {
    let path = std::path::PathBuf::from(&project_root);
    let commits = git_bridge::recent_commits(&path, limit)
        .map_err(|e| format!("Recent commits failed: {e}"))?;
    serde_json::to_string(&commits)
        .map_err(|e| format!("Failed to serialize commits: {e}"))
}

/// Get diff/stat summary.
/// Read-only — CLI-backed, never exposes file contents.
#[command]
pub fn git_diff_stat_cmd(project_root: String) -> Result<String, String> {
    let path = std::path::PathBuf::from(&project_root);
    let stat = git_bridge::diff_stat(&path)
        .map_err(|e| format!("Diff stat failed: {e}"))?;
    serde_json::to_string(&stat)
        .map_err(|e| format!("Failed to serialize diff stat: {e}"))
}

// ---------------------------------------------------------------------------
// v1.3.0: Semantic Commit Preparation (proposal-only)
// ---------------------------------------------------------------------------

/// Prepare a semantic commit proposal.
/// Read-only — never stages files, creates commits, or mutates the repository.
#[command]
pub fn prepare_semantic_commit_cmd(project_root: String) -> Result<String, String> {
    let path = std::path::PathBuf::from(&project_root);
    let proposal = git_bridge::prepare_semantic_commit(&path)
        .map_err(|e| format!("Semantic commit preparation failed: {e}"))?;
    serde_json::to_string(&proposal)
        .map_err(|e| format!("Failed to serialize proposal: {e}"))
}

// ---------------------------------------------------------------------------
// v1.6.0: Git Provider Diagnostics
// ---------------------------------------------------------------------------

/// Get per-operation Git provider diagnostics.
/// Read-only — never mutates the repository.
#[command]
pub fn git_provider_diagnostics_cmd(project_root: String) -> Result<String, String> {
    let path = std::path::PathBuf::from(&project_root);
    let report = git_bridge::build_provider_report(&path)
        .map_err(|e| format!("Provider diagnostics failed: {e}"))?;
    serde_json::to_string(&report)
        .map_err(|e| format!("Failed to serialize provider report: {e}"))
}

/// Get recent commits with provider metadata.
/// Response wrapper carries provider even when list is empty.
#[command]
pub fn git_recent_commits_with_provider_cmd(project_root: String, limit: Option<usize>) -> Result<String, String> {
    let path = std::path::PathBuf::from(&project_root);
    let result = git_bridge::recent_commits_with_provider(&path, limit)
        .map_err(|e| format!("Recent commits with provider failed: {e}"))?;
    serde_json::to_string(&result)
        .map_err(|e| format!("Failed to serialize result: {e}"))
}

/// Get diff/stat with provider metadata.
#[command]
pub fn git_diff_stat_with_provider_cmd(project_root: String) -> Result<String, String> {
    let path = std::path::PathBuf::from(&project_root);
    let result = git_bridge::diff_stat_with_provider(&path)
        .map_err(|e| format!("Diff stat with provider failed: {e}"))?;
    serde_json::to_string(&result)
        .map_err(|e| format!("Failed to serialize result: {e}"))
}
