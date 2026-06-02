//! Diagnostic bundle redaction.
//!
//! Applies pattern-based redaction to remove API keys, tokens, and secrets
//! from diagnostic output before it leaves the application.

use regex_lite::Regex;
use std::collections::HashSet;

/// Patterns that should be redacted from diagnostic output.
const SECRET_PATTERNS: &[&str] = &[
    // Environment variable assignments
    r"(?m)^ZAI_API_KEY=.*$",
    r"(?m)^CLOUDFLARE_API_TOKEN=.*$",
    r"(?m)^CLOUDFLARE_ACCOUNT_ID=.*$",
    r"(?m)^GITHUB_TOKEN=.*$",
    r"(?m)^(?:export\s+)?(?:ZAI_API_KEY|CLOUDFLARE_API_TOKEN|CLOUDFLARE_ACCOUNT_ID|GITHUB_TOKEN)\s*=\s*\S+",
    // Token prefixes
    r"ghp_[A-Za-z0-9]{36,}",
    r"gho_[A-Za-z0-9]{36,}",
    r"ghu_[A-Za-z0-9]{36,}",
    r"ghs_[A-Za-z0-9]{36,}",
    r"ghr_[A-Za-z0-9]{36,}",
    r"sk-[A-Za-z0-9]{20,}",
    // Bearer tokens
    r"Bearer\s+[A-Za-z0-9\-._~+/]+=*",
    // Generic key/secret patterns
    r"(?i)token:\s*\S+",
    r"(?i)password:\s*\S+",
    r"(?i)secret:\s*\S+",
];

/// Redaction label template
const REDACTED_ENV: &str = "[REDACTED:ENV_VAR]";
const REDACTED_TOKEN: &str = "[REDACTED:TOKEN]";
const REDACTED_BEARER: &str = "[REDACTED:BEARER]";
const REDACTED_GENERIC: &str = "[REDACTED]";

/// Result of redacting a diagnostic string.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RedactionResult {
    pub redacted_text: String,
    pub rules_applied: Vec<String>,
    pub redacted_value_count: usize,
}

/// Redact all known secret patterns from a text string.
pub fn redact_text(text: &str) -> RedactionResult {
    let mut result = text.to_string();
    let mut rules_applied = HashSet::new();
    let mut total_count = 0;

    for pattern in SECRET_PATTERNS {
        let re = match Regex::new(pattern) {
            Ok(re) => re,
            Err(_) => continue,
        };

        let before = result.clone();
        let replacement = classify_pattern(pattern);
        result = re.replace_all(&result, replacement.as_str()).to_string();

        if result != before {
            rules_applied.insert(pattern.to_string());
            // Count individual redactions
            let count = re.find_iter(&before).count();
            total_count += count;
        }
    }

    RedactionResult {
        redacted_text: result,
        rules_applied: rules_applied.into_iter().collect(),
        redacted_value_count: total_count,
    }
}

/// Check if text contains any known secret patterns.
pub fn contains_secrets(text: &str) -> bool {
    for pattern in SECRET_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(text) {
                return true;
            }
        }
    }
    false
}

/// Get the list of redaction rule descriptions.
pub fn redaction_rule_descriptions() -> Vec<String> {
    SECRET_PATTERNS
        .iter()
        .map(|p| format!("Pattern: {}", p))
        .collect()
}

fn classify_pattern(pattern: &str) -> String {
    if pattern.contains("ZAI_API_KEY") {
        format!("ZAI_API_KEY={}", REDACTED_ENV)
    } else if pattern.contains("CLOUDFLARE_API_TOKEN") {
        format!("CLOUDFLARE_API_TOKEN={}", REDACTED_ENV)
    } else if pattern.contains("CLOUDFLARE_ACCOUNT_ID") {
        format!("CLOUDFLARE_ACCOUNT_ID={}", REDACTED_ENV)
    } else if pattern.contains("GITHUB_TOKEN") {
        format!("GITHUB_TOKEN={}", REDACTED_ENV)
    } else if pattern.contains("ghp_") || pattern.contains("gho_") || pattern.contains("ghu_") {
        REDACTED_TOKEN.to_string()
    } else if pattern.contains("Bearer") {
        REDACTED_BEARER.to_string()
    } else {
        REDACTED_GENERIC.to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_github_pat() {
        let text = "token=ghp_1234567890abcdefghijklmnoqrstuvwx1234567890";
        let result = redact_text(text);
        assert!(!result.redacted_text.contains("ghp_"));
        assert!(result.redacted_value_count > 0);
    }

    #[test]
    fn redacts_bearer_token() {
        let text = "Authorization: Bearer abc123def456";
        let result = redact_text(text);
        assert!(!result.redacted_text.contains("abc123def456"));
        assert!(result.redacted_text.contains("[REDACTED"));
    }

    #[test]
    fn redacts_env_var_assignment() {
        let text = "ZAI_API_KEY=sk-my-secret-key-1234567890abcdefghij";
        let result = redact_text(text);
        assert!(!result.redacted_text.contains("sk-my-secret"));
    }

    #[test]
    fn redacts_password_field() {
        let text = "password: my-secret-password";
        let result = redact_text(text);
        assert!(!result.redacted_text.contains("my-secret-password"));
    }

    #[test]
    fn preserves_normal_text() {
        let text = "The quick brown fox jumps over the lazy dog";
        let result = redact_text(text);
        assert_eq!(result.redacted_text, text);
        assert_eq!(result.redacted_value_count, 0);
    }

    #[test]
    fn detects_secrets() {
        assert!(contains_secrets("ghp_AbCdEf1234567890AbCdEf1234567890AbCdEf123456"));
        assert!(contains_secrets("Bearer xyz"));
        assert!(!contains_secrets("Hello world"));
    }

    #[test]
    fn redacts_multiple_patterns() {
        let text = "key=ghp_abc123def456ghi789jkl012mno345pqr678\nBearer token123\npassword: secret123";
        let result = redact_text(text);
        assert!(!result.redacted_text.contains("ghp_"));
        assert!(!result.redacted_text.contains("token123"));
        assert!(!result.redacted_text.contains("secret123"));
        assert!(result.redacted_value_count >= 3);
    }
}
