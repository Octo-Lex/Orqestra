//! v1.3.1: Semantic commit preparation stabilization tests.
//!
//! Integration-level tests that create real git repos, run
//! prepare_semantic_commit, and verify proposal quality + no-write invariants.

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

/// Get HEAD SHA.
fn get_head_sha(root: &std::path::Path) -> String {
    let out = std::process::Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Get normalized sorted porcelain status.
fn get_sorted_status(root: &std::path::Path) -> Vec<String> {
    let out = std::process::Command::new("git")
        .current_dir(root)
        .args(["status", "--porcelain"])
        .output()
        .unwrap();
    let mut lines: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(String::from)
        .collect();
    lines.sort();
    lines
}

// ---------------------------------------------------------------------------
// WS-B: Proposal quality fixtures
// ---------------------------------------------------------------------------

#[test]
fn proposal_docs_only_changes() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide\n").unwrap();
    fs::write(root.join("docs/api.md"), "# API\n").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert_eq!(proposal.change_type, "docs");
    assert_eq!(proposal.scope, "docs");
    assert!(!proposal.write_operations);
    assert!(proposal.requires_review);
    assert_eq!(proposal.provider, "deterministic-heuristic");
}

#[test]
fn proposal_test_only_changes() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("tests/integration_test.rs"), "#[test] fn test() {}").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert_eq!(proposal.change_type, "test");
    assert!(!proposal.write_operations);
    assert!(proposal.requires_review);
}

#[test]
fn proposal_new_rust_source() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("crates/git-bridge/src")).unwrap();
    fs::write(root.join("crates/git-bridge/src/new_module.rs"), "pub fn hello() {}").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert_eq!(proposal.change_type, "feat");
    assert_eq!(proposal.scope, "git");
    assert!(proposal.confidence >= 0.8);
}

#[test]
fn proposal_ts_ui_changes() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("apps/desktop/src/components")).unwrap();
    fs::write(root.join("apps/desktop/src/components/Panel.tsx"), "export function Panel() {}").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert_eq!(proposal.scope, "desktop");
    assert!(proposal.change_type == "feat" || proposal.change_type == "refactor");
}

#[test]
fn proposal_mixed_rust_and_ts() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("crates/git-bridge/src")).unwrap();
    fs::write(root.join("crates/git-bridge/src/mod.rs"), "// rust change").unwrap();
    fs::create_dir_all(root.join("apps/desktop/src")).unwrap();
    fs::write(root.join("apps/desktop/src/app.tsx"), "// ts change").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    // Multi-scope should reduce confidence
    assert!(proposal.confidence < 0.9, "Multi-scope should reduce confidence below 0.9, got {}", proposal.confidence);
    assert!(proposal.groups.len() >= 2, "Multi-scope should produce multiple groups");
}

#[test]
fn proposal_release_metadata() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("roadmap")).unwrap();
    fs::write(root.join("roadmap/TASK-001.md"), "# Task").unwrap();
    fs::create_dir_all(root.join("demo")).unwrap();
    fs::write(root.join("demo/evidence.md"), "# Evidence").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert_eq!(proposal.scope, "release");
}

#[test]
fn proposal_workflow_risk() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join(".github/workflows")).unwrap();
    fs::write(root.join(".github/workflows/ci.yml"), "name: CI").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert_eq!(proposal.risk_level, "caution");
    assert!(proposal.risk_notes.iter().any(|n| n.contains("workflow")));
    // Workflow-risk should be in a separate ci group
    let ci_group = proposal.groups.iter().find(|g| g.scope == "ci");
    assert!(ci_group.is_some(), "Workflow changes should produce a ci group");
    assert_eq!(ci_group.unwrap().risk, "workflow");
}

#[test]
fn proposal_secret_risk() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join(".env"), "SECRET_KEY=value").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert_eq!(proposal.risk_level, "elevated");
    assert!(proposal.risk_notes.iter().any(|n| n.contains("secret")));
    let secret_group = proposal.groups.iter().find(|g| g.scope == "security");
    assert!(secret_group.is_some(), "Secret files should produce a security group");
    assert_eq!(secret_group.unwrap().risk, "secret");
}

#[test]
fn proposal_renamed_file() {
    let (_tmp, root) = init_test_repo();

    std::process::Command::new("git")
        .current_dir(&root)
        .args(["mv", "README.md", "NEW_README.md"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert!(!proposal.write_operations);
    assert!(proposal.requires_review);
}

#[test]
fn proposal_deleted_file() {
    let (_tmp, root) = init_test_repo();

    // Create and commit a file, then delete it
    fs::write(root.join("obsolete.txt"), "old content").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Add obsolete"])
        .output()
        .unwrap();
    fs::remove_file(root.join("obsolete.txt")).unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    // Deletion should be reflected in risk notes or body
    assert!(!proposal.write_operations);
    // Body should mention the file or counts
    assert!(proposal.body.contains("1 file") || proposal.body.contains("1 file(s)"));
}

#[test]
fn proposal_multi_scope_produces_multiple_groups() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide").unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join("scripts/build.sh"), "#!/bin/bash").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert!(proposal.groups.len() >= 2, "Multi-scope should produce 2+ groups, got {}", proposal.groups.len());
}

// ---------------------------------------------------------------------------
// WS-C: No-write regression tests
// ---------------------------------------------------------------------------

#[test]
fn proposal_does_not_modify_head() {
    let (_tmp, root) = init_test_repo();

    // Create some changes
    fs::write(root.join("new.txt"), "content").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "new.txt"])
        .output()
        .unwrap();

    let head_before = get_head_sha(&root);
    let _proposal = git_bridge::prepare_semantic_commit(&root).unwrap();
    let head_after = get_head_sha(&root);

    assert_eq!(head_before, head_after, "HEAD must not change after proposal generation");
}

#[test]
fn proposal_does_not_modify_staging_area() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("staged.txt"), "staged").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "staged.txt"])
        .output()
        .unwrap();

    let status_before = get_sorted_status(&root);
    let _proposal = git_bridge::prepare_semantic_commit(&root).unwrap();
    let status_after = get_sorted_status(&root);

    assert_eq!(status_before, status_after, "Staging area must not change after proposal generation");
}

#[test]
fn proposal_does_not_modify_worktree() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("worktree.txt"), "original content").unwrap();

    let contents_before = fs::read_to_string(root.join("worktree.txt")).unwrap();
    let _proposal = git_bridge::prepare_semantic_commit(&root).unwrap();
    let contents_after = fs::read_to_string(root.join("worktree.txt")).unwrap();

    assert_eq!(contents_before, contents_after, "Worktree files must not change after proposal generation");
}

// ---------------------------------------------------------------------------
// WS-D: Grouping stabilization
// ---------------------------------------------------------------------------

#[test]
fn grouping_single_scope_one_group() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/a.md"), "a").unwrap();
    fs::write(root.join("docs/b.md"), "b").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert_eq!(proposal.groups.len(), 1, "Single scope should produce 1 group");
    assert_eq!(proposal.groups[0].scope, "docs");
}

#[test]
fn grouping_isolates_workflow_risk() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join(".github/workflows")).unwrap();
    fs::write(root.join(".github/workflows/ci.yml"), "name: CI").unwrap();
    fs::write(root.join("README.md"), "modified").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    let normal_groups: Vec<_> = proposal.groups.iter().filter(|g| g.risk == "normal").collect();
    let ci_groups: Vec<_> = proposal.groups.iter().filter(|g| g.scope == "ci").collect();

    assert!(!ci_groups.is_empty(), "Workflow files should be in ci group");
    // Normal files should not be mixed with workflow files
    for g in &normal_groups {
        assert!(!g.files.iter().any(|f| f.contains("workflows")));
    }
}

#[test]
fn grouping_isolates_secret_risk() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join(".env"), "SECRET=val").unwrap();
    fs::write(root.join("README.md"), "modified").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    let secret_groups: Vec<_> = proposal.groups.iter().filter(|g| g.risk == "secret").collect();
    let normal_groups: Vec<_> = proposal.groups.iter().filter(|g| g.risk == "normal").collect();

    assert!(!secret_groups.is_empty(), "Secret files should be in a separate group");
    for g in &normal_groups {
        assert!(!g.files.iter().any(|f| f == ".env"));
    }
}

#[test]
fn grouping_separates_docs_from_source() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "guide").unwrap();
    fs::create_dir_all(root.join("crates/git-bridge/src")).unwrap();
    fs::write(root.join("crates/git-bridge/src/lib.rs"), "pub mod x;").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    let docs_groups: Vec<_> = proposal.groups.iter().filter(|g| g.scope == "docs").collect();
    let git_groups: Vec<_> = proposal.groups.iter().filter(|g| g.scope == "git").collect();

    assert!(!docs_groups.is_empty(), "Docs should have their own group");
    assert!(!git_groups.is_empty(), "Git source should have its own group");
}

#[test]
fn grouping_release_metadata() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("roadmap")).unwrap();
    fs::write(root.join("roadmap/TASK-001.md"), "# Task").unwrap();
    fs::write(root.join("CHANGELOG.md"), "## 1.0\n- change").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    let release_groups: Vec<_> = proposal.groups.iter().filter(|g| g.scope == "release").collect();
    assert!(!release_groups.is_empty(), "Release metadata should be in release group");
}

#[test]
fn grouping_all_groups_require_manual_review() {
    let (_tmp, root) = init_test_repo();

    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/guide.md"), "guide").unwrap();
    fs::create_dir_all(root.join("crates/git-bridge/src")).unwrap();
    fs::write(root.join("crates/git-bridge/src/lib.rs"), "pub mod x;").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    for g in &proposal.groups {
        assert!(g.requires_manual_review, "Group '{}' must require manual review", g.scope);
    }
}

#[test]
fn grouping_handles_deleted_file() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("to_delete.txt"), "content").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Add file"])
        .output()
        .unwrap();
    fs::remove_file(root.join("to_delete.txt")).unwrap();

    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert!(!proposal.write_operations);
    // Deleted file should appear somewhere in the proposal
    assert!(!proposal.body.is_empty());
}

// ---------------------------------------------------------------------------
// WS-E: Agent context secret-safety
// ---------------------------------------------------------------------------

#[test]
fn agent_context_has_no_content_field() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("README.md"), "modified").unwrap();

    let ctx = git_bridge::build_agent_context(&root).unwrap();
    let json = serde_json::to_value(&ctx).unwrap();

    // Verify no content-related fields exist
    assert!(json.get("content").is_none(), "Agent context must not have 'content' field");
    assert!(json.get("body").is_none(), "Agent context must not have 'body' field");
    assert!(json.get("diff").is_none(), "Agent context must not have 'diff' field");
    assert!(json.get("patch").is_none(), "Agent context must not have 'patch' field");

    // Verify required fields exist
    assert!(json.get("branch").is_some());
    assert!(json.get("head_short_sha").is_some());
    assert!(json.get("changed_file_paths").is_some());
    assert!(json.get("risk_summary").is_some());
}

#[test]
fn agent_context_excludes_env_contents() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join(".env"), "SUPER_SECRET=value123").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let ctx = git_bridge::build_agent_context(&root).unwrap();
    let json = serde_json::to_string(&ctx).unwrap();

    assert!(!json.contains("SUPER_SECRET"), "Agent context must not contain .env contents");
    assert!(!json.contains("value123"), "Agent context must not contain secret values");
    // The path should be present with risk flag
    assert!(json.contains(".env"), "Agent context should contain the .env path");
}

#[test]
fn agent_context_excludes_key_file_contents() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("deploy_rsa"), "-----BEGIN RSA PRIVATE KEY-----\nMIIE...").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let ctx = git_bridge::build_agent_context(&root).unwrap();
    let json = serde_json::to_string(&ctx).unwrap();

    assert!(!json.contains("BEGIN RSA"), "Agent context must not contain key contents");
    assert!(!json.contains("MIIE"), "Agent context must not contain key material");
}

#[test]
fn agent_context_lists_risk_flags_not_contents() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join(".env"), "SECRET=val").unwrap();
    fs::write(root.join("README.md"), "modified").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let ctx = git_bridge::build_agent_context(&root).unwrap();

    // Risk flags should list .env with risk level, not contents
    assert!(ctx.risk_flags.iter().any(|f| f.contains(".env") && f.contains("secret")));
}

// ---------------------------------------------------------------------------
// WS-F: Diff body pilot safeguards
// ---------------------------------------------------------------------------

#[test]
fn diff_body_pilot_workflow_risk_excluded_even_if_enabled() {
    // This test validates the path-based exclusion even without env var set
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join(".github/workflows/ci.yml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "name: CI").unwrap();

    // Workflow-risk files should be excluded regardless of pilot state
    let result = git_bridge::semantic_prep::read_safe_diff_body(
        tmp.path(),
        ".github/workflows/ci.yml",
        "text",
        "workflow",
        12,
    );
    assert!(result.is_none(), "Workflow-risk files must be excluded from diff body");
}

#[test]
fn proposal_works_without_diff_body() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("README.md"), "modified content").unwrap();

    // Pilot is disabled by default — proposal should still work
    let proposal = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert!(!proposal.title.is_empty());
    assert!(!proposal.body.is_empty());
    assert!(!proposal.write_operations);
    assert!(proposal.requires_review);
}

#[test]
fn diff_body_rejects_symlink() {
    let tmp = tempfile::TempDir::new().unwrap();
    let target = tmp.path().join("target.txt");
    let link = tmp.path().join("link.txt");
    std::fs::write(&target, "hello").unwrap();

    #[cfg(unix)]
    { std::os::unix::fs::symlink(&target, &link).unwrap(); }
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_file(&target, &link).is_err() { return; }
    }

    let result = git_bridge::semantic_prep::read_safe_diff_body(
        tmp.path(), "link.txt", "text", "normal", 5,
    );
    assert!(result.is_none(), "Symlinks must be excluded from diff body");
}

// ---------------------------------------------------------------------------
// Determinism check
// ---------------------------------------------------------------------------

#[test]
fn proposal_deterministic_same_input_same_output() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("README.md"), "v2\n").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();

    let p1 = git_bridge::prepare_semantic_commit(&root).unwrap();
    let p2 = git_bridge::prepare_semantic_commit(&root).unwrap();

    assert_eq!(p1.title, p2.title, "Same input must produce same title");
    assert_eq!(p1.scope, p2.scope, "Same input must produce same scope");
    assert_eq!(p1.change_type, p2.change_type, "Same input must produce same type");
    assert_eq!(p1.confidence, p2.confidence, "Same input must produce same confidence");
}
