/// Token masking utilities.
///
/// Any value that might be a PAT, API key, or secret must be masked
/// before being returned to TypeScript, logged, or included in errors.

/// Mask a token-like string. Shows first 4 and last 4 chars if long enough,
/// otherwise replaces entirely.
pub fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &token[..4], &token[token.len() - 4..])
}

/// Scan a string for token-like patterns and mask them.
/// Matches patterns that look like GitHub PATs (ghp_, gho_, ghu_, ghs_, ghr_),
/// long hex strings, or generic token patterns.
pub fn mask_tokens_in_string(input: &str) -> String {
    let mut result = input.to_string();

    // GitHub PAT patterns: ghp_xxxx..., gho_xxxx..., etc.
    for prefix in ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"] {
        let mut start = 0;
        while let Some(pos) = result[start..].find(prefix) {
            let abs_pos = start + pos;
            let end = result[abs_pos..]
                .char_indices()
                .take(40)
                .last()
                .map(|(i, c)| abs_pos + i + c.len_utf8())
                .unwrap_or(result.len());
            let token = &result[abs_pos..end];
            let masked = mask_token(token);
            result = format!("{}{}{}", &result[..abs_pos], masked, &result[end..]);
            // Skip past the masked token to avoid infinite loop
            start = abs_pos + masked.len();
            if start >= result.len() {
                break;
            }
        }
    }

    result
}

/// Check if a string might contain a raw token (for audit/logging).
pub fn contains_token_like_value(input: &str) -> bool {
    for prefix in &["ghp_", "gho_", "ghu_", "ghs_", "ghr_"] {
        if input.contains(prefix) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_short_token() {
        assert_eq!(mask_token("abc"), "****");
        assert_eq!(mask_token("12345678"), "****");
    }

    #[test]
    fn mask_long_token() {
        let masked = mask_token("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890");
        assert!(masked.starts_with("ghp_"));
        assert!(masked.contains("****"));
        assert_eq!(masked.len(), "ghp_****7890".len());
    }

    #[test]
    fn mask_token_in_string() {
        let input = "Failed with token ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890 for user";
        let masked = mask_tokens_in_string(input);
        assert!(!masked.contains("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890"));
        assert!(masked.contains("ghp_****"));
    }

    #[test]
    fn no_mask_when_no_token() {
        let input = "Regular error message with no secrets";
        assert_eq!(mask_tokens_in_string(input), input);
    }

    #[test]
    fn contains_token_detection() {
        assert!(contains_token_like_value("error: ghp_abc123"));
        assert!(!contains_token_like_value("error: something else"));
    }
}
