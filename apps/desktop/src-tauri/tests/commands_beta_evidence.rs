//! v2.12.0 — External Beta Evidence Intake Tests
//!
//! Tests for consent gating, redaction, session outcome, failure taxonomy,
//! and evidence bundle structure.

use serde_json;
use tempfile::TempDir;
use std::fs;
use std::path::PathBuf;

/// Helper: create a temp repo for testing.
fn create_temp_repo() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let git_dir = dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    // Initialize a real git repo
    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir.path())
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir.path())
        .output();
    fs::write(dir.path().join("initial.txt"), "test").expect("write");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(dir.path())
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(dir.path())
        .output();

    let path = dir.path().to_path_buf();
    (dir, path)
}

// ---------------------------------------------------------------------------
// Consent gating
// ---------------------------------------------------------------------------

#[test]
fn export_beta_evidence_requires_consent() {
    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        None,  // No consent
        None,
        None,
        None,
    ).expect("command should not error");

    assert!(!result.ok, "should fail without consent");
    assert_eq!(result.code, Some("BETA_EVIDENCE_CONSENT_REQUIRED".to_string()));
    assert!(result.path.is_none(), "should not create bundle");
}

#[test]
fn export_beta_evidence_requires_acknowledged_redaction() {
    let consent = orqestra_desktop::commands::beta_evidence::BetaEvidenceConsent {
        explicit: true,
        timestamp: chrono::Utc::now().to_rfc3339(),
        user_acknowledged_redaction: false,  // NOT acknowledged
        user_acknowledged_local_only: true,
    };

    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(consent),
        None,
        None,
        None,
    ).expect("command should not error");

    assert!(!result.ok, "should fail without redaction acknowledgement");
    assert_eq!(result.code, Some("BETA_EVIDENCE_CONSENT_REQUIRED".to_string()));
}

#[test]
fn export_beta_evidence_requires_acknowledged_local_only() {
    let consent = orqestra_desktop::commands::beta_evidence::BetaEvidenceConsent {
        explicit: true,
        timestamp: chrono::Utc::now().to_rfc3339(),
        user_acknowledged_redaction: true,
        user_acknowledged_local_only: false,  // NOT acknowledged
    };

    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(consent),
        None,
        None,
        None,
    ).expect("command should not error");

    assert!(!result.ok, "should fail without local-only acknowledgement");
}

// ---------------------------------------------------------------------------
// Bundle creation
// ---------------------------------------------------------------------------

fn make_valid_consent() -> orqestra_desktop::commands::beta_evidence::BetaEvidenceConsent {
    orqestra_desktop::commands::beta_evidence::BetaEvidenceConsent {
        explicit: true,
        timestamp: chrono::Utc::now().to_rfc3339(),
        user_acknowledged_redaction: true,
        user_acknowledged_local_only: true,
    }
}

#[test]
fn export_beta_evidence_creates_manifest() {
    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(make_valid_consent()),
        None,
        None,
        None,
    ).expect("command should succeed");

    assert!(result.ok, "should succeed with consent");
    assert!(result.path.is_some(), "should return path");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let manifest_path = bundle_path.join("beta-evidence-manifest.json");
    assert!(manifest_path.exists(), "manifest should exist");

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("read")).expect("parse");

    assert_eq!(manifest["uploaded_automatically"], false);
    assert_eq!(manifest["bundle_type"], "external-beta-evidence");
    assert_eq!(manifest["collection_mode"], "local_export");
    assert_eq!(manifest["consent"]["explicit"], true);

    // Cleanup
    let _ = fs::remove_dir_all(&bundle_path);
}

#[test]
fn export_beta_evidence_manifest_says_no_auto_upload() {
    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(make_valid_consent()),
        None,
        None,
        None,
    ).expect("ok");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(bundle_path.join("beta-evidence-manifest.json")).expect("read")
    ).expect("parse");

    assert_eq!(manifest["uploaded_automatically"], false,
        "manifest must always say uploaded_automatically: false");

    let _ = fs::remove_dir_all(&bundle_path);
}

#[test]
fn export_beta_evidence_includes_session_outcome() {
    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(make_valid_consent()),
        None,
        None,
        None,
    ).expect("ok");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let outcome: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(bundle_path.join("beta-session-outcome.json")).expect("read")
    ).expect("parse");

    assert_eq!(outcome["schema_version"], 1);
    assert!(outcome["session_id"].is_string());
    assert!(outcome["outcome"].is_string());
    assert!(outcome["steps"].is_object());
    assert!(outcome["platform"].is_object());

    let _ = fs::remove_dir_all(&bundle_path);
}

#[test]
fn export_beta_evidence_includes_failure_taxonomy() {
    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(make_valid_consent()),
        None,
        None,
        None,
    ).expect("ok");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let taxonomy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(bundle_path.join("beta-failure-taxonomy.json")).expect("read")
    ).expect("parse");

    assert_eq!(taxonomy["schema_version"], 1);
    assert!(taxonomy["observed_failures"].is_array());
    assert!(taxonomy["taxonomy"].is_array());

    let _ = fs::remove_dir_all(&bundle_path);
}

// ---------------------------------------------------------------------------
// Session outcome logic
// ---------------------------------------------------------------------------

#[test]
fn session_completed_with_warnings_when_ai_unavailable() {
    let steps = orqestra_desktop::commands::beta_evidence::SessionSteps {
        app_launched: true,
        repo_opened: true,
        roadmap_detected: true,
        pm_views_rendered: true,
        readiness_reviewed: true,
        ai_service_available: false,
        agent_flow_completed: false,
        ai_degraded_mode_understood: true,
        dashboard_evidence_viewed: true,
        diagnostics_exported: true,
    };

    let failures = vec![orqestra_desktop::commands::beta_evidence::FailureEntry {
        code: "AI_SERVICE_UNAVAILABLE".to_string(),
        severity: "warning".to_string(),
        category: "ai_service".to_string(),
        user_recoverable: true,
        blocked_steps: vec!["agent_flow_completed".to_string()],
    }];

    let outcome = orqestra_desktop::commands::beta_evidence::compute_session_outcome(&steps, &failures);
    assert_eq!(outcome, "completed_with_warnings",
        "AI unavailable should produce completed_with_warnings, not blocked");
}

#[test]
fn session_blocked_when_critical_failure() {
    let steps = orqestra_desktop::commands::beta_evidence::SessionSteps {
        app_launched: true,
        repo_opened: false,
        roadmap_detected: false,
        pm_views_rendered: false,
        readiness_reviewed: false,
        ai_service_available: false,
        agent_flow_completed: false,
        ai_degraded_mode_understood: false,
        dashboard_evidence_viewed: false,
        diagnostics_exported: false,
    };

    let failures = vec![orqestra_desktop::commands::beta_evidence::FailureEntry {
        code: "REPO_OPEN_FAILED".to_string(),
        severity: "blocking".to_string(),
        category: "repo".to_string(),
        user_recoverable: false,
        blocked_steps: vec!["repo_opened".to_string()],
    }];

    let outcome = orqestra_desktop::commands::beta_evidence::compute_session_outcome(&steps, &failures);
    assert_eq!(outcome, "blocked");
}

#[test]
fn session_completed_when_all_steps_pass() {
    let steps = orqestra_desktop::commands::beta_evidence::SessionSteps {
        app_launched: true,
        repo_opened: true,
        roadmap_detected: true,
        pm_views_rendered: true,
        readiness_reviewed: true,
        ai_service_available: true,
        agent_flow_completed: true,
        ai_degraded_mode_understood: true,
        dashboard_evidence_viewed: true,
        diagnostics_exported: true,
    };

    let outcome = orqestra_desktop::commands::beta_evidence::compute_session_outcome(&steps, &[]);
    assert_eq!(outcome, "completed");
}

// ---------------------------------------------------------------------------
// Redaction
// ---------------------------------------------------------------------------

#[test]
fn beta_evidence_redacts_tokens() {
    let feedback = orqestra_desktop::commands::beta_evidence::BetaFeedback {
        feedback_type: "external_beta".to_string(),
        role: "developer".to_string(),
        experience_level: "technical".to_string(),
        ratings: orqestra_desktop::commands::beta_evidence::BetaRatings {
            install_clarity: Some(4),
            onboarding_clarity: Some(4),
            readiness_clarity: Some(5),
            pm_views_usefulness: Some(4),
            ai_degraded_mode_clarity: Some(5),
            dashboard_evidence_clarity: Some(4),
            overall_confidence: Some(4),
        },
        free_text: orqestra_desktop::commands::beta_evidence::BetaFreeText {
            what_worked: Some("My token is ghp_1234567890abcdefghijklmnopqrstuvwxyz".to_string()),
            what_confused_you: None,
            what_blocked_you: Some("API key sk-abcdefghijklmnopqrst12345 blocked me".to_string()),
            what_should_change_before_wider_beta: None,
        },
        share_permission: orqestra_desktop::commands::beta_evidence::SharePermission {
            allow_aggregate_use: true,
            allow_quote_use: false,
        },
    };

    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(make_valid_consent()),
        None,
        Some(feedback),
        None,
    ).expect("ok");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let fb_json = fs::read_to_string(bundle_path.join("beta-feedback.json")).expect("read");

    assert!(!fb_json.contains("ghp_"), "PAT should be redacted");
    assert!(!fb_json.contains("sk-abcdefghijklmnopqrst"), "API key should be redacted");
    assert!(fb_json.contains("[REDACTED]"), "should contain redaction marker");

    let _ = fs::remove_dir_all(&bundle_path);
}

#[test]
fn beta_evidence_hashes_paths() {
    let (_dir, repo_path) = create_temp_repo();
    let repo_str = repo_path.to_str().unwrap().to_string();

    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        Some(repo_str.clone()),
        Some(make_valid_consent()),
        None,
        None,
        None,
    ).expect("ok");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let outcome_json = fs::read_to_string(bundle_path.join("beta-session-outcome.json")).expect("read");

    // Should NOT contain raw path
    assert!(!outcome_json.contains(&repo_str), "raw path should not appear");
    // Should contain hash
    assert!(outcome_json.contains("sha256:"), "should contain hashed path");

    let _ = fs::remove_dir_all(&bundle_path);
}

// ---------------------------------------------------------------------------
// Failure taxonomy completeness
// ---------------------------------------------------------------------------

#[test]
fn failure_taxonomy_includes_all_16_codes() {
    let expected = [
        "INSTALL_BLOCKED",
        "SMARTSCREEN_WARNING",
        "APP_LAUNCH_FAILED",
        "REPO_OPEN_FAILED",
        "ROADMAP_NOT_FOUND",
        "GIT_UNAVAILABLE",
        "KEYCHAIN_UNAVAILABLE",
        "AI_SERVICE_UNAVAILABLE",
        "AGENT_FLOW_FAILED",
        "DIFF_REVIEW_FAILED",
        "DASHBOARD_EXPORT_FAILED",
        "EVIDENCE_SCHEMA_INVALID",
        "DIAGNOSTICS_EXPORT_FAILED",
        "USER_ABANDONED",
        "UNKNOWN_FAILURE",
        "CONSENT_DECLINED",
    ];

    for code in &expected {
        assert!(
            orqestra_desktop::commands::beta_evidence::FAILURE_CODES.contains(code),
            "missing failure code: {}",
            code
        );
    }

    assert_eq!(orqestra_desktop::commands::beta_evidence::FAILURE_CODES.len(), 16,
        "should have exactly 16 failure codes");
}

// ---------------------------------------------------------------------------
// Non-mutation
// ---------------------------------------------------------------------------

#[test]
fn beta_evidence_export_is_non_mutating() {
    // Create a separate temp repo to avoid interference with other tests
    let dir = tempfile::tempdir().expect("temp dir");
    let git_dir = dir.path().join(".git");
    fs::create_dir_all(&git_dir).expect("git dir");

    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir.path())
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir.path())
        .output();
    fs::write(dir.path().join("initial.txt"), "test").expect("write");
    // Add .Orqestra to gitignore
    fs::write(dir.path().join(".gitignore"), ".Orqestra/\n").expect("gitignore");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(dir.path())
        .output();
    let commit_out = std::process::Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(dir.path())
        .output()
        .expect("git commit");
    assert!(commit_out.status.success(), "git commit should succeed");

    let repo_path = dir.path().to_path_buf();

    let status_before = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo_path)
        .output()
        .expect("git status before");
    let dirty_before = !status_before.stdout.is_empty();

    let _result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        Some(repo_path.to_str().unwrap().to_string()),
        Some(make_valid_consent()),
        None,
        None,
        None,
    );

    let status_after = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo_path)
        .output()
        .expect("git status after");
    let dirty_after = !status_after.stdout.is_empty();

    assert_eq!(dirty_before, dirty_after, "repo dirty state should not change");

    // Cleanup
    let orqestra_dir = repo_path.join(".Orqestra");
    let _ = fs::remove_dir_all(&orqestra_dir);
}

// ---------------------------------------------------------------------------
// Correction #2: feedback always has schema_version
// ---------------------------------------------------------------------------

#[test]
fn beta_feedback_always_includes_schema_version() {
    let feedback = orqestra_desktop::commands::beta_evidence::BetaFeedback {
        feedback_type: "external_beta".to_string(),
        role: "developer".to_string(),
        experience_level: "technical".to_string(),
        ratings: orqestra_desktop::commands::beta_evidence::BetaRatings {
            install_clarity: Some(5),
            onboarding_clarity: Some(4),
            readiness_clarity: Some(5),
            pm_views_usefulness: Some(4),
            ai_degraded_mode_clarity: Some(5),
            dashboard_evidence_clarity: Some(4),
            overall_confidence: Some(4),
        },
        free_text: orqestra_desktop::commands::beta_evidence::BetaFreeText {
            what_worked: Some("Everything worked".to_string()),
            what_confused_you: None,
            what_blocked_you: None,
            what_should_change_before_wider_beta: None,
        },
        share_permission: orqestra_desktop::commands::beta_evidence::SharePermission {
            allow_aggregate_use: true,
            allow_quote_use: false,
        },
    };

    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(make_valid_consent()),
        None,
        Some(feedback),
        None,
    ).expect("ok");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let fb: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(bundle_path.join("beta-feedback.json")).expect("read")
    ).expect("parse");

    assert_eq!(fb["schema_version"], 1,
        "beta-feedback.json must always have schema_version when feedback is provided");

    let _ = fs::remove_dir_all(&bundle_path);
}

// ---------------------------------------------------------------------------
// Correction #3: taxonomy includes all 16 codes
// ---------------------------------------------------------------------------

#[test]
fn beta_failure_taxonomy_includes_all_16_codes_in_export() {
    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(make_valid_consent()),
        None,
        None,
        None,
    ).expect("ok");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let taxonomy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(bundle_path.join("beta-failure-taxonomy.json")).expect("read")
    ).expect("parse");

    assert_eq!(taxonomy["schema_version"], 1);

    // Must have "taxonomy" array with all 16 codes
    let tax = taxonomy["taxonomy"].as_array().expect("taxonomy should be array");
    assert_eq!(tax.len(), 16, "taxonomy must have exactly 16 entries");

    let codes: Vec<&str> = tax.iter()
        .filter_map(|e| e["code"].as_str())
        .collect();
    assert!(codes.contains(&"CONSENT_DECLINED"), "must include CONSENT_DECLINED");
    assert!(codes.contains(&"UNKNOWN_FAILURE"), "must include UNKNOWN_FAILURE");
    assert!(codes.contains(&"USER_ABANDONED"), "must include USER_ABANDONED");
    assert!(codes.contains(&"AI_SERVICE_UNAVAILABLE"), "must include AI_SERVICE_UNAVAILABLE");

    // Must have "observed_failures" array (can be empty)
    assert!(taxonomy["observed_failures"].is_array(), "must have observed_failures array");

    let _ = fs::remove_dir_all(&bundle_path);
}

// ---------------------------------------------------------------------------
// Correction #4: incomplete sessions get default failure code
// ---------------------------------------------------------------------------

#[test]
fn abandoned_session_injects_user_abandoned_failure() {
    // Minimal steps: only app launched
    let steps = serde_json::json!({
        "app_launched": true,
        "repo_opened": false,
        "roadmap_detected": false,
        "pm_views_rendered": false,
        "readiness_reviewed": false,
        "ai_service_available": false,
        "agent_flow_completed": false,
        "ai_degraded_mode_understood": false,
        "dashboard_evidence_viewed": false,
        "diagnostics_exported": false,
    });

    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(make_valid_consent()),
        Some(steps),
        None,
        None,  // No failures supplied
    ).expect("ok");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let outcome: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(bundle_path.join("beta-session-outcome.json")).expect("read")
    ).expect("parse");

    // Outcome should be abandoned
    assert_eq!(outcome["outcome"], "abandoned");

    // Taxonomy should have observed failures with USER_ABANDONED
    let taxonomy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(bundle_path.join("beta-failure-taxonomy.json")).expect("read")
    ).expect("parse");

    let observed = taxonomy["observed_failures"].as_array().expect("observed_failures array");
    assert!(!observed.is_empty(), "abandoned session must have at least one observed failure");
    assert!(observed.iter().any(|f| f["code"] == "USER_ABANDONED"),
        "must include USER_ABANDONED failure");

    let _ = fs::remove_dir_all(&bundle_path);
}

#[test]
fn unknown_outcome_injects_unknown_failure() {
    // Steps that produce "unknown" — app launched but mixed
    let steps = serde_json::json!({
        "app_launched": true,
        "repo_opened": true,
        "roadmap_detected": false,
        "pm_views_rendered": false,
        "readiness_reviewed": false,
        "ai_service_available": false,
        "agent_flow_completed": false,
        "ai_degraded_mode_understood": false,
        "dashboard_evidence_viewed": false,
        "diagnostics_exported": false,
    });

    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        None,
        Some(make_valid_consent()),
        Some(steps),
        None,
        None,
    ).expect("ok");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let taxonomy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(bundle_path.join("beta-failure-taxonomy.json")).expect("read")
    ).expect("parse");

    let observed = taxonomy["observed_failures"].as_array().expect("array");
    assert!(!observed.is_empty(), "unknown outcome must have at least one failure");
    assert!(observed.iter().any(|f| f["code"] == "UNKNOWN_FAILURE"),
        "must include UNKNOWN_FAILURE");

    let _ = fs::remove_dir_all(&bundle_path);
}

// ---------------------------------------------------------------------------
// Truthful steps from UI produce correct outcomes
// ---------------------------------------------------------------------------

#[test]
fn truthful_steps_from_readiness_panel_produces_completed_or_warnings() {
    // Simulate what the UI sends when exporting from ReadinessStep
    // with AI unreachable and roadmap found
    let steps = serde_json::json!({
        "app_launched": true,
        "repo_opened": true,
        "roadmap_detected": true,
        "pm_views_rendered": true,
        "readiness_reviewed": true,
        "ai_service_available": false,
        "agent_flow_completed": false,
        "ai_degraded_mode_understood": true,
        "dashboard_evidence_viewed": false,
        "diagnostics_exported": true,
    });

    let failures = vec![serde_json::json!({
        "code": "AI_SERVICE_UNAVAILABLE",
        "severity": "warning",
        "category": "ai_service",
        "user_recoverable": true,
        "blocked_steps": ["agent_execution"]
    })];

    let failures_json = serde_json::json!(failures);

    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        Some("/tmp".to_string()),
        Some(make_valid_consent()),
        Some(steps),
        None,
        Some(failures_json),
    ).expect("ok");

    assert!(result.ok);

    let bundle_path = PathBuf::from(result.path.unwrap());
    let outcome: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(bundle_path.join("beta-session-outcome.json")).expect("read")
    ).expect("parse");

    // Must NOT be "abandoned" — user completed most steps
    let outcome_str = outcome["outcome"].as_str().unwrap();
    assert_ne!(outcome_str, "abandoned", "readiness panel export must not produce abandoned");
    assert_ne!(outcome_str, "blocked", "readiness panel export must not produce blocked");
    // Should be completed_with_warnings (AI down)
    assert_eq!(outcome_str, "completed_with_warnings");

    // Steps should reflect what was actually passed
    let steps = &outcome["steps"];
    assert_eq!(steps["app_launched"], true);
    assert_eq!(steps["repo_opened"], true);
    assert_eq!(steps["roadmap_detected"], true);
    assert_eq!(steps["readiness_reviewed"], true);
    assert_eq!(steps["diagnostics_exported"], true);
    assert_eq!(steps["ai_service_available"], false);

    let _ = fs::remove_dir_all(&bundle_path);
}

#[test]
fn truthful_steps_all_pass_produces_completed() {
    let steps = serde_json::json!({
        "app_launched": true,
        "repo_opened": true,
        "roadmap_detected": true,
        "pm_views_rendered": true,
        "readiness_reviewed": true,
        "ai_service_available": true,
        "agent_flow_completed": true,
        "ai_degraded_mode_understood": true,
        "dashboard_evidence_viewed": true,
        "diagnostics_exported": true,
    });

    let result = orqestra_desktop::commands::beta_evidence::export_beta_evidence_cmd(
        Some("/tmp".to_string()),
        Some(make_valid_consent()),
        Some(steps),
        None,
        None,
    ).expect("ok");

    let bundle_path = PathBuf::from(result.path.unwrap());
    let outcome: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(bundle_path.join("beta-session-outcome.json")).expect("read")
    ).expect("parse");

    assert_eq!(outcome["outcome"], "completed");

    let _ = fs::remove_dir_all(&bundle_path);
}
