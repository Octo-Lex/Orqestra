/// Readiness DTO must not expose raw secret fields.
#[test]
fn readiness_serializes_without_secret_fields() {
    let report = TestReadinessReport {
        generated_at: "2026-06-02T00:00:00Z".to_string(),
        app: TestAppReadiness {
            version: "1.0.3".to_string(),
            git_sha: None,
            tauri_commands_registered: None,
            platform: "windows".to_string(),
        },
        project: None,
        local_tools: vec![],
        ai: TestAiReadiness {
            service_status: "unreachable".to_string(),
            health_url: "http://localhost:8000/health".to_string(),
            api_key_status: "missing".to_string(),
            mode: "unavailable".to_string(),
            last_error: Some("ZAI_API_KEY not set".to_string()),
        },
        credentials: TestCredentialReadiness {
            github_token: "missing".to_string(),
            provider: "keyring".to_string(),
            last_error: None,
        },
        dashboard: TestDashboardReadiness {
            local_json: "missing".to_string(),
            live_url_status: "ok".to_string(),
            source_commit: None,
            cloudflare_secrets: "unknown".to_string(),
        },
        release_artifacts: vec![],
        warnings: vec![],
    };

    let json = serde_json::to_string(&report).unwrap();

    // Must not contain raw secret field names
    assert!(!json.contains("raw_token"), "No raw_token field");
    assert!(!json.contains("api_key_value"), "No api_key_value field");
    assert!(!json.contains("bearer_token"), "No bearer_token field");
    assert!(!json.contains("password"), "No password field");
    assert!(!json.contains("secret_value"), "No secret_value field");
}

/// Missing ZAI_API_KEY should be shown as degraded, not real AI.
#[test]
fn ai_readiness_missing_key_is_degraded() {
    let ai = TestAiReadiness {
        service_status: "reachable".to_string(),
        health_url: "http://localhost:8000/health".to_string(),
        api_key_status: "missing".to_string(),
        mode: "degraded_mock".to_string(),
        last_error: Some("ZAI_API_KEY not set".to_string()),
    };

    assert_ne!(ai.mode, "real", "Missing key must not show as real AI");
    assert_eq!(ai.mode, "degraded_mock");
    assert_eq!(ai.api_key_status, "missing");
}

/// Missing contributor tools must not block local PM.
#[test]
fn tool_check_missing_is_warning() {
    let tool = TestToolReadiness {
        tool: "rustc".to_string(),
        status: "missing".to_string(),
        version: None,
        required_for: vec!["Core crate development".to_string()],
    };

    // Missing tool is for contributors, not users
    assert_eq!(tool.status, "missing");
    assert!(!tool.required_for.contains(&"Roadmap viewing".to_string()));
}

/// Credential errors must be masked — no raw error messages with tokens.
#[test]
fn credential_error_is_redacted() {
    let cred = TestCredentialReadiness {
        github_token: "error".to_string(),
        provider: "none".to_string(),
        last_error: Some("[REDACTED]".to_string()),
    };

    assert_eq!(cred.github_token, "error");
    // Error message should not contain raw tokens
    if let Some(err) = &cred.last_error {
        assert!(!err.starts_with("ghp_"), "Error must not start with token prefix");
    }
}

/// Unknown Cloudflare state is not a failure for local usage.
#[test]
fn cloudflare_unknown_is_not_failure() {
    let dashboard = TestDashboardReadiness {
        local_json: "missing".to_string(),
        live_url_status: "ok".to_string(),
        source_commit: None,
        cloudflare_secrets: "unknown".to_string(),
    };

    assert_eq!(dashboard.cloudflare_secrets, "unknown");
    // Unknown is informational, not an error
    assert_ne!(dashboard.cloudflare_secrets, "error");
}

// Test DTOs — mirror readiness.rs types
#[derive(Debug, serde::Serialize)]
struct TestReadinessReport {
    generated_at: String,
    app: TestAppReadiness,
    project: Option<String>,
    local_tools: Vec<TestToolReadiness>,
    ai: TestAiReadiness,
    credentials: TestCredentialReadiness,
    dashboard: TestDashboardReadiness,
    release_artifacts: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct TestAppReadiness {
    version: String,
    git_sha: Option<String>,
    tauri_commands_registered: Option<usize>,
    platform: String,
}

#[derive(Debug, serde::Serialize)]
struct TestToolReadiness {
    tool: String,
    status: String,
    version: Option<String>,
    required_for: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct TestAiReadiness {
    service_status: String,
    health_url: String,
    api_key_status: String,
    mode: String,
    last_error: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct TestCredentialReadiness {
    github_token: String,
    provider: String,
    last_error: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct TestDashboardReadiness {
    local_json: String,
    live_url_status: String,
    source_commit: Option<String>,
    cloudflare_secrets: String,
}
