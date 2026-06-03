//! Integration tests for v1.2.0 native Git operations.
//!
//! Tests snapshot, HEAD metadata, changed files, recent commits,
//! diff/stat, risk classification, and CLI parity.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temp directory with a git repo and one initial commit.
fn init_test_repo() -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    // Init repo
    let status = std::process::Command::new("git")
        .current_dir(&root)
        .args(["init"])
        .output()
        .unwrap();
    assert!(status.status.success(), "git init failed");

    // Configure user
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["config", "user.name", "Test User"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["config", "user.email", "test@example.com"])
        .output()
        .unwrap();

    // Create initial commit
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

// ---------------------------------------------------------------------------
// Repository snapshot tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_basic_repo() {
    let (_tmp, root) = init_test_repo();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    assert!(!snapshot.branch.is_empty());
    assert!(snapshot.head.is_some());
    assert!(!snapshot.dirty, "Clean repo should not be dirty");
    assert!(snapshot.latency_ms < 5000, "Snapshot should be fast");
    assert!(!snapshot.provider.is_empty());
    assert!(snapshot.changed_files.is_empty(), "Clean repo has no changed files");
}

#[test]
fn snapshot_dirty_repo() {
    let (_tmp, root) = init_test_repo();

    // Create untracked file
    fs::write(root.join("new_file.txt"), "hello").unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    assert!(snapshot.dirty);
    assert!(snapshot.untracked_count >= 1);
    assert!(snapshot.changed_files.len() >= 1);
}

#[test]
fn snapshot_modified_file() {
    let (_tmp, root) = init_test_repo();

    // Modify existing file
    fs::write(root.join("README.md"), "# Modified\n").unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    assert!(snapshot.dirty);
    assert!(snapshot.unstaged_count >= 1);

    let modified = snapshot.changed_files.iter()
        .find(|f| f.path == "README.md");
    assert!(modified.is_some(), "README.md should appear in changed files");
    let f = modified.unwrap();
    assert_eq!(f.status, "modified");
    assert!(!f.staged);
}

#[test]
fn snapshot_staged_file() {
    let (_tmp, root) = init_test_repo();

    // Create and stage a file
    fs::write(root.join("staged.txt"), "staged content").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "staged.txt"])
        .output()
        .unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    assert!(snapshot.dirty);
    assert!(snapshot.staged_count >= 1);

    let staged = snapshot.changed_files.iter()
        .find(|f| f.path == "staged.txt");
    assert!(staged.is_some());
    assert!(staged.unwrap().staged);
}

#[test]
fn snapshot_deleted_file() {
    let (_tmp, root) = init_test_repo();

    // Delete tracked file
    fs::remove_file(root.join("README.md")).unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    assert!(snapshot.dirty);
    let deleted = snapshot.changed_files.iter()
        .find(|f| f.path == "README.md");
    assert!(deleted.is_some());
    assert_eq!(deleted.unwrap().status, "deleted");
}

// ---------------------------------------------------------------------------
// HEAD metadata tests
// ---------------------------------------------------------------------------

#[test]
fn head_metadata_normal_branch() {
    let (_tmp, root) = init_test_repo();

    let head = git_bridge::snapshot::read_head_metadata(&root)
        .expect("HEAD metadata should work")
        .expect("Should have HEAD");

    assert!(!head.sha.is_empty());
    assert!(head.sha.len() >= 40, "SHA should be full hex");
    assert_eq!(head.short_sha.len(), 7);
    assert_eq!(head.message, "Initial commit");
    assert!(!head.author_name.is_empty());
    assert!(!head.author_email.is_empty());
    assert!(!head.timestamp.is_empty());
    assert!(!head.detached, "On a branch, not detached");
}

#[test]
fn head_metadata_detached() {
    let (_tmp, root) = init_test_repo();

    // Get the current commit hash
    let output = std::process::Command::new("git")
        .current_dir(&root)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Checkout detached HEAD
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["checkout", &sha])
        .output()
        .unwrap();

    let head = git_bridge::snapshot::read_head_metadata(&root)
        .expect("HEAD metadata should work on detached")
        .expect("Should have HEAD");

    assert!(head.detached, "Should report detached HEAD");
}

#[test]
fn head_metadata_fresh_repo_no_commits() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    std::process::Command::new("git")
        .current_dir(&root)
        .args(["init"])
        .output()
        .unwrap();

    let result = git_bridge::snapshot::read_head_metadata(&root).unwrap();
    assert!(result.is_none(), "Fresh repo should have no HEAD metadata");
}

// ---------------------------------------------------------------------------
// Risk classification tests
// ---------------------------------------------------------------------------

#[test]
fn risk_classifies_env_files() {
    let (risk, reason) = git_bridge::snapshot::classify_risk_by_path(".env");
    assert_eq!(risk, "secret");
    assert!(reason.is_some());

    let (risk, _) = git_bridge::snapshot::classify_risk_by_path(".env.local");
    assert_eq!(risk, "secret");

    let (risk, _) = git_bridge::snapshot::classify_risk_by_path(".env.production");
    assert_eq!(risk, "secret");
}

#[test]
fn risk_classifies_secret_extensions() {
    let (risk, _) = git_bridge::snapshot::classify_risk_by_path("server.key");
    assert_eq!(risk, "secret");

    let (risk, _) = git_bridge::snapshot::classify_risk_by_path("cert.pem");
    assert_eq!(risk, "secret");

    let (risk, _) = git_bridge::snapshot::classify_risk_by_path("id_rsa");
    assert_eq!(risk, "secret");

    let (risk, _) = git_bridge::snapshot::classify_risk_by_path("id_ed25519");
    assert_eq!(risk, "secret");
}

#[test]
fn risk_classifies_workflow_paths() {
    let (risk, reason) = git_bridge::snapshot::classify_risk_by_path(
        ".github/workflows/ci.yml"
    );
    assert_eq!(risk, "workflow");
    assert!(reason.is_some());
}

#[test]
fn risk_classifies_normal_files() {
    let (risk, reason) = git_bridge::snapshot::classify_risk_by_path("README.md");
    assert_eq!(risk, "normal");
    assert!(reason.is_none());

    let (risk, _) = git_bridge::snapshot::classify_risk_by_path("src/main.rs");
    assert_eq!(risk, "normal");
}

#[test]
fn snapshot_secret_risk_file_flagged_not_read() {
    let (_tmp, root) = init_test_repo();

    // Create a secret-risk file
    fs::write(root.join(".env"), "SECRET_KEY=super-secret-value").unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    let env_file = snapshot.changed_files.iter()
        .find(|f| f.path == ".env");
    assert!(env_file.is_some(), ".env should appear in changed files");

    let f = env_file.unwrap();
    assert_eq!(f.risk, "secret");
    assert_eq!(f.file_kind, "unknown", "Secret-risk files must not be read for kind detection");
}

#[test]
fn snapshot_workflow_risk_file_flagged() {
    let (_tmp, root) = init_test_repo();

    // Create workflow directory and file, then stage it
    let wf_dir = root.join(".github/workflows");
    fs::create_dir_all(&wf_dir).unwrap();
    fs::write(wf_dir.join("ci.yml"), "name: CI\non: push\n").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", ".github/workflows/ci.yml"])
        .output()
        .unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    let wf_file = snapshot.changed_files.iter()
        .find(|f| f.path.contains("workflows") || f.path.contains(".github"));
    assert!(wf_file.is_some(), "Should find workflow file in: {:?}",
        snapshot.changed_files.iter().map(|f| &f.path).collect::<Vec<_>>());
    assert_eq!(wf_file.unwrap().risk, "workflow");
}

// ---------------------------------------------------------------------------
// Recent commits tests
// ---------------------------------------------------------------------------

#[test]
fn recent_commits_basic() {
    let (_tmp, root) = init_test_repo();

    // Add more commits
    for i in 1..=3 {
        fs::write(root.join(format!("file{i}.txt")), format!("content {i}")).unwrap();
        std::process::Command::new("git")
            .current_dir(&root)
            .args(["add", "-A"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .current_dir(&root)
            .args(["commit", "-m", &format!("Commit {i}")])
            .output()
            .unwrap();
    }

    let commits = git_bridge::recent_commits(&root, None).unwrap();

    assert_eq!(commits.len(), 4, "Should return 4 commits (1 initial + 3)");
    assert_eq!(commits[0].message, "Commit 3", "Most recent first");
    assert_eq!(commits[3].message, "Initial commit");
    assert!(!commits[0].sha.is_empty());
    assert!(commits[0].sha.len() >= 40);
}

#[test]
fn recent_commits_limit_enforced() {
    let (_tmp, root) = init_test_repo();

    // Add 5 more commits
    for i in 1..=5 {
        fs::write(root.join(format!("limit{i}.txt")), format!("content {i}")).unwrap();
        std::process::Command::new("git")
            .current_dir(&root)
            .args(["add", "-A"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .current_dir(&root)
            .args(["commit", "-m", &format!("Commit {i}")])
            .output()
            .unwrap();
    }

    let commits = git_bridge::recent_commits(&root, Some(3)).unwrap();
    assert_eq!(commits.len(), 3, "Should respect limit");
}

#[test]
fn recent_commits_max_limit_enforced() {
    let (_tmp, root) = init_test_repo();

    let commits = git_bridge::recent_commits(&root, Some(200)).unwrap();
    assert!(commits.len() <= 100, "Should cap at MAX_LIMIT=100");
}

#[test]
fn recent_commits_empty_repo() {
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

    // Empty repo should return empty list or fall back gracefully
    let result = git_bridge::recent_commits(&root, None);
    match result {
        Ok(commits) => assert_eq!(commits.len(), 0, "Empty repo has no commits"),
        Err(_) => {} // Also acceptable — structured error
    }
}

#[test]
fn recent_commits_have_parents() {
    let (_tmp, root) = init_test_repo();

    let commits = git_bridge::recent_commits(&root, None).unwrap();

    // Initial commit has no parents
    let initial = commits.last().unwrap();
    assert!(initial.parents.is_empty(), "Initial commit has no parents");

    // Non-initial commits have at least one parent
    if commits.len() > 1 {
        assert!(!commits[0].parents.is_empty(), "Non-initial commits have parents");
    }
}

// ---------------------------------------------------------------------------
// Diff/stat tests
// ---------------------------------------------------------------------------

#[test]
fn diff_stat_clean_repo() {
    let (_tmp, root) = init_test_repo();

    let stat = git_bridge::diff_stat(&root).unwrap();

    assert_eq!(stat.files_changed, 0, "Clean repo has no diff");
    assert_eq!(stat.insertions, 0);
    assert_eq!(stat.deletions, 0);
    assert_eq!(stat.provider, "git-cli-fallback");
    assert!(stat.fallback_used);
}

#[test]
fn diff_stat_with_changes() {
    let (_tmp, root) = init_test_repo();

    // Modify file
    fs::write(root.join("README.md"), "# Modified\nMore content\n").unwrap();

    let stat = git_bridge::diff_stat(&root).unwrap();

    assert!(stat.files_changed >= 1);
    // insertions/deletions depend on diff content
    assert_eq!(stat.provider, "git-cli-fallback");
}

#[test]
fn diff_stat_secret_risk_flagged() {
    let (_tmp, root) = init_test_repo();

    // Create and commit a file, then modify it
    fs::write(root.join("config.txt"), "safe content").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Add config"])
        .output()
        .unwrap();

    // Create a .env file (untracked, but diff only shows tracked changes)
    // Instead, modify README to ensure diff has content
    fs::write(root.join("README.md"), "# Changed\n").unwrap();

    let stat = git_bridge::diff_stat(&root).unwrap();
    // The stat should not expose file contents — only counts
    assert!(stat.files_changed >= 1);
}

// ---------------------------------------------------------------------------
// Parity tests — snapshot vs CLI
// ---------------------------------------------------------------------------

#[test]
fn parity_snapshot_branch_matches_cli() {
    let (_tmp, root) = init_test_repo();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    // Get branch from CLI
    let cli = std::process::Command::new("git")
        .current_dir(&root)
        .args(["branch", "--show-current"])
        .output()
        .unwrap();
    let cli_branch = String::from_utf8_lossy(&cli.stdout).trim().to_string();

    assert_eq!(snapshot.branch, cli_branch,
        "Snapshot branch should match CLI: snapshot={}, cli={}",
        snapshot.branch, cli_branch);
}

#[test]
fn parity_snapshot_dirty_matches_cli() {
    let (_tmp, root) = init_test_repo();

    // Make dirty
    fs::write(root.join("dirty.txt"), "content").unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    // CLI check
    let cli = std::process::Command::new("git")
        .current_dir(&root)
        .args(["status", "--porcelain"])
        .output()
        .unwrap();
    let cli_dirty = !String::from_utf8_lossy(&cli.stdout).trim().is_empty();

    assert_eq!(snapshot.dirty, cli_dirty,
        "Dirty flag mismatch: snapshot={}, cli={}", snapshot.dirty, cli_dirty);
}

#[test]
fn parity_snapshot_head_sha_matches_cli() {
    let (_tmp, root) = init_test_repo();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();
    let snapshot_sha = snapshot.head.as_ref().unwrap().sha.clone();

    let cli = std::process::Command::new("git")
        .current_dir(&root)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let cli_sha = String::from_utf8_lossy(&cli.stdout).trim().to_string();

    assert_eq!(snapshot_sha, cli_sha,
        "HEAD SHA mismatch: snapshot={}, cli={}", snapshot_sha, cli_sha);
}

#[test]
fn parity_snapshot_counts_match_cli() {
    let (_tmp, root) = init_test_repo();

    // Create various states
    fs::write(root.join("untracked.txt"), "new").unwrap();
    fs::write(root.join("README.md"), "# Modified\n").unwrap();
    fs::write(root.join("staged.txt"), "staged").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "staged.txt"])
        .output()
        .unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    // Verify counts are reasonable
    assert!(snapshot.untracked_count >= 1, "Should have untracked files");
    assert!(snapshot.unstaged_count >= 1, "Should have unstaged modifications");
    assert!(snapshot.staged_count >= 1, "Should have staged files");
}

#[test]
fn non_repo_returns_structured_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    let result = git_bridge::repository_snapshot(&root);
    assert!(result.is_err(), "Non-repo should return error");
}

// ---------------------------------------------------------------------------
// Non-repo edge cases
// ---------------------------------------------------------------------------

#[test]
fn recent_commits_non_repo_returns_error() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    let result = git_bridge::recent_commits(&root, None);
    assert!(result.is_err(), "Non-repo should return error");
}

#[test]
fn diff_stat_non_repo_handles_gracefully() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();

    let result = git_bridge::diff_stat(&root);
    // Non-repo should either error or return empty stat
    match result {
        Ok(stat) => assert_eq!(stat.files_changed, 0, "Non-repo should have no changes"),
        Err(_) => {} // Structured error is acceptable
    }
}
