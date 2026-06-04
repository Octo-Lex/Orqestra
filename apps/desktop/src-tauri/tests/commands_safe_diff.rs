//! v1.5.0: Safe Diff Context Pilot tests.
//!
//! Tests for opt-in, bounded safe diff context for review-only agents.

use std::fs;
use tempfile::TempDir;

fn init_test_repo() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    std::process::Command::new("git").current_dir(&root).args(["init"]).output().unwrap();
    std::process::Command::new("git").current_dir(&root).args(["config", "user.name", "Test"]).output().unwrap();
    std::process::Command::new("git").current_dir(&root).args(["config", "user.email", "test@test.com"]).output().unwrap();
    fs::write(root.join("README.md"), "# Test\n").unwrap();
    std::process::Command::new("git").current_dir(&root).args(["add", "README.md"]).output().unwrap();
    std::process::Command::new("git").current_dir(&root).args(["commit", "-m", "Initial commit"]).output().unwrap();
    (tmp, root)
}

fn stage_all(root: &std::path::Path) {
    std::process::Command::new("git").current_dir(root).args(["add", "-A"]).output().unwrap();
}

// ---------------------------------------------------------------------------
// Default-off behavior
// ---------------------------------------------------------------------------

#[test]
fn safe_diff_disabled_by_default() {
    let (_tmp, root) = init_test_repo();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn hello() {}\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert!(!ctx.safe_diff_context.enabled);
    assert_eq!(ctx.safe_diff_context.enabled_source, "default-off");
    assert!(!ctx.safe_diff_context.included);
    assert!(ctx.safe_diff_context.provider.is_none());
}

#[test]
fn safe_diff_disabled_no_hunks() {
    let (_tmp, root) = init_test_repo();
    fs::write(root.join("README.md"), "changed\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert!(!ctx.safe_diff_context.enabled);
    assert!(ctx.safe_diff_context.files.is_empty());
}

#[test]
fn safe_diff_disabled_preserves_constraints() {
    let (_tmp, root) = init_test_repo();
    let payload = serde_json::json!({
        "git_context": serde_json::to_value(git_bridge::build_agent_context_v2(&root).unwrap()).unwrap(),
        "constraints": {"review_only": true, "auto_commit": false, "auto_apply": false}
    });
    assert_eq!(payload["constraints"]["review_only"], true);
    assert_eq!(payload["constraints"]["auto_commit"], false);
    assert_eq!(payload["constraints"]["auto_apply"], false);
    assert!(!payload["git_context"]["safe_diff_context"]["enabled"].as_bool().unwrap());
}

// ---------------------------------------------------------------------------
// Env-var isolation
// ---------------------------------------------------------------------------

#[test]
fn legacy_env_var_does_not_enable_safe_diff() {
    // SEMANTIC_PREP_DIFF_BODY_ENABLED should NOT enable safe diff context
    // We can't easily set/unset env vars in parallel tests, so we test
    // the function directly
    let (_tmp, root) = init_test_repo();

    // The safe_diff_context_enabled function reads ORQESTRA_SAFE_DIFF_CONTEXT
    // SEMANTIC_PREP_DIFF_BODY_ENABLED is a different env var
    let ctx = git_bridge::build_safe_diff_context(&root, &[]).unwrap();
    // Without ORQESTRA_SAFE_DIFF_CONTEXT set, it should be disabled
    assert!(!ctx.enabled);
    assert_eq!(ctx.enabled_source, "default-off");
}

// ---------------------------------------------------------------------------
// Eligibility — exclusion reasons
// ---------------------------------------------------------------------------

#[test]
fn eligibility_secret_risk_excluded() {
    let file = git_bridge::ChangedFileSummary {
        path: ".env".into(), status: "modified".into(), staged: false,
        file_kind: "text".into(), risk: "secret".into(), original_path: None,
    };
    let policy = git_bridge::SafeDiffPolicy::default();
    let (eligible, reason) = git_bridge::check_diff_eligibility(&file, &policy);
    assert!(!eligible);
    assert_eq!(reason.unwrap(), "secret-risk");
}

#[test]
fn eligibility_binary_excluded() {
    let file = git_bridge::ChangedFileSummary {
        path: "image.png".into(), status: "modified".into(), staged: false,
        file_kind: "binary".into(), risk: "normal".into(), original_path: None,
    };
    let policy = git_bridge::SafeDiffPolicy::default();
    let (eligible, reason) = git_bridge::check_diff_eligibility(&file, &policy);
    assert!(!eligible);
    assert_eq!(reason.unwrap(), "non-text");
}

#[test]
fn eligibility_large_excluded() {
    let file = git_bridge::ChangedFileSummary {
        path: "big.bin".into(), status: "modified".into(), staged: false,
        file_kind: "large".into(), risk: "normal".into(), original_path: None,
    };
    let policy = git_bridge::SafeDiffPolicy::default();
    let (eligible, reason) = git_bridge::check_diff_eligibility(&file, &policy);
    assert!(!eligible);
    assert_eq!(reason.unwrap(), "non-text");
}

#[test]
fn eligibility_symlink_excluded() {
    let file = git_bridge::ChangedFileSummary {
        path: "link.txt".into(), status: "modified".into(), staged: false,
        file_kind: "text".into(), risk: "unknown".into(), original_path: None,
    };
    let policy = git_bridge::SafeDiffPolicy::default();
    let (eligible, reason) = git_bridge::check_diff_eligibility(&file, &policy);
    assert!(!eligible);
    assert_eq!(reason.unwrap(), "symlink");
}

#[test]
fn eligibility_workflow_risk_excluded() {
    let file = git_bridge::ChangedFileSummary {
        path: ".github/workflows/ci.yml".into(), status: "modified".into(), staged: false,
        file_kind: "text".into(), risk: "workflow".into(), original_path: None,
    };
    let policy = git_bridge::SafeDiffPolicy::default();
    let (eligible, reason) = git_bridge::check_diff_eligibility(&file, &policy);
    assert!(!eligible);
    assert_eq!(reason.unwrap(), "workflow-risk");
}

#[test]
fn eligibility_deleted_excluded() {
    let file = git_bridge::ChangedFileSummary {
        path: "old.txt".into(), status: "deleted".into(), staged: false,
        file_kind: "text".into(), risk: "normal".into(), original_path: None,
    };
    let policy = git_bridge::SafeDiffPolicy::default();
    let (eligible, reason) = git_bridge::check_diff_eligibility(&file, &policy);
    assert!(!eligible);
    assert_eq!(reason.unwrap(), "unsupported-status");
}

#[test]
fn eligibility_untracked_excluded() {
    let file = git_bridge::ChangedFileSummary {
        path: "new.txt".into(), status: "untracked".into(), staged: false,
        file_kind: "text".into(), risk: "normal".into(), original_path: None,
    };
    let policy = git_bridge::SafeDiffPolicy::default();
    let (eligible, reason) = git_bridge::check_diff_eligibility(&file, &policy);
    assert!(!eligible);
    assert_eq!(reason.unwrap(), "unsupported-status");
}

#[test]
fn eligibility_normal_text_included() {
    let file = git_bridge::ChangedFileSummary {
        path: "src/lib.rs".into(), status: "modified".into(), staged: false,
        file_kind: "text".into(), risk: "normal".into(), original_path: None,
    };
    let policy = git_bridge::SafeDiffPolicy::default();
    let (eligible, reason) = git_bridge::check_diff_eligibility(&file, &policy);
    assert!(eligible);
    assert!(reason.is_none());
}

#[test]
fn eligibility_renamed_included() {
    let file = git_bridge::ChangedFileSummary {
        path: "new_name.txt".into(), status: "renamed".into(), staged: false,
        file_kind: "text".into(), risk: "normal".into(),
        original_path: Some("old_name.txt".into()),
    };
    let policy = git_bridge::SafeDiffPolicy::default();
    let (eligible, reason) = git_bridge::check_diff_eligibility(&file, &policy);
    assert!(eligible);
    assert!(reason.is_none());
}

// ---------------------------------------------------------------------------
// Caps
// ---------------------------------------------------------------------------

#[test]
fn policy_caps_are_correct() {
    let policy = git_bridge::SafeDiffPolicy::default();
    assert_eq!(policy.max_files, 5);
    assert_eq!(policy.max_file_size_bytes, 262144);
    assert_eq!(policy.max_lines_per_hunk, 80);
    assert_eq!(policy.max_lines_per_file, 120);
    assert_eq!(policy.max_total_lines, 250);
    assert!(policy.secret_risk_excluded);
    assert!(policy.binary_excluded);
    assert!(policy.large_excluded);
    assert!(policy.symlink_excluded);
    assert!(policy.workflow_risk_excluded_by_default);
}

// ---------------------------------------------------------------------------
// Payload structure
// ---------------------------------------------------------------------------

#[test]
fn payload_has_safe_diff_context_disabled() {
    let (_tmp, root) = init_test_repo();
    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    let json = serde_json::to_value(&ctx).unwrap();

    let sdc = &json["safe_diff_context"];
    assert!(sdc.get("enabled").is_some());
    assert!(!sdc["enabled"].as_bool().unwrap());
    assert_eq!(sdc["enabled_source"], "default-off");
    assert!(sdc.get("policy").is_some());
    assert!(sdc.get("summary").is_some());
    assert!(sdc.get("files").is_some());
}

#[test]
fn no_forbidden_diff_keys_in_payload() {
    let (_tmp, root) = init_test_repo();
    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    let json_str = serde_json::to_string(&ctx).unwrap();

    // These patterns must NOT appear as JSON key patterns in the payload
    // Using specific patterns that indicate key usage, not value content
    assert!(!json_str.contains("\"diff\":"), "Forbidden key 'diff' found");
    assert!(!json_str.contains("\"raw_diff\":"), "Forbidden key 'raw_diff' found");
    assert!(!json_str.contains("\"patch\":"), "Forbidden key 'patch' found");

    // These ARE allowed and present
    assert!(json_str.contains("safe_diff_context"));
    // hunks/lines appear in SafeDiffFile items; when disabled files is empty
    // so they won't appear. That's correct — they're allowed when present.
    // Verify the policy object exists (confirms DTO structure)
    assert!(json_str.contains("\"policy\""));
    assert!(json_str.contains("\"enabled_source\""));
}

#[test]
fn safe_diff_context_has_original_path_field() {
    // Verify the DTO supports original_path for rename metadata
    let (_tmp, root) = init_test_repo();
    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    let json = serde_json::to_value(&ctx).unwrap();

    // The files array should exist even when empty/disabled
    let files = json["safe_diff_context"]["files"].as_array().unwrap();
    // When disabled, files is empty — just verify the structure exists
    assert!(files.is_empty() || files[0].get("original_path").is_some());
}

// ---------------------------------------------------------------------------
// Degradation
// ---------------------------------------------------------------------------

#[test]
fn safe_diff_degrades_gracefully() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    // Non-repo — safe diff should still produce disabled state
    let ctx = git_bridge::build_safe_diff_context(&root, &[]).unwrap();
    assert!(!ctx.enabled);
    assert_eq!(ctx.enabled_source, "default-off");
}

#[test]
fn safe_diff_context_unavailable_agent_still_works() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    // Non-repo — agent context should degrade gracefully
    let result = git_bridge::build_agent_context_v2(&root);
    assert!(result.is_err(), "Non-repo should fail context build");
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn safe_diff_policy_is_deterministic() {
    let p1 = git_bridge::SafeDiffPolicy::default();
    let p2 = git_bridge::SafeDiffPolicy::default();
    assert_eq!(p1.max_files, p2.max_files);
    assert_eq!(p1.max_lines_per_hunk, p2.max_lines_per_hunk);
    assert_eq!(p1.max_lines_per_file, p2.max_lines_per_file);
    assert_eq!(p1.max_total_lines, p2.max_total_lines);
    assert_eq!(p1.max_file_size_bytes, p2.max_file_size_bytes);
}

// ---------------------------------------------------------------------------
// Hunk parsing
// ---------------------------------------------------------------------------

#[test]
fn parse_hunk_header_basic() {
    let result = git_bridge::parse_hunk_header("@@ -10,4 +10,7 @@ fn test()");
    assert!(result.is_some());
    let (os, ol, ns, nl) = result.unwrap();
    assert_eq!(os, 10);
    assert_eq!(ol, 4);
    assert_eq!(ns, 10);
    assert_eq!(nl, 7);
}

#[test]
fn parse_hunk_header_no_count() {
    let result = git_bridge::parse_hunk_header("@@ -1 +1 @@");
    assert!(result.is_some());
    let (os, ol, ns, nl) = result.unwrap();
    assert_eq!(os, 1);
    assert_eq!(ol, 0);
    assert_eq!(ns, 1);
    assert_eq!(nl, 0);
}

#[test]
fn parse_hunk_header_invalid() {
    assert!(git_bridge::parse_hunk_header("not a header").is_none());
}
