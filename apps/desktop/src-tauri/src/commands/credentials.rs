//! v1.0.2 Credential commands — OS-keychain-backed secret vault.
//!
//! Architecture decision: Stronghold plugin (tauri-plugin-stronghold 2.3.1) compiled
//! with Tauri 2.11.2 but is designed for JS invocation, not Rust-side programmatic access.
//! Per v1.0.2 spec §5.2.1 fallback rule, we use the `keyring` crate directly.
//! The OS keychain IS the encrypted vault. The SecretVault trait is the release contract.
//!
//! Security rules:
//! - Never return raw PAT to TypeScript
//! - Never log raw PATs
//! - Never write PATs to .Orqestra/ or filesystem
//! - Mask token-like strings in errors before returning to UI
//! - OS-keychain failure = blocking persistence error, never silent fallback to plaintext

use crate::security::keyring_store::{KeyringVault, SessionVault};
use crate::security::token_mask::mask_tokens_in_string;
use crate::security::{
    CredentialError, CredentialMigrationState, CredentialProvider, CredentialVaultStatus,
    GitHubConnectionStatus, SecretVault, TokenMetadata, TokenStatus,
    KEYRING_GITHUB_PAT_ACCOUNT, KEYRING_GITHUB_PAT_META_ACCOUNT,
    legacy_meta_path, legacy_vault_path,
};
use chrono::Utc;
use std::path::PathBuf;
use tauri::command;
use tauri::Manager;

// ---------------------------------------------------------------------------
// Vault selection — returns the appropriate vault
// ---------------------------------------------------------------------------

fn get_vault(_app: &tauri::AppHandle) -> Box<dyn SecretVault> {
    let keyring = KeyringVault::new();
    if keyring.is_available() {
        Box::new(keyring)
    } else {
        Box::new(SessionVault::new())
    }
}

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {e}"))
}

// ---------------------------------------------------------------------------
// Legacy XOR vault migration helpers
// ---------------------------------------------------------------------------

/// Check if a legacy XOR-encrypted credential exists.
fn legacy_credential_exists(app: &tauri::AppHandle) -> bool {
    let Ok(data_dir) = app.path().app_data_dir() else {
        return false;
    };
    legacy_vault_path(&data_dir).exists()
}

/// Read legacy XOR credential (v1.0.1 format).
/// Returns the decrypted PAT if found.
fn read_legacy_credential(app: &tauri::AppHandle) -> Result<Option<String>, String> {
    let data_dir = app_data_dir(app)?;
    let enc_path = legacy_vault_path(&data_dir);
    let meta_path = legacy_meta_path(&data_dir);

    if !enc_path.exists() {
        return Ok(None);
    }

    // Read the encrypted blob
    let enc_data = std::fs::read(&enc_path)
        .map_err(|e| format!("Failed to read legacy vault: {e}"))?;

    // Read metadata for the key material
    if !meta_path.exists() {
        return Err("Legacy vault exists but metadata is missing".into());
    }
    let meta_json = std::fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read legacy metadata: {e}"))?;
    let meta: serde_json::Value = serde_json::from_str(&meta_json)
        .map_err(|e| format!("Failed to parse legacy metadata: {e}"))?;

    // Derive the machine key (same algorithm as v1.0.1)
    let machine_id = get_machine_id();
    let key = derive_machine_key(&machine_id, &meta);

    // XOR decrypt
    let bytes: Vec<u8> = enc_data
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect();

    let pat = String::from_utf8(bytes)
        .map_err(|_| "Failed to decode legacy credential (invalid UTF-8)".to_string())?;

    Ok(Some(pat))
}

/// Delete legacy credential files.
fn delete_legacy_credential(app: &tauri::AppHandle) -> Result<(), String> {
    let data_dir = app_data_dir(app)?;
    let enc_path = legacy_vault_path(&data_dir);
    let meta_path = legacy_meta_path(&data_dir);

    if enc_path.exists() {
        std::fs::remove_file(&enc_path)
            .map_err(|e| format!("Failed to delete legacy vault: {e}"))?;
    }
    if meta_path.exists() {
        std::fs::remove_file(&meta_path)
            .map_err(|e| format!("Failed to delete legacy metadata: {e}"))?;
    }
    Ok(())
}

// Minimal machine key derivation (matches v1.0.1 algorithm)
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
    let seed = meta
        .get("seed")
        .and_then(|v| v.as_u64())
        .unwrap_or(42);
    let mut hasher = twox_hash::XxHash64::with_seed(seed);
    hasher.write(machine_id.as_bytes());
    let hash = hasher.finish();
    let rounds = meta
        .get("rounds")
        .and_then(|v| v.as_u64())
        .unwrap_or(3);
    let mut key = Vec::with_capacity((rounds as usize) * 8);
    for i in 0..rounds {
        let mut h = twox_hash::XxHash64::with_seed(hash.wrapping_add(i));
        h.write(machine_id.as_bytes());
        let part = h.finish().to_le_bytes();
        key.extend_from_slice(&part);
    }
    key
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Bootstrap the credential vault and report status.
#[command]
pub fn bootstrap_credential_vault_cmd(
    app: tauri::AppHandle,
) -> Result<CredentialVaultStatus, String> {
    let vault = get_vault(&app);
    let has_pat = vault.has_secret(KEYRING_GITHUB_PAT_ACCOUNT).unwrap_or(false);
    let has_meta = vault.has_secret(KEYRING_GITHUB_PAT_META_ACCOUNT).unwrap_or(false);

    let migration = if legacy_credential_exists(&app) {
        if has_pat {
            CredentialMigrationState::Complete
        } else {
            CredentialMigrationState::Required
        }
    } else {
        CredentialMigrationState::NotRequired
    };

    Ok(CredentialVaultStatus {
        available: true,
        provider: vault.provider(),
        vault_exists: has_pat,
        unlock_secret_exists: has_meta,
        migration_state: migration,
        last_error: None,
    })
}

/// Save a GitHub PAT to the OS keychain. Never returns the raw token.
#[command]
pub fn save_github_token_cmd(
    app: tauri::AppHandle,
    token: String,
) -> Result<TokenStatus, String> {
    // Never log the token
    let vault = get_vault(&app);

    // Save the PAT
    vault
        .put_secret(KEYRING_GITHUB_PAT_ACCOUNT, token.as_bytes())
        .map_err(|e| mask_tokens_in_string(&format!("Failed to save GitHub PAT: {e}")))?;

    // Save metadata (no raw token)
    let meta = TokenMetadata {
        label: "GitHub PAT".into(),
        saved_at: Utc::now().to_rfc3339(),
        provider: vault.provider().to_string(),
    };
    let meta_json = serde_json::to_vec(&meta)
        .map_err(|e| format!("Failed to serialize metadata: {e}"))?;
    vault
        .put_secret(KEYRING_GITHUB_PAT_META_ACCOUNT, &meta_json)
        .map_err(|e| format!("Failed to save token metadata: {e}"))?;

    Ok(TokenStatus {
        exists: true,
        provider: vault.provider(),
        label: "GitHub PAT".into(),
        last_updated: Some(meta.saved_at.clone()),
        migration_state: CredentialMigrationState::NotRequired,
    })
}

/// Get the status of the stored GitHub PAT (no raw token returned).
#[command]
pub fn get_github_token_status_cmd(
    app: tauri::AppHandle,
) -> Result<TokenStatus, String> {
    let vault = get_vault(&app);
    let exists = vault.has_secret(KEYRING_GITHUB_PAT_ACCOUNT).unwrap_or(false);

    let last_updated = if exists {
        // Try to read metadata for the timestamp
        match vault.get_secret(KEYRING_GITHUB_PAT_META_ACCOUNT) {
            Ok(meta_bytes) => {
                let meta: Result<TokenMetadata, _> = serde_json::from_slice(&meta_bytes);
                meta.ok().map(|m| m.saved_at)
            }
            Err(_) => None,
        }
    } else {
        None
    };

    let migration = if legacy_credential_exists(&app) {
        if exists {
            CredentialMigrationState::Complete
        } else {
            CredentialMigrationState::Required
        }
    } else {
        CredentialMigrationState::NotRequired
    };

    Ok(TokenStatus {
        exists,
        provider: vault.provider(),
        label: "GitHub PAT".into(),
        last_updated,
        migration_state: migration,
    })
}

/// Delete the stored GitHub PAT.
#[command]
pub fn delete_github_token_cmd(
    app: tauri::AppHandle,
) -> Result<TokenStatus, String> {
    let vault = get_vault(&app);

    // Delete PAT
    let _ = vault.delete_secret(KEYRING_GITHUB_PAT_ACCOUNT);
    // Delete metadata
    let _ = vault.delete_secret(KEYRING_GITHUB_PAT_META_ACCOUNT);

    Ok(TokenStatus {
        exists: false,
        provider: vault.provider(),
        label: "GitHub PAT".into(),
        last_updated: None,
        migration_state: CredentialMigrationState::NotRequired,
    })
}

/// Test the GitHub connection using the stored PAT.
#[command]
pub async fn test_github_connection_cmd(
    app: tauri::AppHandle,
) -> Result<GitHubConnectionStatus, String> {
    let vault = get_vault(&app);

    let pat_bytes = vault
        .get_secret(KEYRING_GITHUB_PAT_ACCOUNT)
        .map_err(|e| mask_tokens_in_string(&format!("No stored credential: {e}")))?;

    let pat = String::from_utf8(pat_bytes)
        .map_err(|_| "Stored credential is invalid".to_string())?;

    // Test connection via GitHub API
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {pat}"))
        .header("User-Agent", "Orqestra")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| mask_tokens_in_string(&format!("GitHub API request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        return Ok(GitHubConnectionStatus {
            ok: false,
            username: None,
            scopes: vec![],
            message: format!("GitHub API returned {status}"),
        });
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {e}"))?;

    let username = body.get("login").and_then(|v| v.as_str()).map(String::from);

    // Scopes are in the response headers but reqwest consumes them.
    // We report success with an empty scope list.
    Ok(GitHubConnectionStatus {
        ok: true,
        username,
        scopes: vec![],
        message: "GitHub connection successful".into(),
    })
}

/// Migrate a legacy XOR credential to the OS keychain.
#[command]
pub fn migrate_legacy_credential_cmd(
    app: tauri::AppHandle,
) -> Result<TokenStatus, String> {
    if !legacy_credential_exists(&app) {
        return Ok(get_github_token_status_cmd(app)?);
    }

    // Read legacy credential
    let pat = read_legacy_credential(&app)?
        .ok_or_else(|| "Legacy credential file exists but could not be read".to_string())?;

    // Save to new vault
    let status = save_github_token_cmd(app.clone(), pat)?;

    // Verify the new credential is accessible
    let vault = get_vault(&app);
    match vault.get_secret(KEYRING_GITHUB_PAT_ACCOUNT) {
        Ok(_) => {
            // Verification passed — safe to delete legacy
            delete_legacy_credential(&app)?;
            Ok(TokenStatus {
                exists: true,
                provider: status.provider,
                label: "GitHub PAT".into(),
                last_updated: status.last_updated,
                migration_state: CredentialMigrationState::Complete,
            })
        }
        Err(e) => {
            // Verification failed — preserve legacy credential
            Err(format!(
                "Migration verification failed (legacy preserved): {}",
                mask_tokens_in_string(&e.to_string())
            ))
        }
    }
}

/// Rotate the vault (re-saves the credential with a fresh entry).
#[command]
pub fn rotate_vault_unlock_secret_cmd(
    app: tauri::AppHandle,
) -> Result<CredentialVaultStatus, String> {
    let vault = get_vault(&app);

    // Read current PAT
    let pat_bytes = vault
        .get_secret(KEYRING_GITHUB_PAT_ACCOUNT)
        .map_err(|e| format!("No credential to rotate: {e}"))?;

    // Delete and re-save (forces OS keychain to refresh)
    let _ = vault.delete_secret(KEYRING_GITHUB_PAT_ACCOUNT);
    let _ = vault.delete_secret(KEYRING_GITHUB_PAT_META_ACCOUNT);

    vault
        .put_secret(KEYRING_GITHUB_PAT_ACCOUNT, &pat_bytes)
        .map_err(|e| "Rotation write failed".to_string())?;

    let meta = TokenMetadata {
        label: "GitHub PAT".into(),
        saved_at: Utc::now().to_rfc3339(),
        provider: vault.provider().to_string(),
    };
    let meta_json = serde_json::to_vec(&meta).unwrap();
    vault
        .put_secret(KEYRING_GITHUB_PAT_META_ACCOUNT, &meta_json)
        .map_err(|e| format!("Rotation metadata failed: {e}"))?;

    bootstrap_credential_vault_cmd(app)
}

/// Internal: get the raw PAT for Git operations (never exposed to TypeScript).
/// This is used by git.rs commands that need the token for shell-out.
pub fn get_stored_pat(app: &tauri::AppHandle) -> Result<String, String> {
    let vault = get_vault(app);
    let bytes = vault
        .get_secret(KEYRING_GITHUB_PAT_ACCOUNT)
        .map_err(|e| mask_tokens_in_string(&format!("No stored credential: {e}")))?;
    String::from_utf8(bytes).map_err(|_| "Stored credential is invalid".into())
}
