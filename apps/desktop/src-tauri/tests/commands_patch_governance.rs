//! v1.7.0 Patch Application Governance tests.
//!
//! Tests verify:
//! - Forbidden paths (secret, workflow, binary, locks, CI) rejected
//! - Path traversal blocked
//! - Outside allowed_paths rejected
//! - Before-checksum mismatch rejected
//! - Valid docs/bugfix patches apply cleanly
//! - Rejection writes audit without file change
//! - Applied patches produce before/after checksums
//! - No auto-commit during patch application
//! - Failed validation leaves files byte-identical
//! - Atomic writes — failed writes leave original unchanged

use std::path::Path;
use orqestra_desktop::security::patch_guard::{
    AgentType, PatchProposal, PatchStatus,
    apply_agent_patch, reject_agent_patch,
};

/// Helper: compute checksum matching patch_guard's hasher.
fn checksum(content: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Helper: create a temp repo with a file.
fn setup_temp_repo(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().to_path_buf();

    // Init git repo
    let init = std::process::Command::new("git")
        .current_dir(&repo)
        .args(["init"])
        .output()
        .unwrap();
    assert!(init.status.success(), "git init failed");

    // Configure git
    let _ = std::process::Command::new("git")
        .current_dir(&repo)
        .args(["config", "user.email", "test@test.com"])
        .output();
    let _ = std::process::Command::new("git")
        .current_dir(&repo)
        .args(["config", "user.name", "Test"])
        .output();

    (dir, repo)
}

/// Helper: write file and return checksum.
fn write_file(repo: &Path, rel_path: &str, content: &str) -> String {
    let full = repo.join(rel_path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&full, content).unwrap();
    checksum(content)
}

fn make_patch(id: &str, path: &str, before: &str, after: &str) -> PatchProposal {
    PatchProposal {
        proposal_id: id.to_string(),
        path: path.to_string(),
        before: before.to_string(),
        after: after.to_string(),
        before_checksum: checksum(before),
        after_checksum: checksum(after),
    }
}

// ---------------------------------------------------------------------------
// 1. Forbidden secret path
// ---------------------------------------------------------------------------

#[test]
fn forbidden_secret_path_env_rejected() {
    let (_dir, repo) = setup_temp_repo("secret-env");
    write_file(&repo, ".env", "SECRET=abc");
    let patch = make_patch("p1", ".env", "SECRET=abc", "SECRET=xyz");
    let result = apply_agent_patch(&repo, &patch, &[], &AgentType::Docs);
    assert_eq!(result.status, PatchStatus::ApplyFailed);
    assert_eq!(result.verification, "forbidden");
    // File unchanged
    assert_eq!(std::fs::read_to_string(repo.join(".env")).unwrap(), "SECRET=abc");
}

#[test]
fn forbidden_secret_path_pem_rejected() {
    let (_dir, repo) = setup_temp_repo("secret-pem");
    write_file(&repo, "cert.pem", "-----BEGIN CERT-----");
    let patch = make_patch("p2", "cert.pem", "-----BEGIN CERT-----", "tampered");
    let result = apply_agent_patch(&repo, &patch, &[], &AgentType::Bugfix);
    assert_eq!(result.status, PatchStatus::ApplyFailed);
}

// ---------------------------------------------------------------------------
// 2. Forbidden workflow path
// ---------------------------------------------------------------------------

#[test]
fn forbidden_workflow_path_rejected() {
    let (_dir, repo) = setup_temp_repo("workflow");
    write_file(&repo, ".github/workflows/ci.yml", "name: CI\n");
    let patch = make_patch("p3", ".github/workflows/ci.yml", "name: CI\n", "name: Hacked\n");
    let result = apply_agent_patch(&repo, &patch, &[], &AgentType::Docs);
    assert_eq!(result.status, PatchStatus::ApplyFailed);
    assert_eq!(result.verification, "forbidden");
}

// ---------------------------------------------------------------------------
// 3. Forbidden binary write
// ---------------------------------------------------------------------------

#[test]
fn forbidden_binary_image_rejected() {
    let (_dir, repo) = setup_temp_repo("binary");
    write_file(&repo, "logo.png", "fake-png");
    let patch = make_patch("p4", "logo.png", "fake-png", "tampered-png");
    let result = apply_agent_patch(&repo, &patch, &[], &AgentType::Docs);
    assert_eq!(result.status, PatchStatus::ApplyFailed);
    assert_eq!(result.verification, "forbidden");
}

// ---------------------------------------------------------------------------
// 4. Forbidden dependency lock
// ---------------------------------------------------------------------------

#[test]
fn forbidden_dependency_lock_rejected() {
    let (_dir, repo) = setup_temp_repo("lock");
    write_file(&repo, "Cargo.lock", "# lockfile");
    let patch = make_patch("p5", "Cargo.lock", "# lockfile", "# tampered");
    let result = apply_agent_patch(&repo, &patch, &[], &AgentType::Bugfix);
    assert_eq!(result.status, PatchStatus::ApplyFailed);
}

// ---------------------------------------------------------------------------
// 5. Path traversal blocked
// ---------------------------------------------------------------------------

#[test]
fn path_traversal_blocked() {
    let (_dir, repo) = setup_temp_repo("traversal");
    let patch = make_patch("p6", "../../etc/passwd", "", "hacked");
    let result = apply_agent_patch(&repo, &patch, &[], &AgentType::Docs);
    assert_eq!(result.status, PatchStatus::ApplyFailed);
}

// ---------------------------------------------------------------------------
// 6. Outside allowed_paths rejected
// ---------------------------------------------------------------------------

#[test]
fn outside_allowed_paths_rejected() {
    let (_dir, repo) = setup_temp_repo("outside-scope");
    write_file(&repo, "docs/guide.md", "old content");
    write_file(&repo, "src/main.rs", "fn main() {}");
    let patch = make_patch("p7", "src/main.rs", "fn main() {}", "fn hacked() {}");
    // UI scope is docs/ only — but docs agent server policy also blocks src/
    let result = apply_agent_patch(&repo, &patch, &["docs/".to_string()], &AgentType::Docs);
    assert_eq!(result.status, PatchStatus::ApplyFailed);
    // Either server-policy-blocked or outside-ui-scope is acceptable
    assert!(result.verification == "server-policy-blocked" || result.verification == "outside-ui-scope",
        "Expected blocked, got: {}", result.verification);
}

// ---------------------------------------------------------------------------
// 7. Before-checksum mismatch
// ---------------------------------------------------------------------------

#[test]
fn before_checksum_mismatch_rejected() {
    let (_dir, repo) = setup_temp_repo("stale");
    write_file(&repo, "docs/guide.md", "current content");
    let mut patch = make_patch("p8", "docs/guide.md", "old content", "new content");
    // Checksum should already be wrong since "old content" != "current content"
    let result = apply_agent_patch(&repo, &patch, &["docs/".to_string()], &AgentType::Docs);
    assert_eq!(result.status, PatchStatus::ApplyFailed);
    assert_eq!(result.verification, "before-checksum-mismatch");
    // File unchanged
    assert_eq!(std::fs::read_to_string(repo.join("docs/guide.md")).unwrap(), "current content");
}

// ---------------------------------------------------------------------------
// 8. Valid docs patch applies cleanly
// ---------------------------------------------------------------------------

#[test]
fn valid_docs_patch_applies() {
    let (_dir, repo) = setup_temp_repo("docs-valid");
    let before_checksum = write_file(&repo, "docs/setup.md", "Setup guide v1");
    let patch = PatchProposal {
        proposal_id: "p9".to_string(),
        path: "docs/setup.md".to_string(),
        before: "Setup guide v1".to_string(),
        after: "Setup guide v2".to_string(),
        before_checksum: before_checksum.clone(),
        after_checksum: checksum("Setup guide v2"),
    };
    let result = apply_agent_patch(&repo, &patch, &["docs/".to_string()], &AgentType::Docs);
    assert_eq!(result.status, PatchStatus::Applied);
    assert_eq!(result.verification, "match");
    assert!(result.after_checksum.is_some());
    // File content matches after
    assert_eq!(std::fs::read_to_string(repo.join("docs/setup.md")).unwrap(), "Setup guide v2");
}

// ---------------------------------------------------------------------------
// 9. Valid bugfix patch applies cleanly
// ---------------------------------------------------------------------------

#[test]
fn valid_bugfix_patch_applies() {
    let (_dir, repo) = setup_temp_repo("bugfix-valid");
    let before_checksum = write_file(&repo, "src/lib.rs", "fn old() {}");
    let patch = PatchProposal {
        proposal_id: "p10".to_string(),
        path: "src/lib.rs".to_string(),
        before: "fn old() {}".to_string(),
        after: "fn fixed() {}".to_string(),
        before_checksum: before_checksum.clone(),
        after_checksum: checksum("fn fixed() {}"),
    };
    let result = apply_agent_patch(&repo, &patch, &["src/".to_string()], &AgentType::Bugfix);
    assert_eq!(result.status, PatchStatus::Applied);
    assert_eq!(std::fs::read_to_string(repo.join("src/lib.rs")).unwrap(), "fn fixed() {}");
}

// ---------------------------------------------------------------------------
// 10. Reject records audit without file change
// ---------------------------------------------------------------------------

#[test]
fn reject_records_audit_no_file_change() {
    let (_dir, repo) = setup_temp_repo("reject");
    let before_checksum = write_file(&repo, "docs/guide.md", "original");
    let patch = PatchProposal {
        proposal_id: "p11".to_string(),
        path: "docs/guide.md".to_string(),
        before: "original".to_string(),
        after: "modified".to_string(),
        before_checksum,
        after_checksum: checksum("modified"),
    };
    let result = reject_agent_patch(&repo, &patch, &AgentType::Docs, "user-rejected");
    assert_eq!(result.status, PatchStatus::Rejected);
    // File unchanged
    assert_eq!(std::fs::read_to_string(repo.join("docs/guide.md")).unwrap(), "original");
    // Audit record exists
    let audit_path = repo.join(".Orqestra/agents/docs/audit.jsonl");
    assert!(audit_path.exists(), "Audit file should exist");
    let audit_content = std::fs::read_to_string(&audit_path).unwrap();
    assert!(audit_content.contains("\"rejected\""), "Audit should contain rejected status");
}

// ---------------------------------------------------------------------------
// 11. Applied patch audit has checksums
// ---------------------------------------------------------------------------

#[test]
fn applied_patch_audit_has_checksums() {
    let (_dir, repo) = setup_temp_repo("audit-checksums");
    let before_checksum = write_file(&repo, "docs/api.md", "API v1");
    let patch = PatchProposal {
        proposal_id: "p12".to_string(),
        path: "docs/api.md".to_string(),
        before: "API v1".to_string(),
        after: "API v2".to_string(),
        before_checksum: before_checksum.clone(),
        after_checksum: checksum("API v2"),
    };
    let result = apply_agent_patch(&repo, &patch, &["docs/".to_string()], &AgentType::Docs);
    assert_eq!(result.status, PatchStatus::Applied);

    let audit_path = repo.join(".Orqestra/agents/docs/audit.jsonl");
    let audit = std::fs::read_to_string(&audit_path).unwrap();
    assert!(audit.contains(&before_checksum), "Audit should contain before_checksum");
    assert!(audit.contains("\"match\""), "Audit should contain verification: match");
}

// ---------------------------------------------------------------------------
// 12. No auto-commit during patch application
// ---------------------------------------------------------------------------

#[test]
fn no_auto_commit_during_patch() {
    let (_dir, repo) = setup_temp_repo("no-autocommit");
    let before_checksum = write_file(&repo, "docs/guide.md", "original");

    // Get HEAD before patch
    let head_before = std::process::Command::new("git")
        .current_dir(&repo)
        .args(["rev-parse", "HEAD"])
        .output();

    let patch = PatchProposal {
        proposal_id: "p13".to_string(),
        path: "docs/guide.md".to_string(),
        before: "original".to_string(),
        after: "patched".to_string(),
        before_checksum,
        after_checksum: checksum("patched"),
    };
    let result = apply_agent_patch(&repo, &patch, &["docs/".to_string()], &AgentType::Docs);
    assert_eq!(result.status, PatchStatus::Applied);

    // HEAD must not have changed (no commit created)
    let head_after = std::process::Command::new("git")
        .current_dir(&repo)
        .args(["rev-parse", "HEAD"])
        .output();

    match (head_before, head_after) {
        (Ok(before), Ok(after)) => {
            assert_eq!(
                String::from_utf8_lossy(&before.stdout).trim(),
                String::from_utf8_lossy(&after.stdout).trim(),
                "HEAD must not change — no auto-commit"
            );
        }
        _ => {} // If git rev-parse fails (e.g., no commits), that's fine
    }
}

// ---------------------------------------------------------------------------
// 13. Failed validation leaves files byte-identical
// ---------------------------------------------------------------------------

#[test]
fn failed_validation_leaves_files_unchanged() {
    let (_dir, repo) = setup_temp_repo("no-side-effect");
    write_file(&repo, "docs/a.md", "content-a");
    write_file(&repo, "docs/b.md", "content-b");
    write_file(&repo, ".env", "secret=123");

    // Snapshot all files
    let a_before = std::fs::read_to_string(repo.join("docs/a.md")).unwrap();
    let b_before = std::fs::read_to_string(repo.join("docs/b.md")).unwrap();
    let env_before = std::fs::read_to_string(repo.join(".env")).unwrap();

    // Try forbidden patch
    let env_cs = checksum("secret=123");
    let forbidden_patch = PatchProposal {
        proposal_id: "p14a".to_string(),
        path: ".env".to_string(),
        before: "secret=123".to_string(),
        after: "secret=hacked".to_string(),
        before_checksum: env_cs,
        after_checksum: checksum("secret=hacked"),
    };
    let _ = apply_agent_patch(&repo, &forbidden_patch, &[], &AgentType::Docs);

    // Try stale patch
    let stale_patch = make_patch("p14b", "docs/a.md", "wrong content", "new content");
    let _ = apply_agent_patch(&repo, &stale_patch, &["docs/".to_string()], &AgentType::Docs);

    // Verify all files are byte-identical
    assert_eq!(std::fs::read_to_string(repo.join("docs/a.md")).unwrap(), a_before);
    assert_eq!(std::fs::read_to_string(repo.join("docs/b.md")).unwrap(), b_before);
    assert_eq!(std::fs::read_to_string(repo.join(".env")).unwrap(), env_before);
}

// ---------------------------------------------------------------------------
// 14. Server-side policy blocks docs agent from source files
// ---------------------------------------------------------------------------

#[test]
fn docs_agent_server_policy_blocks_source() {
    let (_dir, repo) = setup_temp_repo("docs-policy");
    let before_checksum = write_file(&repo, "src/main.rs", "fn main() {}");
    let patch = PatchProposal {
        proposal_id: "p15".to_string(),
        path: "src/main.rs".to_string(),
        before: "fn main() {}".to_string(),
        after: "fn hacked() {}".to_string(),
        before_checksum,
        after_checksum: checksum("fn hacked() {}"),
    };
    // Even though UI allows src/, server-side docs policy blocks it
    let result = apply_agent_patch(&repo, &patch, &["src/".to_string()], &AgentType::Docs);
    assert_eq!(result.status, PatchStatus::ApplyFailed);
    assert_eq!(result.verification, "server-policy-blocked");
    // File unchanged
    assert_eq!(std::fs::read_to_string(repo.join("src/main.rs")).unwrap(), "fn main() {}");
}
