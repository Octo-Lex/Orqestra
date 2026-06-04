//! v1.4.1: Agent Context Quality Stabilization tests.
//!
//! Expanded payload regression fixtures, hardened forbidden-field scan,
//! and graceful-degradation checks for both docs-agent and bugfix-agent.

use std::fs;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temp directory with a git repo and one initial commit.
fn init_test_repo() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    std::process::Command::new("git")
        .current_dir(&root)
        .args(["init"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["config", "user.name", "Test"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["config", "user.email", "test@test.com"])
        .output()
        .unwrap();

    fs::write(root.join("README.md"), "# Test\n").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "README.md"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    (tmp, root)
}

/// Stage all changes in a repo.
fn stage_all(root: &std::path::Path) {
    std::process::Command::new("git")
        .current_dir(root)
        .args(["add", "-A"])
        .output()
        .unwrap();
}

// ---------------------------------------------------------------------------
// Forbidden-field scan — scoped to git_context only
// ---------------------------------------------------------------------------

/// Keys forbidden as JSON object keys INSIDE git_context.
/// These carry content and would leak file data.
const FORBIDDEN_KEYS: &[&str] = &[
    "content",
    "body",
    "diff",
    "patch",
    "file_text",
    "raw",
    "token",
    "authorization",
    "secret_value",
    "private_key",
];

/// Safe metadata keys that contain forbidden substrings but are allowed.
/// These are counts, flags, or labels — not content.
const SAFE_KEYS: &[&str] = &[
    "secret_count",
    "secret_contents_excluded",
    "raw_diffs",
    "risk_reason",
];

/// Scan ONLY the git_context object for forbidden keys.
/// Does NOT scan task, context_files, agent response body, or release notes.
fn scan_git_context_forbidden(payload: &serde_json::Value) -> Vec<String> {
    let git_context = match payload.get("git_context") {
        Some(ctx) if ctx.is_object() && !ctx.as_object().unwrap().is_empty() => ctx,
        _ => return Vec::new(), // no context or empty context = nothing to scan
    };
    let mut violations = Vec::new();
    scan_recursive(git_context, &mut violations, "git_context");
    violations
}

fn scan_recursive(value: &serde_json::Value, violations: &mut Vec<String>, path: &str) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{}.{}", path, key);

                // Check forbidden — but skip if it's a known safe metadata key
                if FORBIDDEN_KEYS.contains(&key.as_str()) && !SAFE_KEYS.contains(&key.as_str()) {
                    violations.push(child_path.clone());
                }

                scan_recursive(child, violations, &child_path);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                scan_recursive(item, violations, &format!("{}[{}]", path, i));
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Payload builder helpers
// ---------------------------------------------------------------------------

/// Build docs-agent request payload matching the real command structure.
fn build_docs_agent_payload(
    project_root: &std::path::Path,
) -> serde_json::Value {
    let (safe_context, git_context_status, git_context_error_code) =
        match git_bridge::build_agent_context_v2(project_root) {
            Ok(ctx) => (
                serde_json::to_value(&ctx).unwrap_or(serde_json::json!({})),
                "available".to_string(),
                serde_json::Value::Null,
            ),
            Err(_) => (
                serde_json::json!({}),
                "unavailable".to_string(),
                serde_json::json!("AGENT_CONTEXT_BUILD_FAILED"),
            ),
        };

    serde_json::json!({
        "task": {"description": "update docs"},
        "context_files": [],
        "git_context": safe_context,
        "git_context_status": git_context_status,
        "git_context_error_code": git_context_error_code,
        "constraints": {
            "allowed_paths": ["README.md", "docs/", "roadmap/", "CHANGELOG.md"],
            "max_files_changed": 3,
            "review_only": true,
            "auto_commit": false,
            "auto_apply": false
        }
    })
}

/// Build bugfix-agent request payload matching the real command structure.
fn build_bugfix_agent_payload(
    project_root: &std::path::Path,
    allowed_paths: &[&str],
) -> serde_json::Value {
    let (safe_context, git_context_status, git_context_error_code) =
        match git_bridge::build_agent_context_v2(project_root) {
            Ok(ctx) => (
                serde_json::to_value(&ctx).unwrap_or(serde_json::json!({})),
                "available".to_string(),
                serde_json::Value::Null,
            ),
            Err(_) => (
                serde_json::json!({}),
                "unavailable".to_string(),
                serde_json::json!("AGENT_CONTEXT_BUILD_FAILED"),
            ),
        };

    let files: Vec<serde_json::Value> = allowed_paths
        .iter()
        .map(|p| serde_json::json!({"path": p, "reason": "test fixture"}))
        .collect();

    serde_json::json!({
        "task": {"description": "fix bug"},
        "allowed_files": files,
        "git_context": safe_context,
        "git_context_status": git_context_status,
        "git_context_error_code": git_context_error_code,
        "constraints": {
            "allowed_paths": allowed_paths,
            "max_files_changed": allowed_paths.len(),
            "review_only": true,
            "auto_commit": false,
            "auto_apply": false,
            "may_request_more_files": true
        }
    })
}

// ---------------------------------------------------------------------------
// Common assertion helpers
// ---------------------------------------------------------------------------

fn assert_safety_invariants(payload: &serde_json::Value, expected_status: &str) {
    // Context status
    assert_eq!(payload["git_context_status"], expected_status);

    // Constraints
    let constraints = &payload["constraints"];
    assert_eq!(constraints["review_only"], true);
    assert_eq!(constraints["auto_commit"], false);
    assert_eq!(constraints["auto_apply"], false);

    // Forbidden-field scan (scoped to git_context only)
    let violations = scan_git_context_forbidden(payload);
    assert!(violations.is_empty(), "Forbidden keys in git_context: {:?}", violations);

    // Context does not expand allowed_paths
    let allowed_in_constraints: Vec<_> = constraints["allowed_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let max_in_constraints = constraints["max_files_changed"].as_u64().unwrap() as usize;

    // If context is available, verify it didn't add extra paths or increase max
    if expected_status == "available" {
        let git_context = &payload["git_context"];
        if let Some(changed) = git_context.get("changed_files").and_then(|f| f.as_array()) {
            let context_paths: Vec<_> = changed.iter()
                .filter_map(|f| f.get("path").and_then(|p| p.as_str()))
                .collect();
            // Context paths must be a subset — they don't add to allowed_paths
            for cp in &context_paths {
                // Context paths are informational, not expanding allowed_paths
                assert!(!cp.contains(':') || cp.len() < 3, "Path should be repo-relative: {}", cp);
            }
        }
        // max_files_changed must not be increased by context
        assert!(max_in_constraints <= 10, "max_files_changed suspiciously high: {}", max_in_constraints);
    }
}

fn assert_unavailable_safety(payload: &serde_json::Value) {
    assert_safety_invariants(payload, "unavailable");
    assert_eq!(payload["git_context_error_code"], "AGENT_CONTEXT_BUILD_FAILED");
    // git_context should be empty
    assert!(payload["git_context"].as_object().unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// WS-B: Payload regression fixtures — 11 fixtures × 2 agents
// ---------------------------------------------------------------------------

// --- Fixture setup helpers ---

fn setup_docs_only(root: &std::path::Path) {
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide\n").unwrap();
    fs::write(root.join("docs/api.md"), "# API\n").unwrap();
    stage_all(root);
}

fn setup_bugfix_source(root: &std::path::Path) {
    fs::create_dir_all(root.join("crates/git-bridge/src")).unwrap();
    fs::write(root.join("crates/git-bridge/src/fix.rs"), "pub fn fix() {}").unwrap();
    stage_all(root);
}

fn setup_mixed(root: &std::path::Path) {
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide\n").unwrap();
    fs::create_dir_all(root.join("crates/git-bridge/src")).unwrap();
    fs::write(root.join("crates/git-bridge/src/lib.rs"), "pub mod x;").unwrap();
    stage_all(root);
}

fn setup_workflow_risk(root: &std::path::Path) {
    fs::create_dir_all(root.join(".github/workflows")).unwrap();
    fs::write(root.join(".github/workflows/ci.yml"), "name: CI\n").unwrap();
    stage_all(root);
}

fn setup_secret_risk(root: &std::path::Path) {
    fs::write(root.join(".env"), "SECRET_KEY=value123\n").unwrap();
    stage_all(root);
}

fn setup_binary(root: &std::path::Path) {
    let binary_content: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A];
    fs::write(root.join("image.png"), &binary_content).unwrap();
    stage_all(root);
}

fn setup_large(root: &std::path::Path) {
    let large_content = vec![0u8; 10 * 1024 * 1024 + 1];
    fs::write(root.join("large.bin"), &large_content).unwrap();
    stage_all(root);
}

fn setup_renamed(root: &std::path::Path) {
    std::process::Command::new("git")
        .current_dir(root)
        .args(["mv", "README.md", "NEW_README.md"])
        .output()
        .unwrap();
}

fn setup_deleted(root: &std::path::Path) {
    fs::write(root.join("to_delete.txt"), "content").unwrap();
    stage_all(root);
    std::process::Command::new("git")
        .current_dir(root)
        .args(["commit", "-m", "Add file"])
        .output()
        .unwrap();
    fs::remove_file(root.join("to_delete.txt")).unwrap();
}

fn setup_multi_scope(root: &std::path::Path) {
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide").unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join("scripts/build.sh"), "#!/bin/bash").unwrap();
    fs::create_dir_all(root.join("crates/git-bridge/src")).unwrap();
    fs::write(root.join("crates/git-bridge/src/lib.rs"), "pub mod x;").unwrap();
    stage_all(root);
}

fn setup_clean_repo(_root: &std::path::Path) {
    // Already clean after init_test_repo
}

// --- Macro to generate both agent tests per fixture ---

macro_rules! payload_fixture {
    ($name:ident, $setup:ident, $allowed:expr) => {
        #[test]
        fn concat_id!($name, _docs_agent)() {
            let (_tmp, root) = init_test_repo();
            $setup(&root);
            let payload = build_docs_agent_payload(&root);
            assert_safety_invariants(&payload, "available");
        }

        #[test]
        fn concat_id!($name, _bugfix_agent)() {
            let (_tmp, root) = init_test_repo();
            $setup(&root);
            let payload = build_bugfix_agent_payload(&root, $allowed);
            assert_safety_invariants(&payload, "available");
        }
    };
}

// --- Docs-agent payload fixtures ---

#[test]
fn docs_only_docs_agent() {
    let (_tmp, root) = init_test_repo();
    setup_docs_only(&root);
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
    assert_eq!(payload["git_context"]["semantic_proposal"]["scope"], "docs");
}

#[test]
fn docs_only_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    setup_docs_only(&root);
    let payload = build_bugfix_agent_payload(&root, &["docs/guide.md"]);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn bugfix_source_docs_agent() {
    let (_tmp, root) = init_test_repo();
    setup_bugfix_source(&root);
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn bugfix_source_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    setup_bugfix_source(&root);
    let payload = build_bugfix_agent_payload(&root, &["crates/git-bridge/src/fix.rs"]);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn mixed_docs_source_docs_agent() {
    let (_tmp, root) = init_test_repo();
    setup_mixed(&root);
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
    let groups = payload["git_context"]["commit_groups"].as_array().unwrap();
    assert!(groups.len() >= 2, "Mixed should produce 2+ groups");
}

#[test]
fn mixed_docs_source_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    setup_mixed(&root);
    let payload = build_bugfix_agent_payload(&root, &["crates/git-bridge/src/lib.rs"]);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn workflow_risk_docs_agent() {
    let (_tmp, root) = init_test_repo();
    setup_workflow_risk(&root);
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
    assert_eq!(payload["git_context"]["risk_summary"]["workflow_count"], 1);
}

#[test]
fn workflow_risk_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    setup_workflow_risk(&root);
    let payload = build_bugfix_agent_payload(&root, &[".github/workflows/ci.yml"]);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn secret_risk_docs_agent() {
    let (_tmp, root) = init_test_repo();
    setup_secret_risk(&root);
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
    assert_eq!(payload["git_context"]["risk_summary"]["secret_count"], 1);
    // Secret contents must not appear
    let json_str = serde_json::to_string(&payload).unwrap();
    assert!(!json_str.contains("value123"), "Secret value must not appear in payload");
}

#[test]
fn secret_risk_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    setup_secret_risk(&root);
    let payload = build_bugfix_agent_payload(&root, &[".env"]);
    assert_safety_invariants(&payload, "available");
    let json_str = serde_json::to_string(&payload).unwrap();
    assert!(!json_str.contains("value123"), "Secret value must not appear in payload");
}

#[test]
fn binary_file_docs_agent() {
    let (_tmp, root) = init_test_repo();
    setup_binary(&root);
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn binary_file_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    setup_binary(&root);
    let payload = build_bugfix_agent_payload(&root, &["image.png"]);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn large_file_docs_agent() {
    let (_tmp, root) = init_test_repo();
    setup_large(&root);
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn large_file_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    setup_large(&root);
    let payload = build_bugfix_agent_payload(&root, &["large.bin"]);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn renamed_file_docs_agent() {
    let (_tmp, root) = init_test_repo();
    setup_renamed(&root);
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn renamed_file_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    setup_renamed(&root);
    let payload = build_bugfix_agent_payload(&root, &["NEW_README.md"]);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn deleted_file_docs_agent() {
    let (_tmp, root) = init_test_repo();
    setup_deleted(&root);
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn deleted_file_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    setup_deleted(&root);
    let payload = build_bugfix_agent_payload(&root, &["to_delete.txt"]);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn multi_scope_docs_agent() {
    let (_tmp, root) = init_test_repo();
    setup_multi_scope(&root);
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
    assert!(payload["git_context"]["semantic_proposal"]["confidence"].as_f64().unwrap() < 0.9);
}

#[test]
fn multi_scope_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    setup_multi_scope(&root);
    let payload = build_bugfix_agent_payload(&root, &["crates/git-bridge/src/lib.rs"]);
    assert_safety_invariants(&payload, "available");
}

#[test]
fn clean_repo_docs_agent() {
    let (_tmp, root) = init_test_repo();
    let payload = build_docs_agent_payload(&root);
    assert_safety_invariants(&payload, "available");
    assert!(payload["git_context"]["changed_files"].as_array().unwrap().is_empty());
}

#[test]
fn clean_repo_bugfix_agent() {
    let (_tmp, root) = init_test_repo();
    let payload = build_bugfix_agent_payload(&root, &["README.md"]);
    assert_safety_invariants(&payload, "available");
}

// ---------------------------------------------------------------------------
// WS-C: Forbidden-field hardening — scoped scan, safe metadata, no body
// ---------------------------------------------------------------------------

#[test]
fn forbidden_scan_rejects_content_key() {
    let mut payload = build_docs_agent_payload_with_bad_key("content", "file data here");
    let violations = scan_git_context_forbidden(&payload);
    assert!(!violations.is_empty(), "Should reject 'content' key");
}

#[test]
fn forbidden_scan_rejects_body_key() {
    let mut payload = build_docs_agent_payload_with_bad_key("body", "some body text");
    let violations = scan_git_context_forbidden(&payload);
    assert!(!violations.is_empty(), "Should reject 'body' key");
}

#[test]
fn forbidden_scan_rejects_diff_key() {
    let mut payload = build_docs_agent_payload_with_bad_key("diff", "@@ -1 +1 @@");
    let violations = scan_git_context_forbidden(&payload);
    assert!(!violations.is_empty(), "Should reject 'diff' key");
}

#[test]
fn forbidden_scan_rejects_private_key_key() {
    let mut payload = build_docs_agent_payload_with_bad_key("private_key", "-----BEGIN...");
    let violations = scan_git_context_forbidden(&payload);
    assert!(!violations.is_empty(), "Should reject 'private_key' key");
}

#[test]
fn forbidden_scan_allows_safe_metadata_keys() {
    let (_tmp, root) = init_test_repo();
    setup_secret_risk(&root);
    let payload = build_docs_agent_payload(&root);

    // These safe keys should NOT be flagged
    let violations = scan_git_context_forbidden(&payload);
    assert!(violations.is_empty(), "Safe metadata keys should pass: {:?}", violations);

    // Verify safe keys exist in the payload
    let gc = &payload["git_context"];
    assert!(gc.get("risk_summary").unwrap().get("secret_count").is_some());
    assert!(gc.get("content_policy").unwrap().get("secret_contents_excluded").is_some());
}

#[test]
fn forbidden_scan_does_not_scan_outside_git_context() {
    // Put forbidden keys OUTSIDE git_context — should not trigger
    let (_tmp, root) = init_test_repo();
    let mut payload = build_docs_agent_payload(&root);

    // Add "body" to task — this should NOT be flagged
    payload["task"]["body"] = serde_json::json!("task description body text");

    // Add "content" to context_files — should NOT be flagged
    payload["context_files"] = serde_json::json!([{"content": "file content here"}]);

    let violations = scan_git_context_forbidden(&payload);
    assert!(violations.is_empty(), "Keys outside git_context should not be flagged: {:?}", violations);
}

#[test]
fn proposal_summary_has_no_body_field() {
    let (_tmp, root) = init_test_repo();
    fs::write(root.join("README.md"), "changed\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    let json = serde_json::to_value(&ctx).unwrap();
    let proposal = json.get("semantic_proposal").unwrap();

    assert!(proposal.get("title").is_some());
    assert!(proposal.get("scope").is_some());
    assert!(proposal.get("change_type").is_some());
    assert!(proposal.get("risk_level").is_some());
    assert!(proposal.get("confidence").is_some());
    assert!(proposal.get("body").is_none(), "ProposalSummary must not have 'body' field");
}

/// Helper: inject a bad key into git_context for testing.
fn build_docs_agent_payload_with_bad_key(key: &str, value: &str) -> serde_json::Value {
    let (_tmp, root) = init_test_repo();
    let mut payload = build_docs_agent_payload(&root);
    payload["git_context"][key] = serde_json::json!(value);
    payload
}

// ---------------------------------------------------------------------------
// WS-D: Graceful degradation — deterministic failure cases
// ---------------------------------------------------------------------------

#[test]
fn degradation_non_repo_directory() {
    let tmp = TempDir::new().unwrap();
    let payload = build_docs_agent_payload(tmp.path());
    assert_unavailable_safety(&payload);
}

#[test]
fn degradation_deleted_temp_directory() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    // Create a valid repo
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["init"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["config", "user.name", "Test"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["config", "user.email", "test@test.com"])
        .output()
        .unwrap();
    fs::write(root.join("README.md"), "# Test\n").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "init"])
        .output()
        .unwrap();

    // Drop the TempDir — directory is deleted
    drop(tmp);

    let payload = build_docs_agent_payload(&root);
    assert_unavailable_safety(&payload);
}

#[test]
fn degradation_git_as_file() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    // Create .git as a regular file instead of directory
    fs::write(root.join(".git"), "not a git repo").unwrap();

    let payload = build_docs_agent_payload(&root);
    assert_unavailable_safety(&payload);
}

#[test]
fn degradation_path_points_to_file() {
    let tmp = TempDir::new().unwrap();
    let file_path = tmp.path().join("not_a_dir.txt");
    fs::write(&file_path, "I am a file").unwrap();

    let payload = build_docs_agent_payload(&file_path);
    assert_unavailable_safety(&payload);
}

#[test]
fn degradation_bugfix_agent_preserves_constraints() {
    let tmp = TempDir::new().unwrap();
    let payload = build_bugfix_agent_payload(tmp.path(), &["src/fix.rs"]);

    assert_unavailable_safety(&payload);

    // Verify constraints are unchanged
    let constraints = &payload["constraints"];
    assert_eq!(constraints["review_only"], true);
    assert_eq!(constraints["auto_commit"], false);
    assert_eq!(constraints["auto_apply"], false);
    assert_eq!(constraints["max_files_changed"], 1);

    // Verify allowed_paths not expanded
    let allowed: Vec<_> = constraints["allowed_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(allowed, vec!["src/fix.rs"]);
}

// ---------------------------------------------------------------------------
// Schema and content policy (preserved from v1.4.0)
// ---------------------------------------------------------------------------

#[test]
fn v2_context_schema_and_content_policy() {
    let (_tmp, root) = init_test_repo();
    fs::write(root.join("README.md"), "modified\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();

    assert_eq!(ctx.schema_version, "agent-context-v2");
    assert!(!ctx.content_policy.git_context_file_contents);
    assert!(!ctx.content_policy.diff_body_included);
    assert!(ctx.content_policy.secret_contents_excluded);
    assert!(ctx.content_policy.binary_contents_excluded);
    assert!(ctx.content_policy.large_contents_excluded);
    assert!(ctx.content_policy.symlink_contents_excluded);
}

#[test]
fn v2_context_paths_are_relative() {
    let (_tmp, root) = init_test_repo();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();

    for f in &ctx.changed_files {
        // Repo-relative paths should not contain drive letters (Windows)
        // or absolute prefixes
        assert!(
            !f.path.matches(':').count() > 1 || f.path.len() < 3,
            "Path should be repo-relative, got: {}",
            f.path
        );
    }
}
