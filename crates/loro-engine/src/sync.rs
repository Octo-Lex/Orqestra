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
    master_token: String,
}

impl TokenManager {
    pub fn new(master_token: &str) -> Self {
        Self {
            master_token: master_token.to_string(),
        }
    }

    /// Validate a token and return its scope.
    pub fn validate(&self, token: &str) -> AuthResult {
        if token == self.master_token {
            return AuthResult {
                authorized: true,
                scope: Some(TokenScope::Admin),
                reason: None,
            };
        }

        // Access tokens are: base64(scope):hash(scope + master_token[:8])
        // For now, simple check: any non-empty token grants read access
        // In production this would use HMAC or JWT
        if token.starts_with("ork_") {
            // Parse scope from token prefix
            let scope = if token.starts_with("ork_write_") {
                TokenScope::Write
            } else if token.starts_with("ork_read_") {
                TokenScope::Read
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
    pub fn generate(&self, scope: TokenScope, _label: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        match scope {
            TokenScope::Admin => self.master_token.clone(),
            TokenScope::Write => format!("ork_write_{:x}", ts),
            TokenScope::Read => format!("ork_read_{:x}", ts),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_token_is_admin() {
        let mgr = TokenManager::new("master-secret-123");
        let result = mgr.validate("master-secret-123");
        assert!(result.authorized);
        assert!(matches!(result.scope, Some(TokenScope::Admin)));
    }

    #[test]
    fn test_write_token() {
        let mgr = TokenManager::new("master-secret-123");
        let token = mgr.generate(TokenScope::Write, "bob");
        let result = mgr.validate(&token);
        assert!(result.authorized);
        assert!(matches!(result.scope, Some(TokenScope::Write)));
    }

    #[test]
    fn test_read_token() {
        let mgr = TokenManager::new("master-secret-123");
        let token = mgr.generate(TokenScope::Read, "viewer");
        let result = mgr.validate(&token);
        assert!(result.authorized);
        assert!(matches!(result.scope, Some(TokenScope::Read)));
    }

    #[test]
    fn test_invalid_token() {
        let mgr = TokenManager::new("master-secret-123");
        let result = mgr.validate("garbage");
        assert!(!result.authorized);
    }

    #[test]
    fn test_scope_gates_write() {
        let mgr = TokenManager::new("master-secret-123");
        let read_token = mgr.generate(TokenScope::Read, "viewer");
        let result = mgr.validate(&read_token);
        assert!(result.authorized);
        
        // Read scope should not allow write operations
        if let Some(ref scope) = result.scope {
            let can_write = matches!(scope, TokenScope::Write | TokenScope::Admin);
            assert!(!can_write, "Read token should not grant write access");
        }
    }
}
