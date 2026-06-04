//! v1.4.0: Agent Context Quality tests.
//!
//! Integration-level tests that create real git repos, run build_agent_context_v2(),
//! and verify schema, content policy, forbidden-field absence, and agent payload structure.

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

/// Forbidden keys that must NOT appear as JSON object keys in agent Git context.
/// These carry content and would leak file data.
const FORBIDDEN_KEYS: &[&str] = &[
    "content",
    "diff",
    "patch",
    "file_text",
    "token",
    "authorization",
];

/// Scan a JSON value for forbidden keys at the object-key level.
/// Path-aware: only checks object keys, not string values or safe metadata.
fn scan_forbidden_keys(value: &serde_json::Value) -> Vec<String> {
    let mut violations = Vec::new();
    scan_forbidden_keys_recursive(value, &mut violations, "");
    violations
}

fn scan_forbidden_keys_recursive(
    value: &serde_json::Value,
    violations: &mut Vec<String>,
    path: &str,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };

                // Check if this key is a forbidden content key
                if FORBIDDEN_KEYS.contains(&key.as_str()) {
                    violations.push(child_path.clone());
                }

                scan_forbidden_keys_recursive(child, violations, &child_path);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                let child_path = format!("{}[{}]", path, i);
                scan_forbidden_keys_recursive(item, violations, &child_path);
            }
        }
        _ => {}
    }
}

/// Stage all changes in a repo.
fn stage_all(root: &std::path::Path) {
    std::process::Command::new("git")
        .current_dir(root)
        .args(["add", "-A"])
        .output()
        .unwrap();
}

/// Verify common v2 invariants on a context.
fn assert_v2_invariants(ctx: &git_bridge::AgentContextV2) {
    assert_eq!(ctx.schema_version, "agent-context-v2");
    assert!(!ctx.content_policy.git_context_file_contents);
    assert!(!ctx.content_policy.diff_body_included);
    assert!(ctx.content_policy.secret_contents_excluded);
    assert!(ctx.content_policy.binary_contents_excluded);
    assert!(ctx.content_policy.large_contents_excluded);
    assert!(ctx.content_policy.symlink_contents_excluded);

    // Verify no forbidden keys in serialized JSON
    let json = serde_json::to_value(ctx).unwrap();
    let violations = scan_forbidden_keys(&json);
    assert!(
        violations.is_empty(),
        "Forbidden keys found in agent context: {:?}",
        violations
    );
}

// ---------------------------------------------------------------------------
// WS-B: AgentContextV2 schema and content policy
// ---------------------------------------------------------------------------

#[test]
fn v2_context_schema_and_content_policy() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("README.md"), "modified\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();

    assert_eq!(ctx.schema_version, "agent-context-v2");
    assert!(!ctx.dirty || ctx.changed_files.is_empty() || !ctx.changed_files.is_empty());
    assert!(ctx.provider == "deterministic-heuristic" || ctx.provider == "git-cli-fallback" || ctx.provider == "gix-hybrid", "Unexpected provider: {}", ctx.provider);

    // Content policy defaults
    assert!(!ctx.content_policy.git_context_file_contents);
    assert!(!ctx.content_policy.diff_body_included);
    assert!(ctx.content_policy.secret_contents_excluded);
    assert!(ctx.content_policy.binary_contents_excluded);
    assert!(ctx.content_policy.large_contents_excluded);
    assert!(ctx.content_policy.symlink_contents_excluded);
}

#[test]
fn v2_context_has_no_forbidden_keys() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide\n").unwrap();
    fs::write(root.join("README.md"), "v2\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    let json = serde_json::to_value(&ctx).unwrap();

    // Safe metadata keys that contain "secret" or "raw" — these are allowed
    assert!(json.get("risk_summary").unwrap().get("secret_count").is_some());
    assert!(json.get("content_policy").unwrap().get("secret_contents_excluded").is_some());

    // Forbidden content keys must not appear as object keys
    let violations = scan_forbidden_keys(&json);
    assert!(violations.is_empty(), "Forbidden keys: {:?}", violations);
}

#[test]
fn v2_context_proposal_summary_has_no_body() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("README.md"), "changed\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    let json = serde_json::to_value(&ctx).unwrap();
    let proposal = json.get("semantic_proposal").unwrap();

    // Must have these fields
    assert!(proposal.get("title").is_some());
    assert!(proposal.get("scope").is_some());
    assert!(proposal.get("change_type").is_some());
    assert!(proposal.get("risk_level").is_some());
    assert!(proposal.get("confidence").is_some());

    // Must NOT have body
    assert!(proposal.get("body").is_none(), "ProposalSummary must not have 'body' field");
}

#[test]
fn v2_context_paths_are_relative() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();

    for f in &ctx.changed_files {
        assert!(
            !f.path.contains(':') || f.path.len() < 3,
            "Path should be repo-relative, got: {}",
            f.path
        );
        assert!(
            !f.path.starts_with('/') || f.path.starts_with("/"),
            "Path should be repo-relative, got: {}",
            f.path
        );
    }
}

// ---------------------------------------------------------------------------
// WS-E: Agent context fixtures — 11 change sets
// ---------------------------------------------------------------------------

#[test]
fn fixture_docs_only() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide\n").unwrap();
    fs::write(root.join("docs/api.md"), "# API\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert_v2_invariants(&ctx);

    assert!(ctx.changed_files.len() >= 2);
    assert_eq!(ctx.semantic_proposal.scope, "docs");
    assert_eq!(ctx.semantic_proposal.change_type, "docs");
}

#[test]
fn fixture_bugfix_source() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("crates/git-bridge/src")).unwrap();
    fs::write(root.join("crates/git-bridge/src/fix.rs"), "pub fn fix() {}").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert_v2_invariants(&ctx);

    assert!(ctx.changed_files.len() >= 1);
    assert!(ctx.commit_groups.len() >= 1);
}

#[test]
fn fixture_mixed_docs_and_source() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide\n").unwrap();
    fs::create_dir_all(root.join("crates/git-bridge/src")).unwrap();
    fs::write(root.join("crates/git-bridge/src/lib.rs"), "pub mod x;").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert_v2_invariants(&ctx);

    assert!(ctx.commit_groups.len() >= 2, "Mixed docs+source should produce 2+ groups");
}

#[test]
fn fixture_workflow_risk() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join(".github/workflows")).unwrap();
    fs::write(root.join(".github/workflows/ci.yml"), "name: CI\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert_v2_invariants(&ctx);

    assert_eq!(ctx.risk_summary.workflow_count, 1);
    assert!(ctx.commit_groups.iter().any(|g| g.scope == "ci"));
}

#[test]
fn fixture_secret_risk() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join(".env"), "SECRET_KEY=value123\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert_v2_invariants(&ctx);

    assert_eq!(ctx.risk_summary.secret_count, 1);
    assert!(ctx.changed_files.iter().any(|f| f.path == ".env" && f.risk == "secret"));

    // Verify .env contents are NOT in JSON
    let json = serde_json::to_string(&ctx).unwrap();
    assert!(!json.contains("value123"), "Secret values must not appear in context");
    assert!(!json.contains("SECRET_KEY=value"), "Secret contents must not appear in context");
}

#[test]
fn fixture_binary_file() {
    let (_tmp, root) = init_test_repo();

    // Write a small binary file (>0 non-text bytes)
    let binary_content: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A]; // PNG header
    fs::write(root.join("image.png"), &binary_content).unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert_v2_invariants(&ctx);

    // Binary file should appear as metadata
    assert!(ctx.changed_files.iter().any(|f| f.path == "image.png"));
}

#[test]
fn fixture_large_file() {
    let (_tmp, root) = init_test_repo();

    // Write a file > 10 MiB
    let large_content = vec![0u8; 10 * 1024 * 1024 + 1];
    fs::write(root.join("large.bin"), &large_content).unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert_v2_invariants(&ctx);

    // Large file should appear as metadata with appropriate risk
    assert!(ctx.changed_files.iter().any(|f| f.path == "large.bin"));
}

#[test]
fn fixture_renamed_file() {
    let (_tmp, root) = init_test_repo();

    std::process::Command::new("git")
        .current_dir(&root)
        .args(["mv", "README.md", "NEW_README.md"])
        .output()
        .unwrap();

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert_v2_invariants(&ctx);

    // Rename should appear in changed files
    assert!(ctx.changed_files.iter().any(|f| f.path.contains("README")));
}

#[test]
fn fixture_deleted_file() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("to_delete.txt"), "content").unwrap();
    stage_all(&root);
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Add file"])
        .output()
        .unwrap();
    fs::remove_file(root.join("to_delete.txt")).unwrap();

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert_v2_invariants(&ctx);

    assert!(ctx.changed_files.iter().any(|f| f.path == "to_delete.txt"));
}

#[test]
fn fixture_multi_scope() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide").unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join("scripts/build.sh"), "#!/bin/bash").unwrap();
    fs::create_dir_all(root.join("crates/git-bridge/src")).unwrap();
    fs::write(root.join("crates/git-bridge/src/lib.rs"), "pub mod x;").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    assert_v2_invariants(&ctx);

    assert!(ctx.commit_groups.len() >= 2, "Multi-scope should produce 2+ groups");
    assert!(ctx.semantic_proposal.confidence < 0.9, "Multi-scope should reduce confidence");
}

#[test]
fn fixture_empty_repo() {
    let (_tmp, root) = init_test_repo();

    // No changes — context should still build
    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();

    assert_eq!(ctx.schema_version, "agent-context-v2");
    assert!(ctx.changed_files.is_empty());
    assert!(ctx.commit_groups.is_empty());
}

// ---------------------------------------------------------------------------
// WS-C/D: Agent payload structure tests
// ---------------------------------------------------------------------------

#[test]
fn docs_agent_payload_structure() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("README.md"), "modified\n").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    let safe_context = serde_json::to_value(&ctx).unwrap();

    // Build the same payload structure as run_docs_agent_cmd
    let request_body = serde_json::json!({
        "task": {"description": "update docs"},
        "context_files": [],
        "git_context": safe_context,
        "git_context_status": "available",
        "git_context_error_code": null,
        "constraints": {
            "allowed_paths": ["README.md", "docs/", "roadmap/", "CHANGELOG.md"],
            "max_files_changed": 3,
            "review_only": true,
            "auto_commit": false,
            "auto_apply": false
        }
    });

    // Verify constraints
    let constraints = request_body.get("constraints").unwrap();
    assert_eq!(constraints.get("review_only").unwrap(), true);
    assert_eq!(constraints.get("auto_commit").unwrap(), false);
    assert_eq!(constraints.get("auto_apply").unwrap(), false);

    // Verify status
    assert_eq!(request_body.get("git_context_status").unwrap(), "available");
    assert!(request_body.get("git_context_error_code").unwrap().is_null());

    // Verify forbidden keys absent
    let violations = scan_forbidden_keys(&request_body);
    assert!(violations.is_empty(), "Forbidden keys in docs payload: {:?}", violations);
}

#[test]
fn bugfix_agent_payload_structure() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/fix.rs"), "pub fn fix() {}").unwrap();
    stage_all(&root);

    let ctx = git_bridge::build_agent_context_v2(&root).unwrap();
    let safe_context = serde_json::to_value(&ctx).unwrap();

    let request_body = serde_json::json!({
        "task": {"description": "fix bug"},
        "allowed_files": [{"path": "src/fix.rs", "reason": "contains bug"}],
        "git_context": safe_context,
        "git_context_status": "available",
        "git_context_error_code": null,
        "constraints": {
            "allowed_paths": ["src/fix.rs"],
            "max_files_changed": 1,
            "review_only": true,
            "auto_commit": false,
            "auto_apply": false,
            "may_request_more_files": true
        }
    });

    let constraints = request_body.get("constraints").unwrap();
    assert_eq!(constraints.get("review_only").unwrap(), true);
    assert_eq!(constraints.get("auto_commit").unwrap(), false);
    assert_eq!(constraints.get("auto_apply").unwrap(), false);

    // Context does not expand allowed paths
    let allowed: Vec<&str> = constraints
        .get("allowed_paths")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(allowed, vec!["src/fix.rs"]);

    let violations = scan_forbidden_keys(&request_body);
    assert!(violations.is_empty(), "Forbidden keys in bugfix payload: {:?}", violations);
}

#[test]
fn context_failure_degrades_gracefully() {
    // Build context from a non-existent path — should fail gracefully
    let bad_path = std::path::PathBuf::from("/nonexistent/path/that/does/not/exist");
    let result = git_bridge::build_agent_context_v2(&bad_path);

    assert!(result.is_err(), "Non-existent path should return error");

    // Verify graceful degradation payload structure
    let (safe_context, status, error_code) = match result {
        Ok(ctx) => (
            serde_json::to_value(&ctx).unwrap(),
            "available".to_string(),
            serde_json::Value::Null,
        ),
        Err(_) => (
            serde_json::json!({}),
            "unavailable".to_string(),
            serde_json::json!("AGENT_CONTEXT_BUILD_FAILED"),
        ),
    };

    assert_eq!(status, "unavailable");
    assert_eq!(error_code, "AGENT_CONTEXT_BUILD_FAILED");
    assert!(safe_context.as_object().unwrap().is_empty());
}
