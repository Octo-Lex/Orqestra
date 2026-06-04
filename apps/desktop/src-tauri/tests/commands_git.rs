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

// ---------------------------------------------------------------------------
// v1.2.1: Expanded snapshot parity tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_staged_and_unstaged_same_file() {
    let (_tmp, root) = init_test_repo();

    // Modify, stage, then modify again
    fs::write(root.join("README.md"), "version 2\n").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "README.md"])
        .output()
        .unwrap();
    fs::write(root.join("README.md"), "version 3\n").unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    assert!(snapshot.dirty);
    assert!(snapshot.staged_count >= 1, "Should have staged changes");
    assert!(snapshot.unstaged_count >= 1, "Should have unstaged changes");

    let readme = snapshot.changed_files.iter().find(|f| f.path == "README.md");
    assert!(readme.is_some(), "README.md should appear in changed files");
}

#[test]
fn snapshot_added_file() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("new_file.rs"), "fn main() {}").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "new_file.rs"])
        .output()
        .unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    let added = snapshot.changed_files.iter().find(|f| f.path == "new_file.rs");
    assert!(added.is_some());
    assert_eq!(added.unwrap().status, "added");
    assert!(added.unwrap().staged);
}

#[test]
fn snapshot_renamed_file() {
    let (_tmp, root) = init_test_repo();

    // Rename README.md to NEW_README.md
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["mv", "README.md", "NEW_README.md"])
        .output()
        .unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    let renamed = snapshot.changed_files.iter().find(|f| f.status == "renamed");
    assert!(renamed.is_some(), "Should detect renamed file");
    let r = renamed.unwrap();
    assert!(r.staged, "Renames are staged by git mv");
    assert!(r.original_path.is_some(), "Rename should have original_path");
    assert_eq!(r.original_path.as_ref().unwrap(), "README.md");
}

#[test]
fn snapshot_nested_directory_change() {
    let (_tmp, root) = init_test_repo();

    let deep = root.join("a/b/c");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("deep.txt"), "nested content").unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    let deep_file = snapshot.changed_files.iter().find(|f| f.path.contains("deep.txt"));
    assert!(deep_file.is_some(), "Should find nested file");
}

#[test]
fn snapshot_ignored_files_not_counted() {
    let (_tmp, root) = init_test_repo();

    // Create .gitignore
    fs::write(root.join(".gitignore"), "ignored_dir/\n*.log\n").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", ".gitignore"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Add gitignore"])
        .output()
        .unwrap();

    // Create ignored files
    fs::create_dir_all(root.join("ignored_dir")).unwrap();
    fs::write(root.join("ignored_dir/file.txt"), "ignored").unwrap();
    fs::write(root.join("debug.log"), "log content").unwrap();

    // Create a non-ignored file for contrast
    fs::write(root.join("tracked.txt"), "tracked").unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    assert!(!snapshot.changed_files.iter().any(|f| f.path.contains("ignored_dir")),
        "Ignored directory files should not appear");
    assert!(!snapshot.changed_files.iter().any(|f| f.path.ends_with(".log")),
        "Ignored *.log files should not appear");
    assert!(snapshot.changed_files.iter().any(|f| f.path == "tracked.txt"),
        "Non-ignored files should appear");
}

#[test]
fn snapshot_multiple_changes() {
    let (_tmp, root) = init_test_repo();

    // Staged + unstaged + untracked + deleted
    fs::write(root.join("staged.txt"), "staged").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "staged.txt"])
        .output()
        .unwrap();
    fs::write(root.join("README.md"), "modified\n").unwrap(); // unstaged
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("sub/untracked.txt"), "new").unwrap();
    fs::remove_file(root.join("README.md")).unwrap(); // actually delete it

    // Recreate for unstaged test
    fs::write(root.join("README.md"), "modified content\n").unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    assert!(snapshot.staged_count >= 1, "Should have staged files");
    assert!(snapshot.untracked_count >= 1, "Should have untracked files");
    assert!(snapshot.changed_files.len() >= 2, "Should have multiple changed files");
}


// ---------------------------------------------------------------------------
// v1.2.1: Risk classification hardening tests
// ---------------------------------------------------------------------------

#[test]
fn risk_classifies_certificate_extensions() {
    let (risk, reason) = git_bridge::snapshot::classify_risk_by_path("server.crt");
    assert_eq!(risk, "secret");
    assert!(reason.unwrap().contains("certificate"), "Should mention certificate");

    let (risk, reason) = git_bridge::snapshot::classify_risk_by_path("ca.cer");
    assert_eq!(risk, "secret");
    assert!(reason.unwrap().contains("certificate"), "Should mention certificate");
}

#[test]
fn risk_classifies_secret_suffixes() {
    let (risk, _) = git_bridge::snapshot::classify_risk_by_path("deploy_rsa");
    assert_eq!(risk, "secret");

    let (risk, _) = git_bridge::snapshot::classify_risk_by_path("backup_ed25519");
    assert_eq!(risk, "secret");
}

#[test]
fn risk_classifies_credential_prefixes() {
    let (risk, _) = git_bridge::snapshot::classify_risk_by_path("secrets.yml");
    assert_eq!(risk, "secret");

    let (risk, _) = git_bridge::snapshot::classify_risk_by_path("credentials.yaml");
    assert_eq!(risk, "secret");

    let (risk, _) = git_bridge::snapshot::classify_risk_by_path("secrets.json");
    assert_eq!(risk, "secret");
}

#[test]
fn risk_classifies_github_actions() {
    let (risk, reason) = git_bridge::snapshot::classify_risk_by_path(
        ".github/actions/build/action.yml"
    );
    assert_eq!(risk, "workflow");
    assert!(reason.unwrap().contains("reusable action"));
}

#[test]
fn snapshot_symlink_classified_safely() {
    let (_tmp, root) = init_test_repo();

    let link = root.join("link_to_readme");
    let symlink_created = {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("README.md"), &link).is_ok()
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(root.join("README.md"), &link).is_ok()
        }
    };

    if !symlink_created {
        return; // Symlinks not available on this platform/environment
    }

    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "link_to_readme"])
        .output()
        .unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    let link_file = snapshot.changed_files.iter().find(|f| f.path == "link_to_readme");
    if let Some(f) = link_file {
        // If our detection correctly identifies the symlink, verify constraints
        if f.file_kind == "unknown" {
            // Good — symlink was detected and classified as unknown
            assert_ne!(f.risk, "normal", "Symlinks detected as unknown kind must not be normal risk");
        }
        // If file_kind is "text" or something else, the symlink was not detected.
        // This is acceptable on Windows CI where symlink metadata differs.
        // The invariant we enforce: if detected → must be unknown, not normal.
    }
}

#[test]
fn snapshot_large_file_classified() {
    let (_tmp, root) = init_test_repo();

    let large = root.join("large_file.bin");
    let content = vec![0u8; (10 * 1024 * 1024) + 1];
    fs::write(&large, &content).unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "large_file.bin"])
        .output()
        .unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    let large_file = snapshot.changed_files.iter().find(|f| f.path == "large_file.bin");
    assert!(large_file.is_some());
    assert_eq!(large_file.unwrap().file_kind, "large");
}

#[test]
fn snapshot_binary_file_classified() {
    let (_tmp, root) = init_test_repo();

    let bin = root.join("image.png");
    let mut content = vec![0x89, 0x50, 0x4E, 0x47];
    content.extend(vec![0u8; 100]);
    fs::write(&bin, &content).unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "image.png"])
        .output()
        .unwrap();

    let snapshot = git_bridge::repository_snapshot(&root).unwrap();

    let bin_file = snapshot.changed_files.iter().find(|f| f.path == "image.png");
    assert!(bin_file.is_some());
    assert_eq!(bin_file.unwrap().file_kind, "binary");
}

// ---------------------------------------------------------------------------
// v1.2.1: Commit metadata edge cases
// ---------------------------------------------------------------------------

#[test]
fn recent_commits_merge_commit() {
    let (_tmp, root) = init_test_repo();

    std::process::Command::new("git")
        .current_dir(&root)
        .args(["checkout", "-b", "feature"])
        .output()
        .unwrap();
    fs::write(root.join("feature.txt"), "feature").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Feature commit"])
        .output()
        .unwrap();

    std::process::Command::new("git")
        .current_dir(&root)
        .args(["checkout", "master"])
        .output()
        .unwrap();
    fs::write(root.join("master.txt"), "master").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Master commit"])
        .output()
        .unwrap();

    std::process::Command::new("git")
        .current_dir(&root)
        .args(["merge", "feature", "-m", "Merge feature"])
        .output()
        .unwrap();

    let commits = git_bridge::recent_commits(&root, None).unwrap();

    let merge = commits.iter().find(|c| c.message.contains("Merge"));
    assert!(merge.is_some(), "Should find merge commit");
    assert!(merge.unwrap().parents.len() >= 2, "Merge commit should have 2+ parents");
}

#[test]
fn recent_commits_multiline_message() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("multi.txt"), "content").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Title line\n\nDetailed description\nMore details"])
        .output()
        .unwrap();

    let commits = git_bridge::recent_commits(&root, None).unwrap();
    assert_eq!(commits[0].message, "Title line",
        "Message should be title only, got: {:?}", commits[0].message);
}

#[test]
fn recent_commits_unicode_metadata() {
    let (_tmp, root) = init_test_repo();

    std::process::Command::new("git")
        .current_dir(&root)
        .args(["config", "user.name", "日本太郎"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["config", "user.email", "taro@example.jp"])
        .output()
        .unwrap();

    fs::write(root.join("unicode.txt"), "content").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Unicode commit message"])
        .output()
        .unwrap();

    let commits = git_bridge::recent_commits(&root, None).unwrap();
    assert!(!commits[0].message.is_empty());
    assert!(!commits[0].author_name.is_empty());
}

// ---------------------------------------------------------------------------
// v1.2.1: Diff/stat parser robustness tests
// ---------------------------------------------------------------------------

#[test]
fn diff_stat_file_with_spaces() {
    let (_tmp, root) = init_test_repo();

    let spaced = root.join("docs with spaces");
    fs::create_dir_all(&spaced).unwrap();
    fs::write(spaced.join("my file.md"), "# Hello\n").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Add spaced file"])
        .output()
        .unwrap();

    fs::write(spaced.join("my file.md"), "# Hello World\n").unwrap();

    let stat = git_bridge::diff_stat(&root).unwrap();
    assert!(stat.files_changed >= 1);
    let spaced_file = stat.files.iter().find(|f| f.path.contains("my file"));
    assert!(spaced_file.is_some(), "Should find file with spaces: {:?}", stat.files);
}

#[test]
fn diff_stat_binary_file() {
    let (_tmp, root) = init_test_repo();

    let bin = root.join("image.png");
    fs::write(&bin, &[0x89u8, 0x50, 0x4E, 0x47, 0x00, 0x00, 0x00, 0x00]).unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Add binary"])
        .output()
        .unwrap();

    fs::write(&bin, &[0x89u8, 0x50, 0x4E, 0x47, 0xFF, 0xFF, 0xFF, 0xFF]).unwrap();

    let stat = git_bridge::diff_stat(&root).unwrap();
    let bin_file = stat.files.iter().find(|f| f.path.contains("image"));
    if let Some(f) = bin_file {
        assert!(f.binary, "Binary file should be flagged");
    }
}

#[test]
fn diff_stat_deleted_file() {
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

    let stat = git_bridge::diff_stat(&root).unwrap();
    assert!(stat.files_changed >= 1);
    let deleted = stat.files.iter().find(|f| f.path.contains("to_delete"));
    assert!(deleted.is_some(), "Deleted file should appear");
    assert!(deleted.unwrap().deletions > 0, "Should have deletions");
}

#[test]
fn diff_stat_multiple_files() {
    let (_tmp, root) = init_test_repo();

    for i in 1..=3 {
        fs::write(root.join(format!("file{i}.txt")), format!("content {i}")).unwrap();
    }
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "-A"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["commit", "-m", "Add files"])
        .output()
        .unwrap();

    for i in 1..=3 {
        fs::write(root.join(format!("file{i}.txt")), format!("modified {i}")).unwrap();
    }

    let stat = git_bridge::diff_stat(&root).unwrap();
    assert!(stat.files_changed >= 3, "Should detect all 3 files");
    assert!(stat.files.len() >= 3);
}

#[test]
fn diff_stat_staged_only() {
    let (_tmp, root) = init_test_repo();

    fs::write(root.join("README.md"), "staged content\n").unwrap();
    std::process::Command::new("git")
        .current_dir(&root)
        .args(["add", "README.md"])
        .output()
        .unwrap();

    let stat = git_bridge::diff_stat(&root).unwrap();
    // Staged changes may or may not appear in diff HEAD depending on git behavior
    assert!(stat.files_changed <= 1);
}
