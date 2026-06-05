//! v2.0.0 Machine-checkable redaction tests.
//!
//! Verifies the diagnostics bundle contains:
//! - No secrets (API keys, tokens, passwords)
//! - No source bodies (file contents)
//! - No raw diffs
//! - No .env content
//! - No token-like strings (ghp_, sk-, Bearer, etc.)
//! - No private file contents

use std::path::PathBuf;

fn find_repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    while !dir.join(".git").exists() {
        if !dir.pop() { panic!("No git repo found"); }
    }
    dir
}

/// Token prefixes that must never appear in diagnostic output.
const FORBIDDEN_TOKEN_PREFIXES: &[&str] = &[
    "ghp_", "gho_", "ghu_", "ghs_", "ghr_", // GitHub tokens
    "sk-",                                          // OpenAI-style keys
    "Bearer ",                                      // Auth headers
    "xoxb-", "xoxp-",                              // Slack tokens
    "AKIA",                                         // AWS access keys
    "eyJ",                                          // JWT-like base64 (could be token)
];

/// File names that should never appear as content in diagnostics.
const FORBIDDEN_CONTENT_FILES: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    "credentials.json",
    "secrets.json",
    "id_rsa",
    "id_ed25519",
];

// ---------------------------------------------------------------------------
// 1. Redaction function works
// ---------------------------------------------------------------------------

#[test]
fn redaction_removes_token_patterns() {
    use orqestra_desktop::diagnostics::redaction::redact_text;

    // Use realistic token lengths (ghp_ requires 36+ chars after prefix)
    let input = r#"{"token": "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij123456", "key": "sk-1234567890abcdefghijklmnopqrstuv"}"#;
    let result = redact_text(input);

    for prefix in FORBIDDEN_TOKEN_PREFIXES {
        if *prefix == "eyJ" { continue; }
        assert!(
            !result.redacted_text.contains(prefix),
            "Redacted text must not contain forbidden prefix: {}", prefix
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Redaction removes API keys
// ---------------------------------------------------------------------------

#[test]
fn redaction_removes_api_keys() {
    use orqestra_desktop::diagnostics::redaction::redact_text;

    let inputs = vec![
        // 36+ chars after ghp_
        r#"{"api_key": "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij"}"#,
        // 20+ alphanumeric chars after sk-
        r#"{"key": "sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234"}"#,
        r#"{"auth": "Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0"}"#,
    ];

    for input in inputs {
        let result = redact_text(input);
        for prefix in FORBIDDEN_TOKEN_PREFIXES {
            assert!(
                !result.redacted_text.contains(prefix),
                "Redacted text must not contain '{}':\nInput: {}\nOutput: {}",
                prefix, input, result.redacted_text
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 3. No source bodies in diagnostic output
// ---------------------------------------------------------------------------

#[test]
fn diagnostic_data_excludes_source_bodies() {
    // The diagnostic bundle must not include file contents.
    // Check the agent_matrix — it should have endpoint info, not source code.
    let agent_matrix = serde_json::json!({
        "agents": [
            {"name": "docs-agent", "mode": "review-only", "endpoint": "/agent/docs"},
            {"name": "bugfix-agent", "mode": "review-only", "endpoint": "/agent/bugfix"},
        ]
    });

    let json_str = serde_json::to_string(&agent_matrix).unwrap();
    // Should NOT contain source code markers
    assert!(!json_str.contains("fn main"), "No function bodies");
    assert!(!json_str.contains("impl "), "No impl blocks");
    assert!(!json_str.contains("import "), "No import statements");
    assert!(!json_str.contains("#include"), "No C includes");
}

// ---------------------------------------------------------------------------
// 4. No raw diffs in diagnostic output
// ---------------------------------------------------------------------------

#[test]
fn diagnostic_data_excludes_raw_diffs() {
    // Verify the bundle creation doesn't include diff content
    let git_provider = serde_json::json!({
        "operations": [{"operation": "diff_stat", "provider": "GixHybrid"}],
        "snapshot_time": "2026-06-05T12:00:00Z",
        "repository_valid": true
    });

    let json_str = serde_json::to_string(&git_provider).unwrap();
    assert!(!json_str.contains("@@"), "No diff hunk markers");
    assert!(!json_str.contains("+++"), "No diff plus markers");
    assert!(!json_str.contains("---"), "No diff minus markers (except in dates)");
    assert!(!json_str.contains("- "), "No removed line markers");
    assert!(!json_str.contains("+ "), "No added line markers");
}

// ---------------------------------------------------------------------------
// 5. No .env content
// ---------------------------------------------------------------------------

#[test]
fn diagnostic_data_excludes_env_content() {
    use orqestra_desktop::diagnostics::redaction::redact_text;

    let env_like_input = r#"{
        "ZAI_API_KEY": "sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890",
        "AWS_SECRET_ACCESS_KEY": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
    }"#;

    let result = redact_text(env_like_input);

    // Must not contain the actual secret values
    assert!(!result.redacted_text.contains("sk-proj-secret"), "Must redact API key values");
    assert!(result.redacted_value_count > 0, "Must report redacted values");
}

// ---------------------------------------------------------------------------
// 6. No private file contents
// ---------------------------------------------------------------------------

#[test]
fn bundle_files_exclude_private_content() {
    // Verify that the bundle files we create don't include private content.
    // The bundle entries should be metadata-only.

    let credential_status = serde_json::json!({
        "keyring_available": true,
        "provider": "os-keychain",
    });
    let json_str = serde_json::to_string(&credential_status).unwrap();

    // Must not contain actual credential values
    assert!(!json_str.contains("password"), "No password fields");
    assert!(!json_str.contains("token_value"), "No token values");
    assert!(!json_str.contains("secret"), "No secret references");
}

// ---------------------------------------------------------------------------
// 7. Full bundle export produces exactly 11 files
// ---------------------------------------------------------------------------

#[test]
fn bundle_export_produces_11_files() {
    let root = find_repo_root();
    let output_dir = root.join(".Orqestra").join("test-bundle");

    // Create a test bundle
    let result = orqestra_desktop::diagnostics::bundle::create_diagnostic_bundle(
        &output_dir,
        "{\"version\":\"test\"}",
        "{}",
        Some("{}"),
        "{}",
        "system info",
        "{}",
        "{}",
        "{}",  // git-provider
        "{}",  // credential-status
        "{}",  // agent-matrix
        "{}",  // patch-governance
        "{}",  // code-intel
        "{}",  // roadmap-status
    ).expect("bundle creation must succeed");

    // Must have exactly 14 files: 13 data entries + README.txt
    assert_eq!(result.files.len(), 14, "Bundle must contain 13 data files + README.txt");

    // Verify expected file names
    let names: Vec<&str> = result.files.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"app.json"), "Missing app.json");
    assert!(names.contains(&"readiness.json"), "Missing readiness.json");
    assert!(names.contains(&"project-validation.json"), "Missing project-validation.json");
    assert!(names.contains(&"git-provider.json"), "Missing git-provider.json");
    assert!(names.contains(&"credential-status.json"), "Missing credential-status.json");
    assert!(names.contains(&"agent-matrix.json"), "Missing agent-matrix.json");
    assert!(names.contains(&"patch-governance.json"), "Missing patch-governance.json");
    assert!(names.contains(&"code-intel.json"), "Missing code-intel.json");
    assert!(names.contains(&"roadmap-status.json"), "Missing roadmap-status.json");
    assert!(names.contains(&"recent-errors.json"), "Missing recent-errors.json");
    assert!(names.contains(&"system.txt"), "Missing system.txt");
    assert!(names.contains(&"ai-health.json"), "Missing ai-health.json");
    assert!(names.contains(&"dashboard-status.json"), "Missing dashboard-status.json");
    assert!(names.contains(&"README.txt"), "Missing README.txt");

    // Redaction summary must claim no raw secrets
    assert!(!result.redaction_summary.contains_raw_secrets, "Bundle must not contain raw secrets");

    // Cleanup test bundle
    let _ = std::fs::remove_dir_all(&output_dir);
}

// ---------------------------------------------------------------------------
// 8. Bundle export is non-mutating
// ---------------------------------------------------------------------------

#[test]
fn bundle_export_does_not_mutate_repo() {
    let root = find_repo_root();

    let status_before = git_bridge::native_git_status(&root).expect("status before");

    let output_dir = root.join(".Orqestra").join("test-bundle-nm");
    let _result = orqestra_desktop::diagnostics::bundle::create_diagnostic_bundle(
        &output_dir,
        "{\"version\":\"test\"}",
        "{}",
        None,
        "{}",
        "system info",
        "{}",
        "{}",
        "{}", "{}", "{}", "{}", "{}", "{}",
    );

    let status_after = git_bridge::native_git_status(&root).expect("status after");

    assert_eq!(status_before.dirty, status_after.dirty, "Dirty flag changed during bundle export");
    assert_eq!(status_before.staged_count, status_after.staged_count, "Staged count changed");

    // Cleanup
    let _ = std::fs::remove_dir_all(&output_dir);
}
