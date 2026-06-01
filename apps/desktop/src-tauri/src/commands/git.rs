//! Git sync commands for roadmap/ directory.
//!
//! Shells out to the system `git` binary via tauri-plugin-shell.
//! This is intentionally NOT a Rust git implementation — git-bridge
//! (Phase 1) will handle that.

use serde::Serialize;
use std::path::PathBuf;
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
// Git helpers
// ---------------------------------------------------------------------------

/// Build a credential helper batch script for Windows.
/// Writes a .bat file that echoes the PAT when git asks for a password.
fn write_askpass_script(pat: &str) -> Result<PathBuf, String> {
    let tmp_dir = std::env::temp_dir();
    let script_path = tmp_dir.join("orqestra-git-askpass.bat");
    std::fs::write(&script_path, format!("@echo {}\n", pat))
        .map_err(|e| format!("failed to write askpass script: {}", e))?;
    Ok(script_path)
}

/// Run a git command in the project root with credential helper set.
async fn run_git(
    app: &tauri::AppHandle,
    project_root: &str,
    args: &[&str],
    pat: &str,
) -> Result<GitResult, String> {
    let askpass = write_askpass_script(pat)?;

    let shell = app.shell();
    let cmd = shell
        .command("git")
        .args(args)
        .current_dir(PathBuf::from(project_root))
        .env("GIT_ASKPASS", &askpass)
        .env("GIT_TERMINAL_PROMPT", "0");

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("failed to execute git: {}", e))?;

    // Clean up askpass script
    let _ = std::fs::remove_file(&askpass);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(GitResult {
        success: output.status.success(),
        stdout,
        stderr,
    })
}

// ---------------------------------------------------------------------------
// Commands — PAT is passed from TypeScript, never stored in Rust
// ---------------------------------------------------------------------------

/// Pull latest changes from origin.
#[command]
pub async fn git_pull_roadmap(
    app: tauri::AppHandle,
    project_root: String,
    pat: String,
) -> Result<GitResult, String> {
    run_git(&app, &project_root, &["pull", "origin", "HEAD"], &pat).await
}

/// Stage all changes in roadmap/, commit, and push to origin.
#[command]
pub async fn git_push_roadmap(
    app: tauri::AppHandle,
    project_root: String,
    pat: String,
) -> Result<GitResult, String> {
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

    // "nothing to commit" is not an error — nothing new to push
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
