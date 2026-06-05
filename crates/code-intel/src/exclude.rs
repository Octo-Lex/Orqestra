//! File exclusion rules for symbol extraction.
//!
//! Pure functions — no I/O, no dependencies on git-bridge.
//! Duplicates narrow path-risk logic locally to avoid cross-crate dependency.

use crate::ParseStatus;

/// Maximum file size for parsing (256 KiB).
const MAX_FILE_SIZE_BYTES: usize = 262_144;

/// Directories that are always excluded from parsing.
const EXCLUDED_DIRS: &[&str] = &[
    "target/",
    "node_modules/",
    "dist/",
    "build/",
    "coverage/",
    ".git/",
    ".Orqestra/",
];

/// Secret-risk filename patterns.
const SECRET_PATTERNS: &[&str] = &[
    ".env", ".env.local", ".env.production", ".env.staging", ".env.development",
    ".pem", ".key", ".p12", ".pfx", ".p8",
    "id_rsa", "id_ed25519", "id_ecdsa",
];

/// Check if a path should be excluded from symbol extraction.
/// Returns Some(ParseStatus) if excluded, None if parsing should proceed.
pub fn check_excluded(path: &str, source: &str) -> Option<ParseStatus> {
    let normalized = path.replace("\\", "/");
    let lower = path.to_lowercase();

    // Generated/vendor directories
    for dir in EXCLUDED_DIRS {
        if normalized.contains(dir) {
            return Some(ParseStatus::Excluded);
        }
    }

    // Secret-risk paths
    let filename = normalized.rsplit('/').next().unwrap_or(&normalized);
    if SECRET_PATTERNS.iter().any(|p| lower.contains(p)) {
        return Some(ParseStatus::Secret);
    }
    if lower.starts_with("secrets.") || lower.starts_with("credentials.") {
        return Some(ParseStatus::Secret);
    }

    // Binary detection by extension
    let binary_exts = [
        ".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".ico",
        ".exe", ".dll", ".so", ".dylib", ".bin",
        ".zip", ".tar", ".gz", ".7z",
        ".pdf", ".doc", ".docx",
        ".woff", ".woff2", ".ttf", ".eot",
        ".mp3", ".mp4", ".avi",
    ];
    if binary_exts.iter().any(|ext| lower.ends_with(ext)) {
        return Some(ParseStatus::Binary);
    }

    // Size check
    if source.len() > MAX_FILE_SIZE_BYTES {
        return Some(ParseStatus::TooLarge);
    }

    // Only parse supported languages
    let is_supported = lower.ends_with(".rs")
        || lower.ends_with(".ts")
        || lower.ends_with(".tsx")
        || lower.ends_with(".js")
        || lower.ends_with(".jsx");
    if !is_supported {
        return Some(ParseStatus::Excluded);
    }

    None
}
