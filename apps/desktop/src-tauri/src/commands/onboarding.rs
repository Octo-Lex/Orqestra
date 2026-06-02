//! Onboarding state management.
//!
//! Persists first-run wizard completion in local app config.
//! Never stores secrets or raw credentials.

use serde::{Deserialize, Serialize};
use tauri::command;

use super::roadmap::CommandError;

type CommandResult<T> = Result<T, CommandError>;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingState {
    pub onboarding_completed: bool,
    pub last_project_root: Option<String>,
    pub last_readiness_check: Option<String>,
    pub chosen_path: Option<String>,
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            onboarding_completed: false,
            last_project_root: None,
            last_readiness_check: None,
            chosen_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStateUpdate {
    pub onboarding_completed: Option<bool>,
    pub last_project_root: Option<String>,
    pub last_readiness_check: Option<String>,
    pub chosen_path: Option<String>,
}

// ---------------------------------------------------------------------------
// State storage — managed via Tauri managed state
// ---------------------------------------------------------------------------

pub struct OnboardingStateManager {
    state: std::sync::Mutex<OnboardingState>,
}

impl Default for OnboardingStateManager {
    fn default() -> Self {
        Self {
            state: std::sync::Mutex::new(OnboardingState::default()),
        }
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[command]
pub fn get_onboarding_state_cmd(
    manager: tauri::State<'_, OnboardingStateManager>,
) -> CommandResult<OnboardingState> {
    let state = manager.state.lock().map_err(|e| CommandError {
        code: "STATE_LOCK_ERROR",
        message: format!("Failed to read onboarding state: {}", e),
    })?;
    Ok(state.clone())
}

#[command]
pub fn set_onboarding_state_cmd(
    update: OnboardingStateUpdate,
    manager: tauri::State<'_, OnboardingStateManager>,
) -> CommandResult<OnboardingState> {
    let mut state = manager.state.lock().map_err(|e| CommandError {
        code: "STATE_LOCK_ERROR",
        message: format!("Failed to write onboarding state: {}", e),
    })?;

    if let Some(completed) = update.onboarding_completed {
        state.onboarding_completed = completed;
    }
    if let Some(root) = update.last_project_root {
        state.last_project_root = Some(root);
    }
    if let Some(check) = update.last_readiness_check {
        state.last_readiness_check = Some(check);
    }
    if let Some(path) = update.chosen_path {
        state.chosen_path = Some(path);
    }

    Ok(state.clone())
}

#[command]
pub fn reset_onboarding_cmd(
    manager: tauri::State<'_, OnboardingStateManager>,
) -> CommandResult<OnboardingState> {
    let mut state = manager.state.lock().map_err(|e| CommandError {
        code: "STATE_LOCK_ERROR",
        message: format!("Failed to reset onboarding state: {}", e),
    })?;
    *state = OnboardingState::default();
    Ok(state.clone())
}
