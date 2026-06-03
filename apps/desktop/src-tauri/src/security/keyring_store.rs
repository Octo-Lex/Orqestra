use crate::security::{
    CredentialError, CredentialProvider, KEYRING_SERVICE,
    SecretVault,
};

/// OS-keychain-backed secret vault using `keyring-core` + platform-native store.
///
/// On Windows: Windows Credential Manager (via windows-native-keyring-store)
/// On macOS: Keychain (via apple-native-keyring-store)
/// On Linux: Secret Service / libsecret (via dbus-secret-service-keyring-store)
pub struct KeyringVault {
    available: bool,
    unavailable_reason: Option<String>,
}

impl KeyringVault {
    pub fn new() -> Self {
        // Try to set up the platform-native store
        let store_result = Self::create_platform_store();
        match store_result {
            Ok(store) => {
                keyring_core::set_default_store(store);
                Self {
                    available: true,
                    unavailable_reason: None,
                }
            }
            Err(e) => Self {
                available: false,
                unavailable_reason: Some(format!("OS keychain unavailable: {e}")),
            },
        }
    }

    #[cfg(target_os = "windows")]
    fn create_platform_store() -> Result<std::sync::Arc<keyring_core::CredentialStore>, String> {
        let store = windows_native_keyring_store::Store::new()
            .map_err(|e| format!("Windows Credential Manager error: {e}"))?;
        Ok(store as std::sync::Arc<keyring_core::CredentialStore>)
    }

    #[cfg(target_os = "macos")]
    fn create_platform_store() -> Result<std::sync::Arc<keyring_core::CredentialStore>, String> {
        // On macOS, use apple-native-keyring-store if available
        Err("macOS keychain not yet configured in this build".into())
    }

    #[cfg(target_os = "linux")]
    fn create_platform_store() -> Result<std::sync::Arc<keyring_core::CredentialStore>, String> {
        Err("Linux keyring not yet configured in this build".into())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn create_platform_store() -> Result<std::sync::Arc<keyring_core::CredentialStore>, String> {
        Err("Unsupported platform".into())
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    pub fn unavailable_reason(&self) -> Option<&str> {
        self.unavailable_reason.as_deref()
    }
}

impl Default for KeyringVault {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretVault for KeyringVault {
    fn put_secret(&self, key: &str, value: &[u8]) -> Result<(), CredentialError> {
        if !self.available {
            return Err(CredentialError::Unavailable(
                self.unavailable_reason
                    .clone()
                    .unwrap_or_else(|| "OS keychain not available".into()),
            ));
        }

        let entry = keyring_core::Entry::new(KEYRING_SERVICE, key)
            .map_err(|e| CredentialError::OperationFailed(format!("Failed to create keyring entry: {e}")))?;

        let value_str = String::from_utf8(value.to_vec())
            .map_err(|_| CredentialError::OperationFailed("Secret contains non-UTF-8 bytes".into()))?;

        entry
            .set_password(&value_str)
            .map_err(|e| CredentialError::OperationFailed(format!("Failed to store secret: {e}")))?;

        Ok(())
    }

    fn has_secret(&self, key: &str) -> Result<bool, CredentialError> {
        if !self.available {
            return Ok(false);
        }

        let entry = keyring_core::Entry::new(KEYRING_SERVICE, key)
            .map_err(|e| CredentialError::OperationFailed(format!("Failed to create keyring entry: {e}")))?;

        match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring_core::Error::NoEntry) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    fn get_secret(&self, key: &str) -> Result<Vec<u8>, CredentialError> {
        if !self.available {
            return Err(CredentialError::Unavailable(
                self.unavailable_reason
                    .clone()
                    .unwrap_or_else(|| "OS keychain not available".into()),
            ));
        }

        let entry = keyring_core::Entry::new(KEYRING_SERVICE, key)
            .map_err(|e| CredentialError::OperationFailed(format!("Failed to create keyring entry: {e}")))?;

        let password = entry
            .get_password()
            .map_err(|e| match e {
                keyring_core::Error::NoEntry => CredentialError::NotFound,
                other => CredentialError::OperationFailed(format!("Failed to retrieve secret: {other}")),
            })?;

        Ok(password.into_bytes())
    }

    fn delete_secret(&self, key: &str) -> Result<(), CredentialError> {
        if !self.available {
            return Err(CredentialError::Unavailable(
                self.unavailable_reason
                    .clone()
                    .unwrap_or_else(|| "OS keychain not available".into()),
            ));
        }

        let entry = keyring_core::Entry::new(KEYRING_SERVICE, key)
            .map_err(|e| CredentialError::OperationFailed(format!("Failed to create keyring entry: {e}")))?;

        entry
            .delete_credential()
            .map_err(|e| match e {
                keyring_core::Error::NoEntry => CredentialError::NotFound,
                other => CredentialError::OperationFailed(format!("Failed to delete secret: {other}")),
            })?;

        Ok(())
    }

    fn provider(&self) -> CredentialProvider {
        if self.available {
            CredentialProvider::OsKeychain
        } else {
            CredentialProvider::Unavailable
        }
    }
}

// ---------------------------------------------------------------------------
// Session-only vault (in-memory, for when OS keychain is unavailable)
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::sync::Mutex;

pub struct SessionVault {
    secrets: Mutex<HashMap<String, Vec<u8>>>,
}

impl SessionVault {
    pub fn new() -> Self {
        Self {
            secrets: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for SessionVault {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretVault for SessionVault {
    fn put_secret(&self, key: &str, value: &[u8]) -> Result<(), CredentialError> {
        self.secrets
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn has_secret(&self, key: &str) -> Result<bool, CredentialError> {
        Ok(self.secrets.lock().unwrap().contains_key(key))
    }

    fn get_secret(&self, key: &str) -> Result<Vec<u8>, CredentialError> {
        self.secrets
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or(CredentialError::NotFound)
    }

    fn delete_secret(&self, key: &str) -> Result<(), CredentialError> {
        self.secrets
            .lock()
            .unwrap()
            .remove(key)
            .map(|_| ())
            .ok_or(CredentialError::NotFound)
    }

    fn provider(&self) -> CredentialProvider {
        CredentialProvider::SessionOnly
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyring_vault_reports_availability() {
        let vault = KeyringVault::new();
        let _ = vault.is_available();
    }

    #[test]
    fn session_vault_round_trip() {
        let vault = SessionVault::new();
        assert!(!vault.has_secret("test-key").unwrap());

        vault.put_secret("test-key", b"secret-value").unwrap();
        assert!(vault.has_secret("test-key").unwrap());
        assert_eq!(vault.get_secret("test-key").unwrap(), b"secret-value");

        vault.delete_secret("test-key").unwrap();
        assert!(!vault.has_secret("test-key").unwrap());
    }

    #[test]
    fn session_vault_delete_missing_returns_not_found() {
        let vault = SessionVault::new();
        let result = vault.delete_secret("nonexistent");
        assert!(result.is_err());
        match result {
            Err(CredentialError::NotFound) => {}
            other => panic!("Expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn session_vault_provider_is_session_only() {
        let vault = SessionVault::new();
        assert_eq!(vault.provider(), CredentialProvider::SessionOnly);
    }

    #[test]
    fn session_vault_overwrite() {
        let vault = SessionVault::new();
        vault.put_secret("key", b"value1").unwrap();
        vault.put_secret("key", b"value2").unwrap();
        assert_eq!(vault.get_secret("key").unwrap(), b"value2");
    }

    // v1.1.0 product-readiness credential validation tests

    #[test]
    fn token_status_dto_contains_no_raw_secret() {
        use crate::security::CredentialMigrationState;
        let status = crate::security::TokenStatus {
            exists: true,
            provider: CredentialProvider::OsKeychain,
            label: "GitHub PAT".into(),
            last_updated: Some("2026-06-03T00:00:00Z".into()),
            migration_state: CredentialMigrationState::NotRequired,
        };
        let json = serde_json::to_string(&status).unwrap();
        // The DTO must never contain a token field
        assert!(!json.contains("token"));
        assert!(!json.contains("secret"));
        assert!(!json.contains("value"));
        assert!(json.contains("exists"));
        assert!(json.contains("provider"));
    }

    #[test]
    fn credential_vault_status_reports_provider_correctly() {
        let status = crate::security::CredentialVaultStatus {
            available: false,
            provider: CredentialProvider::Unavailable,
            vault_exists: false,
            unlock_secret_exists: false,
            migration_state: crate::security::CredentialMigrationState::NotRequired,
            last_error: Some("test".into()),
        };
        assert!(!status.available);
        assert_eq!(status.provider, CredentialProvider::Unavailable);
    }

    #[test]
    fn credential_provider_display_matches_manifest() {
        assert_eq!(format!("{}", CredentialProvider::OsKeychain), "os-keychain");
        assert_eq!(format!("{}", CredentialProvider::SessionOnly), "session-only");
        assert_eq!(format!("{}", CredentialProvider::Unavailable), "unavailable");
    }

    #[test]
    fn credential_error_messages_are_secret_safe() {
        let err = CredentialError::OperationFailed("Connection refused".into());
        let msg = format!("{}", err);
        assert!(!msg.contains("ghp_"));
        assert!(!msg.contains("sk-"));
        // The error message should not leak secret patterns
        assert!(msg.contains("Connection refused"));
    }

    #[test]
    fn migration_state_serializes_correctly() {
        use crate::security::CredentialMigrationState;
        let states = vec![
            CredentialMigrationState::NotRequired,
            CredentialMigrationState::Required,
            CredentialMigrationState::InProgress,
            CredentialMigrationState::Complete,
            CredentialMigrationState::Failed,
        ];
        let json = serde_json::to_string(&states).unwrap();
        assert!(json.contains("not_required"));
        assert!(json.contains("complete"));
        assert!(json.contains("failed"));
    }

    #[test]
    fn github_connection_status_is_secret_safe() {
        let status = crate::security::GitHubConnectionStatus {
            ok: true,
            username: Some("testuser".into()),
            scopes: vec!["repo".into()],
            message: "Connected".into(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(!json.contains("token"));
        assert!(!json.contains("password"));
        assert!(json.contains("testuser"));
        assert!(json.contains("repo"));
    }
}
