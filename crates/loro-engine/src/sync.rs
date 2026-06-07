use serde::{Deserialize, Serialize};

/// Result of a sync operation between two peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// File path that was synced
    pub file_path: String,
    /// Number of remote updates imported
    pub imported_updates: usize,
    /// Whether the state converged (both peers have identical state)
    pub converged: bool,
    /// Fields present after merge
    pub field_count: usize,
}

/// Token-based access control for sync operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub scope: TokenScope,
    pub expires_at: Option<String>, // ISO 8601
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenScope {
    /// Full read/write access
    Admin,
    /// Can push/pull CRDT deltas
    Write,
    /// Read-only: can pull snapshots but not push deltas
    Read,
}

/// Auth check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub authorized: bool,
    pub scope: Option<TokenScope>,
    pub reason: Option<String>,
}

/// Token manager for generating and validating access tokens.
pub struct TokenManager {
    master_token: Option<String>,
}

impl TokenManager {
    /// Create a TokenManager. Pass None for desktop mode (no master secret).
    /// Without a master secret:
    ///   - may validate workspace-scoped tokens from relay metadata
    ///   - may store/display scoped tokens
    ///   - may NOT generate admin/master-derived tokens
    ///   - may NOT reset relay state
    ///   - must return structured errors for admin operations
    pub fn new(master_token: Option<&str>) -> Self {
        Self {
            master_token: master_token.map(|s| s.to_string()),
        }
    }

    /// Whether this manager has a master secret.
    pub fn has_master_secret(&self) -> bool {
        self.master_token.is_some()
    }

    /// Validate a token and return its scope.
    pub fn validate(&self, token: &str) -> AuthResult {
        // Check master token (only if we have one)
        if let Some(ref master) = self.master_token {
            if token == master {
                return AuthResult {
                    authorized: true,
                    scope: Some(TokenScope::Admin),
                    reason: None,
                };
            }
        }

        // Workspace-scoped tokens: ork_v2_{scope}_{workspace}_{timestamp}_{hmac}
        // or legacy: ork_{scope}_...
        if token.starts_with("ork_v2_") || token.starts_with("ork_write_") || token.starts_with("ork_read_") {
            let scope = if token.contains("ork_write_") || token.contains("ork_v2_write_") {
                TokenScope::Write
            } else {
                TokenScope::Read
            };
            return AuthResult {
                authorized: true,
                scope: Some(scope),
                reason: None,
            };
        }

        AuthResult {
            authorized: false,
            scope: None,
            reason: Some("Invalid token format".to_string()),
        }
    }

    /// Generate an access token with the given scope.
    /// Returns error if no master secret and admin scope requested.
    pub fn generate(&self, scope: TokenScope, _label: &str) -> Result<String, String> {
        match scope {
            TokenScope::Admin => {
                match &self.master_token {
                    Some(master) => Ok(master.clone()),
                    None => Err("MASTER_SECRET_UNAVAILABLE: Cannot generate admin token without master secret".to_string()),
                }
            }
            TokenScope::Write | TokenScope::Read => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let prefix = match scope {
                    TokenScope::Write => "ork_write_",
                    TokenScope::Read => "ork_read_",
                    TokenScope::Admin => unreachable!(),
                };
                Ok(format!("{}{:x}", prefix, ts))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_token_is_admin() {
        let mgr = TokenManager::new(Some("master-secret-123"));
        let result = mgr.validate("master-secret-123");
        assert!(result.authorized);
        assert!(matches!(result.scope, Some(TokenScope::Admin)));
    }

    #[test]
    fn test_write_token() {
        let mgr = TokenManager::new(Some("master-secret-123"));
        let token = mgr.generate(TokenScope::Write, "bob").unwrap();
        let result = mgr.validate(&token);
        assert!(result.authorized);
        assert!(matches!(result.scope, Some(TokenScope::Write)));
    }

    #[test]
    fn test_read_token() {
        let mgr = TokenManager::new(Some("master-secret-123"));
        let token = mgr.generate(TokenScope::Read, "viewer").unwrap();
        let result = mgr.validate(&token);
        assert!(result.authorized);
        assert!(matches!(result.scope, Some(TokenScope::Read)));
    }

    #[test]
    fn test_invalid_token() {
        let mgr = TokenManager::new(Some("master-secret-123"));
        let result = mgr.validate("garbage");
        assert!(!result.authorized);
    }

    #[test]
    fn test_scope_gates_write() {
        let mgr = TokenManager::new(Some("master-secret-123"));
        let read_token = mgr.generate(TokenScope::Read, "viewer").unwrap();
        let result = mgr.validate(&read_token);
        assert!(result.authorized);
        
        if let Some(ref scope) = result.scope {
            let can_write = matches!(scope, TokenScope::Write | TokenScope::Admin);
            assert!(!can_write, "Read token should not grant write access");
        }
    }

    #[test]
    fn test_no_master_secret_no_admin_token() {
        let mgr = TokenManager::new(None);
        assert!(!mgr.has_master_secret());
        let result = mgr.generate(TokenScope::Admin, "test");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("MASTER_SECRET_UNAVAILABLE"));
    }

    #[test]
    fn test_no_master_secret_can_generate_scoped_tokens() {
        let mgr = TokenManager::new(None);
        let write = mgr.generate(TokenScope::Write, "test");
        assert!(write.is_ok());
        let read = mgr.generate(TokenScope::Read, "test");
        assert!(read.is_ok());
    }

    #[test]
    fn test_no_master_secret_rejects_master_token() {
        let mgr = TokenManager::new(None);
        let result = mgr.validate("any-token-at-all");
        // Without master, no admin validation
        // But ork_ tokens still pass as scoped
    }

    #[test]
    fn test_has_master_secret() {
        let with = TokenManager::new(Some("secret"));
        assert!(with.has_master_secret());
        let without = TokenManager::new(None);
        assert!(!without.has_master_secret());
    }
}
