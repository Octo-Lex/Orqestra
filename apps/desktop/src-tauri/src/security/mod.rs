pub mod keyring_store;
pub mod patch_guard;
pub mod token_mask;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("Secure credential storage unavailable: {0}")]
    Unavailable(String),

    #[error("Credential operation failed: {0}")]
    OperationFailed(String),

    #[error("No credential stored")]
    NotFound,

    #[error("IO error on {0}: {1}")]
    Io(PathBuf, #[source] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Provider backing the secret vault.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CredentialProvider {
    /// Secrets stored directly in OS keychain (keyring crate).
    /// This is the v1.0.2 implementation — Stronghold plugin proved
    /// incompatible for Rust-side access (designed for JS invocation).
    /// The OS keychain IS the encrypted vault.
    OsKeychain,
    /// Session-only: secret held in memory, never persisted.
    SessionOnly,
    /// Secure storage unavailable on this machine.
    Unavailable,
}

impl std::fmt::Display for CredentialProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OsKeychain => write!(f, "os-keychain"),
            Self::SessionOnly => write!(f, "session-only"),
            Self::Unavailable => write!(f, "unavailable"),
        }
    }
}

/// Migration state for legacy XOR vault → keyring migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialMigrationState {
    NotRequired,
    Required,
    InProgress,
    Complete,
    Failed,
}

/// Status of the credential vault infrastructure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialVaultStatus {
    pub available: bool,
    pub provider: CredentialProvider,
    pub vault_exists: bool,
    pub unlock_secret_exists: bool,
    pub migration_state: CredentialMigrationState,
    pub last_error: Option<String>,
}

/// Status of a stored GitHub PAT (no raw token exposed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStatus {
    pub exists: bool,
    pub provider: CredentialProvider,
    pub label: String,
    pub last_updated: Option<String>,
    pub migration_state: CredentialMigrationState,
}

/// Result of testing GitHub connectivity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConnectionStatus {
    pub ok: bool,
    pub username: Option<String>,
    pub scopes: Vec<String>,
    pub message: String,
}

/// Metadata stored alongside the PAT for audit (no raw secret).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub label: String,
    pub saved_at: String,
    pub provider: String,
}

/// The secret vault trait. Implementations may use keyring, Stronghold, etc.
pub trait SecretVault: Send + Sync {
    fn put_secret(&self, key: &str, value: &[u8]) -> Result<(), CredentialError>;
    fn has_secret(&self, key: &str) -> Result<bool, CredentialError>;
    fn get_secret(&self, key: &str) -> Result<Vec<u8>, CredentialError>;
    fn delete_secret(&self, key: &str) -> Result<(), CredentialError>;
    fn provider(&self) -> CredentialProvider;
}

// ---------------------------------------------------------------------------
// Keyring constants — stable service/account names
// ---------------------------------------------------------------------------

pub const KEYRING_SERVICE: &str = "com.elephantrocklab.orqestra";
pub const KEYRING_GITHUB_PAT_ACCOUNT: &str = "github-pat";
pub const KEYRING_GITHUB_PAT_META_ACCOUNT: &str = "github-pat-meta";

// ---------------------------------------------------------------------------
// Legacy XOR vault paths (for migration)
// ---------------------------------------------------------------------------

pub fn legacy_vault_path(app_data_dir: &std::path::Path) -> PathBuf {
    app_data_dir.join("github-pat.enc")
}

pub fn legacy_meta_path(app_data_dir: &std::path::Path) -> PathBuf {
    app_data_dir.join("github-pat-meta.json")
}

// ---------------------------------------------------------------------------
// Module-level helpers for readiness checks (no raw secrets exposed)
// ---------------------------------------------------------------------------

use keyring_store::KeyringVault;

/// Check if the OS keychain is available on this platform.
pub fn is_keyring_available() -> bool {
    KeyringVault::new().is_available()
}

/// Check if a GitHub PAT exists in the keyring (no value returned).
pub fn has_github_token() -> Result<bool, CredentialError> {
    let vault = KeyringVault::new();
    if !vault.is_available() {
        return Ok(false);
    }
    vault.has_secret(KEYRING_GITHUB_PAT_ACCOUNT)
}
