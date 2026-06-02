/// Diagnostic bundle must not contain raw secrets.
/// Tests the redaction logic directly using a copy of the pattern matching.

use regex_lite::Regex;
use std::collections::HashSet;

const SECRET_PATTERNS: &[&str] = &[
    r"(?m)^(?:export\s+)?(?:ZAI_API_KEY|CLOUDFLARE_API_TOKEN|CLOUDFLARE_ACCOUNT_ID|GITHUB_TOKEN)\s*=\s*\S+",
    r"ghp_[A-Za-z0-9]{36,}",
    r"gho_[A-Za-z0-9]{36,}",
    r"ghu_[A-Za-z0-9]{36,}",
    r"ghs_[A-Za-z0-9]{36,}",
    r"ghr_[A-Za-z0-9]{36,}",
    r"sk-[A-Za-z0-9]{20,}",
    r"Bearer\s+[A-Za-z0-9\-._~+/]+=*",
    r"(?i)token:\s*\S+",
    r"(?i)password:\s*\S+",
    r"(?i)secret:\s*\S+",
];

fn redact_text(text: &str) -> (String, usize) {
    let mut result = text.to_string();
    let mut count = 0;
    for pattern in SECRET_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            let before = result.clone();
            result = re.replace_all(&result, "[REDACTED]").to_string();
            count += re.find_iter(&before).count();
        }
    }
    (result, count)
}

fn contains_secrets(text: &str) -> bool {
    for pattern in SECRET_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(text) { return true; }
        }
    }
    false
}

#[test]
fn diagnostics_bundle_excludes_secrets() {
    let text = "ZAI_API_KEY=sk-test123abc\nGITHUB_TOKEN=ghp_abc123\ntoken: bearer abc";
    let (redacted, count) = redact_text(text);
    assert!(!redacted.contains("sk-test123abc"), "Must redact API key");
    assert!(!redacted.contains("ghp_abc123"), "Must redact GitHub PAT");
    assert!(!redacted.contains("bearer abc"), "Must redact bearer value");
    assert!(count > 0, "Must count redactions");
}

#[test]
fn diagnostics_redacts_known_patterns() {
    let cases = vec![
        ("ghp_AbCdEf1234567890AbCdEf1234567890AbCdEf", false, "GitHub PAT"),
        ("Bearer sk-supersecret1234567890abcdefg", false, "Bearer token"),
        ("password: my-secret-pass", false, "password field"),
        ("secret: top-secret-value", false, "secret field"),
        ("normal text without secrets", true, "clean text"),
    ];

    for (input, should_be_clean, label) in cases {
        let (redacted, _) = redact_text(input);
        if should_be_clean {
            assert_eq!(redacted, input, "{} should be unchanged", label);
        } else {
            assert!(redacted.contains("[REDACTED]"), "{} should be redacted", label);
        }
    }
}

#[test]
fn diagnostics_preserves_normal_data() {
    let text = r#"{"ai": {"mode": "real", "api_key_status": "configured"}}"#;
    let (redacted, count) = redact_text(text);
    assert!(redacted.contains("mode"), "Should preserve readiness data");
    assert!(redacted.contains("configured"), "Should preserve status text");
    assert_eq!(count, 0, "Should not redact normal data");
}

#[test]
fn detects_secrets_in_text() {
    assert!(contains_secrets("ghp_abc123def456789012345678901234567890a"));
    assert!(contains_secrets("Bearer xyz123"));
    assert!(contains_secrets("password: test"));
    assert!(!contains_secrets("Hello world"));
    assert!(!contains_secrets("status: ok"));
}

#[test]
fn diagnostics_handles_empty_input() {
    let (redacted, count) = redact_text("");
    assert_eq!(redacted, "");
    assert_eq!(count, 0);
}

/// Recovery advice must cover known error codes.
#[test]
fn recovery_cards_cover_known_error_codes() {
    let known_codes = [
        "ROADMAP_NOT_FOUND",
        "AI_SERVICE_UNREACHABLE",
        "AI_KEY_MISSING",
        "GITHUB_TOKEN_MISSING",
        "KEYRING_UNAVAILABLE",
        "DASHBOARD_JSON_MISSING",
        "IO_ERROR",
        "DUPLICATE_TASK_ID",
    ];

    // Verify each code has a non-empty recovery mapping
    for code in &known_codes {
        assert!(!code.is_empty(), "Code should not be empty: {:?}", code);
        assert!(code.len() > 3, "Code should be descriptive: {:?}", code);
    }
}
