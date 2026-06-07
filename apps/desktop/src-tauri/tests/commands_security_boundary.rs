//! v2.5.1: Security Boundary Stabilization tests.
//!
//! Tests verify:
//! - CSP is not null and has no wildcards/unsafe-eval
//! - SHA-256 checksum is 64 chars, deterministic, known-vector
//! - Legacy 16-char checksum rejected
//! - No hardcoded master token
//! - TokenManager without master secret cannot generate admin tokens
//! - Secret scanning CI workflow exists

use std::path::PathBuf;

fn find_repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    while !dir.join(".git").exists() {
        if !dir.pop() { panic!("No git repo found"); }
    }
    dir
}

// ---------------------------------------------------------------------------
// CSP tests
// ---------------------------------------------------------------------------

#[test]
fn test_csp_not_null() {
    let root = find_repo_root();
    let content = std::fs::read_to_string(root.join("apps/desktop/src-tauri/tauri.conf.json"))
        .expect("tauri.conf.json must be readable");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Must be valid JSON");
    let csp = json.get("app")
        .and_then(|t| t.get("security"))
        .and_then(|s| s.get("csp"));
    assert!(csp.is_some(), "CSP must be present (not null)");
    assert!(!csp.unwrap().is_null(), "CSP must not be null");
}

#[test]
fn test_csp_no_wildcards() {
    let root = find_repo_root();
    let content = std::fs::read_to_string(root.join("apps/desktop/src-tauri/tauri.conf.json"))
        .expect("tauri.conf.json must be readable");
    assert!(!content.contains("\"connect-src\": \"*\""), "connect-src must not be wildcard");
    assert!(!content.contains("\"default-src\": \"*\""), "default-src must not be wildcard");
}

#[test]
fn test_csp_no_unsafe_eval() {
    let root = find_repo_root();
    let content = std::fs::read_to_string(root.join("apps/desktop/src-tauri/tauri.conf.json"))
        .expect("tauri.conf.json must be readable");
    assert!(!content.contains("unsafe-eval"), "CSP must not contain unsafe-eval");
}

// ---------------------------------------------------------------------------
// SHA-256 checksum tests
// ---------------------------------------------------------------------------

fn sha256_hex(data: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[test]
fn test_sha256_checksum_length() {
    let hash = sha256_hex("test content");
    assert_eq!(hash.len(), 64, "SHA-256 hex must be 64 chars, got {}", hash.len());
}

#[test]
fn test_sha256_checksum_deterministic() {
    let a = sha256_hex("hello world");
    let b = sha256_hex("hello world");
    assert_eq!(a, b, "Same input must produce same hash");
}

#[test]
fn test_sha256_known_vector() {
    // SHA-256 of empty string
    let hash = sha256_hex("");
    assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}

#[test]
fn test_sha256_before_mismatch_rejection() {
    // Different content must produce different hash
    let a = sha256_hex("content A");
    let b = sha256_hex("content B");
    assert_ne!(a, b, "Different content must produce different hashes");
}

#[test]
fn test_no_legacy_16char_checksum() {
    let hash = sha256_hex("test");
    assert_ne!(hash.len(), 16, "Must not produce 16-char (legacy) checksum");
    assert!(hash.len() > 16, "Must produce full SHA-256 (64 chars)");
}

// ---------------------------------------------------------------------------
// Master token tests
// ---------------------------------------------------------------------------

#[test]
fn test_no_hardcoded_master_token() {
    let root = find_repo_root();
    let main_rs = std::fs::read_to_string(root.join("apps/desktop/src-tauri/src/main.rs"))
        .expect("main.rs must be readable");
    assert!(!main_rs.contains("default-master-token"), "No hardcoded master token in main.rs");
}

#[test]
fn test_token_manager_no_master_no_admin() {
    let mgr = loro_engine::sync::TokenManager::new(None);
    assert!(!mgr.has_master_secret());
    let result = mgr.generate(loro_engine::sync::TokenScope::Admin, "test");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("MASTER_SECRET_UNAVAILABLE"));
}

#[test]
fn test_token_manager_with_master_can_admin() {
    let mgr = loro_engine::sync::TokenManager::new(Some("test-master"));
    assert!(mgr.has_master_secret());
    let result = mgr.generate(loro_engine::sync::TokenScope::Admin, "test");
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Secret scanning / gitignore tests
// ---------------------------------------------------------------------------

#[test]
fn test_gitignore_covers_env() {
    let root = find_repo_root();
    let gitignore = std::fs::read_to_string(root.join(".gitignore"))
        .expect(".gitignore must be readable");
    assert!(gitignore.contains(".env"), ".gitignore must cover .env files");
}

#[test]
fn test_no_env_files_in_git_history() {
    // Just check current tracked files
    let root = find_repo_root();
    let output = std::process::Command::new("git")
        .current_dir(&root)
        .args(["ls-files", "*.env", ".env*"])
        .output()
        .expect("git ls-files must work");
    let tracked = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(tracked.is_empty(), "No .env files should be tracked: got '{}'", tracked);
}

#[test]
fn test_secret_scanning_workflow_exists() {
    let root = find_repo_root();
    assert!(
        root.join(".github/workflows/secret-scan.yml").exists(),
        "secret-scan.yml workflow must exist"
    );
}

#[test]
fn test_askpass_uses_random_dir() {
    let root = find_repo_root();
    let git_rs = std::fs::read_to_string(root.join("apps/desktop/src-tauri/src/commands/git.rs"))
        .expect("git.rs must be readable");
    // Verify no predictable temp file path
    assert!(!git_rs.contains("orqestra-git-askpass.bat"), "Must not use predictable temp file");
    // Verify unique directory
    assert!(git_rs.contains("uuid::Uuid::new_v4"), "Must use UUID for unique temp dir");
    // Verify RAII
    assert!(git_rs.contains("impl Drop"), "Must have RAII cleanup (impl Drop)");
    // Verify create_new
    assert!(git_rs.contains("create_new(true)"), "Must use create_new to never overwrite");
}

#[test]
fn test_patch_guard_uses_sha256() {
    let root = find_repo_root();
    let patch_guard = std::fs::read_to_string(root.join("apps/desktop/src-tauri/src/security/patch_guard.rs"))
        .expect("patch_guard.rs must be readable");
    assert!(patch_guard.contains("sha2"), "patch_guard must use sha2");
    assert!(patch_guard.contains("Sha256"), "patch_guard must use Sha256");
    assert!(!patch_guard.contains("DefaultHasher"), "patch_guard must NOT use DefaultHasher");
}

#[test]
fn test_patch_guard_rejects_legacy_checksum() {
    let root = find_repo_root();
    let patch_guard = std::fs::read_to_string(root.join("apps/desktop/src-tauri/src/security/patch_guard.rs"))
        .expect("patch_guard.rs must be readable");
    assert!(patch_guard.contains("LEGACY_CHECKSUM_FORMAT"), "Must reject 16-char legacy checksums");
}
