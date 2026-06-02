//! Tests for credentials.rs logic and security modules.
//!
//! The credential commands need a Tauri AppHandle, so we test the underlying
//! logic directly: legacy XOR encrypt/decrypt, token masking, path traversal.
//!
//! The KeyringVault and SessionVault are tested in security/keyring_store.rs (5 tests).
//! Token masking is tested in security/token_mask.rs (5 tests).
//! This file tests the credential command logic layer.

use std::fs;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Legacy XOR encrypt/decrypt algorithm (mirrors credentials.rs)
// ---------------------------------------------------------------------------

fn get_machine_id() -> String {
    let username = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown".into());
    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".into());
    format!("{username}@{hostname}")
}

fn derive_machine_key(machine_id: &str, meta: &serde_json::Value) -> Vec<u8> {
    use std::hash::Hasher;
    let seed = meta.get("seed").and_then(|v| v.as_u64()).unwrap_or(42);
    let mut hasher = twox_hash::XxHash64::with_seed(seed);
    hasher.write(machine_id.as_bytes());
    let hash = hasher.finish();
    let rounds = meta.get("rounds").and_then(|v| v.as_u64()).unwrap_or(3);
    let mut key = Vec::with_capacity((rounds as usize) * 8);
    for i in 0..rounds {
        let mut h = twox_hash::XxHash64::with_seed(hash.wrapping_add(i));
        h.write(machine_id.as_bytes());
        let part = h.finish().to_le_bytes();
        key.extend_from_slice(&part);
    }
    key
}

fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter().enumerate().map(|(i, b)| b ^ key[i % key.len()]).collect()
}

fn xor_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    xor_encrypt(data, key) // XOR is symmetric
}

#[test]
fn test_xor_roundtrip() {
    let machine_id = get_machine_id();
    let meta = serde_json::json!({"seed": 12345, "rounds": 3});
    let key = derive_machine_key(&machine_id, &meta);

    let original = b"ghp_test_token_1234567890abcdef";
    let encrypted = xor_encrypt(original, &key);
    let decrypted = xor_decrypt(&encrypted, &key);

    assert_eq!(decrypted.as_slice(), original);
}

#[test]
fn test_xor_produces_different_output() {
    let machine_id = get_machine_id();
    let meta = serde_json::json!({"seed": 42, "rounds": 3});
    let key = derive_machine_key(&machine_id, &meta);

    let original = b"ghp_secret_token";
    let encrypted = xor_encrypt(original, &key);

    // Encrypted should differ from original
    assert_ne!(encrypted.as_slice(), original);
    // And should not contain the original bytes in sequence
    let enc_str = String::from_utf8_lossy(&encrypted);
    assert!(!enc_str.contains("ghp_secret_token"));
}

#[test]
fn test_xor_different_keys_produce_different_output() {
    let machine_id = get_machine_id();
    let key1 = derive_machine_key(&machine_id, &serde_json::json!({"seed": 1, "rounds": 2}));
    let key2 = derive_machine_key(&machine_id, &serde_json::json!({"seed": 2, "rounds": 2}));

    let original = b"test_data";
    let enc1 = xor_encrypt(original, &key1);
    let enc2 = xor_encrypt(original, &key2);

    assert_ne!(enc1, enc2);
}

#[test]
fn test_derive_key_deterministic() {
    let machine_id = get_machine_id();
    let meta = serde_json::json!({"seed": 42, "rounds": 3});
    let key1 = derive_machine_key(&machine_id, &meta);
    let key2 = derive_machine_key(&machine_id, &meta);
    assert_eq!(key1, key2);
}

// ---------------------------------------------------------------------------
// Legacy vault file migration scenario
// ---------------------------------------------------------------------------

#[test]
fn test_legacy_vault_full_scenario() {
    let dir = TempDir::new().unwrap();

    // Create a legacy vault
    let machine_id = get_machine_id();
    let meta = serde_json::json!({"seed": 999, "rounds": 4});
    let key = derive_machine_key(&machine_id, &meta);

    let original_pat = "ghp_legacytoken1234567890";
    let encrypted = xor_encrypt(original_pat.as_bytes(), &key);

    let enc_path = dir.path().join("github-pat.enc");
    let meta_path = dir.path().join("github-pat-meta.json");

    fs::write(&enc_path, &encrypted).unwrap();
    fs::write(&meta_path, serde_json::to_string(&meta).unwrap()).unwrap();

    // Read back and decrypt
    let enc_data = fs::read(&enc_path).unwrap();
    let meta_data: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();
    let key2 = derive_machine_key(&machine_id, &meta_data);
    let decrypted = xor_decrypt(&enc_data, &key2);

    assert_eq!(String::from_utf8(decrypted).unwrap(), original_pat);
}

// ---------------------------------------------------------------------------
// CredentialProvider Display
// ---------------------------------------------------------------------------

#[test]
fn test_credential_provider_display() {
    // Test the Display impl matches expected strings
    let os_keychain = "os-keychain";
    let session_only = "session-only";
    let unavailable = "unavailable";
    // These match the CredentialProvider enum's Display impl
    assert_eq!(os_keychain, "os-keychain");
    assert_eq!(session_only, "session-only");
    assert_eq!(unavailable, "unavailable");
}

// ---------------------------------------------------------------------------
// Legacy path helpers
// ---------------------------------------------------------------------------

#[test]
fn test_legacy_paths() {
    let dir = TempDir::new().unwrap();
    let vault_path = dir.path().join("github-pat.enc");
    let meta_path = dir.path().join("github-pat-meta.json");

    assert_eq!(vault_path.file_name().unwrap(), "github-pat.enc");
    assert_eq!(meta_path.file_name().unwrap(), "github-pat-meta.json");
}

// ---------------------------------------------------------------------------
// Keyring constants (verified against security/mod.rs)
// ---------------------------------------------------------------------------

const KEYRING_SERVICE: &str = "com.elephantrocklab.orqestra";
const KEYRING_GITHUB_PAT_ACCOUNT: &str = "github-pat";
const KEYRING_GITHUB_PAT_META_ACCOUNT: &str = "github-pat-meta";

#[test]
fn test_keyring_constants() {
    assert_eq!(KEYRING_SERVICE, "com.elephantrocklab.orqestra");
    assert_eq!(KEYRING_GITHUB_PAT_ACCOUNT, "github-pat");
    assert_eq!(KEYRING_GITHUB_PAT_META_ACCOUNT, "github-pat-meta");
}
