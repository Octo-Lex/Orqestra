//! Onboarding state management (v2.5.3).
//!
//! Persists first-run wizard completion and project context in local app data.
//! Never stores secrets or raw credentials.
//!
//! Disk persistence:
//!   {app_data_dir}/app-state.json — atomic write (tmp + rename)
//!   Corrupt file → backed up as app-state.corrupt.{timestamp}.json → default state
//!
//! Boundaries:
//!   app-state.json may store local paths (private local state).
//!   Diagnostic bundles must hash/redact all paths.

use crate::commands::onboarding_types::*;
use serde::{Deserialize, Serialize};
use tauri::{command, Manager};

use super::roadmap::CommandError;

type CommandResult<T> = Result<T, CommandError>;

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

fn app_state_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_default()
        .join("app-state.json")
}

/// Load AppState from disk. On corruption: backup and return default.
fn load_app_state(app: &tauri::AppHandle) -> AppState {
    let path = app_state_path(app);

    if !path.exists() {
        return AppState::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            match serde_json::from_str::<AppState>(&content) {
                Ok(state) => state,
                Err(e) => {
                    tracing::warn!("Corrupt app-state.json: {}", e);
                    // Backup corrupt file
                    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
                    let backup = path.with_file_name(format!("app-state.corrupt.{}.json", timestamp));
                    let _ = std::fs::rename(&path, &backup);
                    tracing::info!("Corrupt file backed up to {}", backup.display());
                    AppState::default()
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to read app-state.json: {}", e);
            AppState::default()
        }
    }
}

/// Save AppState to disk with atomic write (tmp + rename).
fn save_app_state(app: &tauri::AppHandle, state: &AppState) -> CommandResult<()> {
    let path = app_state_path(app);

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CommandError {
            code: "STATE_DIR_ERROR",
            message: format!("Failed to create app data dir: {}", e),
        })?;
    }

    let json = serde_json::to_string_pretty(state).map_err(|e| CommandError {
        code: "STATE_SERIALIZE_ERROR",
        message: format!("Failed to serialize app state: {}", e),
    })?;

    // Atomic write: write to tmp, then rename
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, &json).map_err(|e| CommandError {
        code: "STATE_WRITE_ERROR",
        message: format!("Failed to write app state: {}", e),
    })?;

    std::fs::rename(&tmp_path, &path).map_err(|e| {
        // Clean up tmp on rename failure
        let _ = std::fs::remove_file(&tmp_path);
        CommandError {
            code: "STATE_RENAME_ERROR",
            message: format!("Failed to rename app state: {}", e),
        }
    })?;

    Ok(())
}

// ---------------------------------------------------------------------------
// State manager — holds AppState and AppHandle for persistence
// ---------------------------------------------------------------------------

pub struct OnboardingStateManager {
    state: std::sync::Mutex<Option<AppState>>,
}

impl Default for OnboardingStateManager {
    fn default() -> Self {
        Self {
            state: std::sync::Mutex::new(None), // Loaded lazily on first access
        }
    }
}

impl OnboardingStateManager {
    /// Get or load state. Loads from disk on first call.
    pub fn get_or_load(&self, app: &tauri::AppHandle) -> AppState {
        let mut guard = self.state.lock().unwrap();
        if guard.is_none() {
            *guard = Some(load_app_state(app));
        }
        guard.clone().unwrap()
    }

    /// Update state and persist to disk.
    pub fn update<F>(&self, app: &tauri::AppHandle, f: F) -> CommandResult<AppState>
    where
        F: FnOnce(&mut AppState),
    {
        let mut guard = self.state.lock().map_err(|e| CommandError {
            code: "STATE_LOCK_ERROR",
            message: format!("Failed to lock state: {}", e),
        })?;

        if guard.is_none() {
            *guard = Some(load_app_state(app));
        }

        let state = guard.as_mut().unwrap();
        f(state);
        save_app_state(app, state)?;
        Ok(state.clone())
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[command]
pub fn get_onboarding_state_cmd(
    app: tauri::AppHandle,
    manager: tauri::State<'_, OnboardingStateManager>,
) -> CommandResult<OnboardingStateResponse> {
    let state = manager.get_or_load(&app);
    Ok(OnboardingStateResponse {
        onboarding_completed: state.onboarding_completed,
        last_project_root: state.last_project_root.clone(),
        last_project_id: state.last_project_id.clone(),
        recent_projects: state.recent_projects.clone(),
        last_opened_at: state.last_opened_at.clone(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStateResponse {
    pub onboarding_completed: bool,
    pub last_project_root: Option<String>,
    pub last_project_id: Option<String>,
    pub recent_projects: Vec<ProjectRecord>,
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStateUpdate {
    pub onboarding_completed: Option<bool>,
    pub last_project_root: Option<String>,
}

#[command]
pub fn set_onboarding_state_cmd(
    app: tauri::AppHandle,
    update: OnboardingStateUpdate,
    manager: tauri::State<'_, OnboardingStateManager>,
) -> CommandResult<OnboardingStateResponse> {
    manager.update(&app, |state| {
        if let Some(completed) = update.onboarding_completed {
            state.onboarding_completed = completed;
        }
        if let Some(root) = update.last_project_root {
            state.last_project_root = Some(root.clone());
            state.last_project_id = Some(project_id_from_root(&root));
            state.last_opened_at = Some(chrono::Utc::now().to_rfc3339());

            // Update recent projects
            let pid = project_id_from_root(&root);
            let name = std::path::Path::new(&root)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            // Remove existing entry for this project
            state.recent_projects.retain(|p| p.project_id != pid);

            // Add to front
            state.recent_projects.insert(0, ProjectRecord {
                project_id: pid,
                root: root.clone(),
                name,
                last_opened_at: chrono::Utc::now().to_rfc3339(),
                last_known_credential_status: CredentialStatus::Unknown,
                last_known_relay_status: RelayConnectionStatus::Unknown,
            });

            // Cap at MAX_RECENT_PROJECTS
            state.recent_projects.truncate(MAX_RECENT_PROJECTS);
        }
    }).map(|state| OnboardingStateResponse {
        onboarding_completed: state.onboarding_completed,
        last_project_root: state.last_project_root,
        last_project_id: state.last_project_id,
        recent_projects: state.recent_projects,
        last_opened_at: state.last_opened_at,
    })
}

#[command]
pub fn reset_onboarding_cmd(
    app: tauri::AppHandle,
    request: ResetOnboardingRequest,
    manager: tauri::State<'_, OnboardingStateManager>,
) -> CommandResult<OnboardingStateResponse> {
    // NOTE: This command NEVER deletes OS-keychain credentials.
    // Credential deletion requires a separate explicit credential command.
    manager.update(&app, |state| {
        if request.clear_metadata {
            state.onboarding_completed = false;
            state.last_opened_at = None;
        }
        if request.clear_project_history {
            state.last_project_root = None;
            state.last_project_id = None;
            state.recent_projects.clear();
        }
    }).map(|state| OnboardingStateResponse {
        onboarding_completed: state.onboarding_completed,
        last_project_root: state.last_project_root,
        last_project_id: state.last_project_id,
        recent_projects: state.recent_projects,
        last_opened_at: state.last_opened_at,
    })
}

/// Record project access with updated status.
#[command]
pub fn record_project_access_cmd(
    app: tauri::AppHandle,
    root: String,
    credential_status: CredentialStatus,
    relay_status: RelayConnectionStatus,
    manager: tauri::State<'_, OnboardingStateManager>,
) -> CommandResult<ProjectRecord> {
    let pid = project_id_from_root(&root);
    let name = std::path::Path::new(&root)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let record = ProjectRecord {
        project_id: pid.clone(),
        root: root.clone(),
        name,
        last_opened_at: chrono::Utc::now().to_rfc3339(),
        last_known_credential_status: credential_status,
        last_known_relay_status: relay_status,
    };

    manager.update(&app, |state| {
        // Remove existing
        state.recent_projects.retain(|p| p.project_id != pid);
        // Add updated to front
        state.recent_projects.insert(0, record.clone());
        state.recent_projects.truncate(MAX_RECENT_PROJECTS);
        state.last_project_root = Some(root);
        state.last_project_id = Some(pid);
        state.last_opened_at = Some(record.last_opened_at.clone());
    })?;

    Ok(record)
}
