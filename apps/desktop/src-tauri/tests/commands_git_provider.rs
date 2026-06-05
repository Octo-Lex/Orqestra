//! v1.6.0 Git Provider Diagnostics tests.
//!
//! Tests verify:
//! - Provider report completeness (every expected operation appears)
//! - Provider label accuracy per operation
//! - Commit creation labeled gix-hybrid (not gix)
//! - Read-only enforcement (no mutations during diagnostics)
//! - Mutating operations never executed in diagnostics
//! - Empty results still carry provider labels
//! - Non-repo graceful degradation
//! - Git status unchanged before/after diagnostics

use git_bridge::{
    build_provider_report, GitProvider, GitProviderReport, GitOperationProvider,
    recent_commits_with_provider, RecentCommitsResult,
    diff_stat_with_provider, DiffStatResult,
};

/// Helper: find the project root (walk up to find .git).
fn find_repo_root() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    while !dir.join(".git").exists() {
        if !dir.pop() {
            panic!("No git repository found");
        }
    }
    dir
}

// ---------------------------------------------------------------------------
// 1. Provider report completeness
// ---------------------------------------------------------------------------

#[test]
fn provider_report_has_all_expected_operations() {
    let root = find_repo_root();
    let report = build_provider_report(&root).expect("Provider report should succeed");
    let ops: Vec<&str> = report.operations.iter().map(|o| o.operation.as_str()).collect();

    let expected = [
        "head_hash", "branch_name", "recent_commits", "repository_snapshot",
        "changed_file_summary", "diff_stat", "safe_diff_context",
        "semantic_commit_prep", "staging", "commit_creation",
        "push", "pull", "merge",
    ];

    for exp in &expected {
        assert!(ops.contains(exp), "Missing operation: {exp}");
    }
    assert_eq!(report.operations.len(), expected.len(), "Unexpected extra operations");
}

// ---------------------------------------------------------------------------
// 2. Provider label accuracy
// ---------------------------------------------------------------------------

#[test]
fn head_hash_reports_gix() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    let head_hash = report.operations.iter().find(|o| o.operation == "head_hash").unwrap();
    assert_eq!(head_hash.provider, GitProvider::Gix);
}

#[test]
fn diff_stat_reports_cli_fallback() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    let diff = report.operations.iter().find(|o| o.operation == "diff_stat").unwrap();
    assert_eq!(diff.provider, GitProvider::GitCliFallback);
}

#[test]
fn semantic_commit_prep_reports_deterministic_heuristic() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    let scp = report.operations.iter().find(|o| o.operation == "semantic_commit_prep").unwrap();
    assert_eq!(scp.provider, GitProvider::DeterministicHeuristic);
}

// ---------------------------------------------------------------------------
// 3. Commit creation labeled gix-hybrid
// ---------------------------------------------------------------------------

#[test]
fn commit_creation_labeled_gix_hybrid() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    let cc = report.operations.iter().find(|o| o.operation == "commit_creation").unwrap();
    assert_eq!(cc.provider, GitProvider::GixHybrid, "Commit creation must be gix-hybrid, not gix");
}

// ---------------------------------------------------------------------------
// 4. Read-only enforcement
// ---------------------------------------------------------------------------

#[test]
fn all_executed_ops_are_read_only() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    for op in &report.operations {
        if op.executed_in_diagnostics {
            assert!(op.read_only, "Executed op '{}' should be read_only", op.operation);
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Mutating ops not executed
// ---------------------------------------------------------------------------

#[test]
fn mutating_ops_not_executed() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    for op in &report.operations {
        if op.mutates_repository {
            assert!(!op.executed_in_diagnostics,
                "Mutating op '{}' must not be executed in diagnostics", op.operation);
            assert!(op.latency_ms.is_none(),
                "Mutating op '{}' must have latency_ms: null", op.operation);
        }
    }
}

#[test]
fn staging_and_commit_not_executed() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    let staging = report.operations.iter().find(|o| o.operation == "staging").unwrap();
    let commit = report.operations.iter().find(|o| o.operation == "commit_creation").unwrap();
    assert!(!staging.executed_in_diagnostics);
    assert!(!commit.executed_in_diagnostics);
}

// ---------------------------------------------------------------------------
// 6. Latency bounded for executed ops
// ---------------------------------------------------------------------------

#[test]
fn executed_ops_latency_bounded() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    for op in &report.operations {
        if op.executed_in_diagnostics {
            if let Some(latency) = op.latency_ms {
                assert!(latency < 5000, "Op '{}' latency {}ms exceeds 5000ms", op.operation, latency);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Non-repo graceful degradation
// ---------------------------------------------------------------------------

#[test]
fn provider_report_non_repo_degrades_gracefully() {
    let tmp = std::env::temp_dir().join("provider-diagnostics-nonrepo");
    std::fs::create_dir_all(&tmp).ok();

    let result = build_provider_report(&tmp);
    // Should either error or return with repository_valid: false
    if let Ok(report) = result {
        assert!(!report.repository_valid, "Non-repo should report repository_valid: false");
    }
    // Error is also acceptable

    std::fs::remove_dir_all(&tmp).ok();
}

// ---------------------------------------------------------------------------
// 8. No-mutation guarantee
// ---------------------------------------------------------------------------

#[test]
fn git_status_unchanged_after_diagnostics() {
    let root = find_repo_root();

    // Capture status before
    let status_before = git_bridge::native_git_status(&root).expect("Status before should work");
    let dirty_before = status_before.dirty;
    let staged_before = status_before.staged_count;
    let unstaged_before = status_before.unstaged_count;
    let untracked_before = status_before.untracked_count;

    // Run diagnostics
    let _report = build_provider_report(&root).expect("Diagnostics should succeed");

    // Capture status after
    let status_after = git_bridge::native_git_status(&root).expect("Status after should work");

    assert_eq!(dirty_before, status_after.dirty, "Dirty flag changed after diagnostics");
    assert_eq!(staged_before, status_after.staged_count, "Staged count changed after diagnostics");
    assert_eq!(unstaged_before, status_after.unstaged_count, "Unstaged count changed after diagnostics");
    assert_eq!(untracked_before, status_after.untracked_count, "Untracked count changed after diagnostics");
}

// ---------------------------------------------------------------------------
// 9. Empty results carry provider
// ---------------------------------------------------------------------------

#[test]
fn recent_commits_empty_repo_carries_provider() {
    // Create a temp repo with no commits
    let tmp = std::env::temp_dir().join("provider-empty-commits");
    std::fs::remove_dir_all(&tmp).ok();
    std::fs::create_dir_all(&tmp).ok();

    let init = std::process::Command::new("git")
        .current_dir(&tmp)
        .args(["init"])
        .output();
    if init.is_err() || !init.unwrap().status.success() {
        std::fs::remove_dir_all(&tmp).ok();
        return; // git not available
    }

    let result = recent_commits_with_provider(&tmp, Some(10));
    // May error on empty repo, but if it succeeds, provider must be set
    if let Ok(wrapper) = result {
        assert!(!wrapper.provider.is_empty(), "Provider must be set even on empty result");
        assert!(wrapper.commits.is_empty(), "Empty repo should have no commits");
    }

    std::fs::remove_dir_all(&tmp).ok();
}

// ---------------------------------------------------------------------------
// 10. RecentCommitsResult wrapper carries provider on normal repo
// ---------------------------------------------------------------------------

#[test]
fn recent_commits_wrapper_has_provider() {
    let root = find_repo_root();
    let result = recent_commits_with_provider(&root, Some(5)).expect("Should succeed");
    assert!(!result.provider.is_empty(), "Provider must not be empty");
    assert!(result.provider == "gix" || result.provider == "git-cli-fallback",
        "Provider must be gix or git-cli-fallback, got: {}", result.provider);
    assert!(result.latency_ms < 5000, "Latency too high: {}ms", result.latency_ms);
}

// ---------------------------------------------------------------------------
// 11. DiffStatResult wrapper carries provider
// ---------------------------------------------------------------------------

#[test]
fn diff_stat_wrapper_has_provider() {
    let root = find_repo_root();
    let result = diff_stat_with_provider(&root).expect("Should succeed");
    assert_eq!(result.provider, "git-cli-fallback");
    assert!(result.latency_ms < 5000, "Latency too high: {}ms", result.latency_ms);
}

// ---------------------------------------------------------------------------
// 12. Push/pull/merge are not-implemented
// ---------------------------------------------------------------------------

#[test]
fn push_pull_merge_not_implemented() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    for name in &["push", "pull", "merge"] {
        let op = report.operations.iter().find(|o| o.operation == *name).unwrap();
        assert_eq!(op.provider, GitProvider::NotImplemented, "{} should be not-implemented", name);
        assert!(op.mutates_repository, "{} should be marked as mutating", name);
    }
}

// ---------------------------------------------------------------------------
// 13. Snapshot reports gix-hybrid
// ---------------------------------------------------------------------------

#[test]
fn snapshot_reports_gix_hybrid() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    let snap = report.operations.iter().find(|o| o.operation == "repository_snapshot").unwrap();
    assert_eq!(snap.provider, GitProvider::GixHybrid);
}

// ---------------------------------------------------------------------------
// 14. Repository valid flag
// ---------------------------------------------------------------------------

#[test]
fn report_repo_valid_on_real_repo() {
    let root = find_repo_root();
    let report = build_provider_report(&root).unwrap();
    assert!(report.repository_valid, "Real repo should report repository_valid: true");
}
