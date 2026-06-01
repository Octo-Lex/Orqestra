//! Git sync commands for roadmap/ directory.
//!
//! Shells out to the system `git` binary via tauri-plugin-shell.
//! This is intentionally NOT a Rust git implementation — git-bridge
//! (Phase 1) will handle that.

use serde::Serialize;
use std::path::PathBuf;
use tauri::command;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::StoreExt;

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
// PAT management
// ---------------------------------------------------------------------------

/// Store the GitHub PAT in the app's credential store.
///
/// SECURITY NOTE: tauri-plugin-store writes to a JSON file on disk.
/// The PAT is stored as plaintext. Replace with tauri-plugin-stronghold
/// for OS-level encrypted storage before production use.
#[command]
pub fn store_pat(app: tauri::AppHandle, pat: String) -> Result<(), String> {
    let store = app
        .store("credentials.json")
        .map_err(|e| format!("failed to open store: {}", e))?;
    store.set("github_pat", serde_json::Value::String(pat));
    store
        .save()
        .map_err(|e| format!("failed to save store: {}", e))?;
    Ok(())
}

/// Retrieve the stored GitHub PAT.
fn get_pat(app: &tauri::AppHandle) -> Result<String, String> {
    let store = app
        .store("credentials.json")
        .map_err(|e| format!("failed to open store: {}", e))?;
    let val = store
        .get("github_pat")
        .ok_or("GitHub PAT not configured. Save it in Settings.")?;
    val.as_str()
        .map(|s| s.to_string())
        .ok_or("stored PAT is not a string".into())
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
) -> Result<GitResult, String> {
    let pat = get_pat(app)?;
    let askpass = write_askpass_script(&pat)?;

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
// Commands
// ---------------------------------------------------------------------------

/// Pull latest changes from origin.
#[command]
pub async fn git_pull_roadmap(
    app: tauri::AppHandle,
    project_root: String,
) -> Result<GitResult, String> {
    run_git(&app, &project_root, &["pull", "origin", "HEAD"]).await
}

/// Stage all changes in roadmap/, commit, and push to origin.
#[command]
pub async fn git_push_roadmap(
    app: tauri::AppHandle,
    project_root: String,
) -> Result<GitResult, String> {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let message = format!("orqestra: sync roadmap [{}]", timestamp);

    // Stage roadmap/ changes
    let stage_result = run_git(&app, &project_root, &["add", "roadmap/"]).await?;
    if !stage_result.success {
        return Ok(stage_result);
    }

    // Commit
    let commit_result = run_git(
        &app,
        &project_root,
        &["commit", "-m", &message],
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
    run_git(&app, &project_root, &["push", "origin", "HEAD"]).await
}

/// Check whether a GitHub PAT is stored.
#[command]
pub fn has_stored_pat(app: tauri::AppHandle) -> Result<bool, String> {
    match get_pat(&app) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
