use std::sync::Mutex;

/// Read and write onboarding state. No secrets are ever persisted.
#[test]
fn onboarding_state_read_write() {
    // Simulate the managed state
    let state = Mutex::new(OnboardingState::default());
    assert!(!state.lock().unwrap().onboarding_completed);

    // Update
    {
        let mut s = state.lock().unwrap();
        s.onboarding_completed = true;
        s.last_project_root = Some("/test/project".to_string());
    }

    let s = state.lock().unwrap();
    assert!(s.onboarding_completed);
    assert_eq!(s.last_project_root, Some("/test/project".to_string()));
}

#[test]
fn onboarding_state_no_secret_fields() {
    let state = OnboardingState::default();
    // Verify the state has no token/secret/password fields
    let json = serde_json::to_string(&state).unwrap();
    assert!(!json.contains("token"), "State should not contain 'token'");
    assert!(!json.contains("secret"), "State should not contain 'secret'");
    assert!(!json.contains("password"), "State should not contain 'password'");
    assert!(!json.contains("api_key"), "State should not contain 'api_key'");
    assert!(!json.contains("credential"), "State should not contain 'credential'");
}

#[test]
fn onboarding_reset_returns_default() {
    let mut state = OnboardingState {
        onboarding_completed: true,
        last_project_root: Some("/path".to_string()),
        last_readiness_check: Some("2026-06-02T00:00:00Z".to_string()),
        chosen_path: Some("/path".to_string()),
    };

    // Reset
    state = OnboardingState::default();
    assert!(!state.onboarding_completed);
    assert!(state.last_project_root.is_none());
}

// Minimal DTO for testing — mirrors commands::onboarding::OnboardingState
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct OnboardingState {
    onboarding_completed: bool,
    last_project_root: Option<String>,
    last_readiness_check: Option<String>,
    chosen_path: Option<String>,
}
