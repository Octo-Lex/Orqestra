//! Repository snapshot — read-only composite view of Git state.
//!
//! Composes branch/HEAD metadata, working-tree status, and changed-file
//! summary into a single DTO for desktop consumption.

use crate::error::GitBridgeError;
use crate::gix_ops::{self, NativeGitStatus};
use serde::Serialize;
use std::path::Path;
use std::time::Instant;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// HEAD commit metadata.
#[derive(Debug, Clone, Serialize)]
pub struct GitHeadMetadata {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: String,
    pub detached: bool,
}

/// Per-file risk classification.
///
/// `file_kind` describes the file's detected type (text/binary/large/unknown).
/// `risk` describes the safety classification (normal/secret/workflow/binary/large/unknown).
/// These are orthogonal: a `.env` file has `file_kind: unknown` and `risk: secret`.
#[derive(Debug, Clone, Serialize)]
pub struct GitChangedFile {
    pub path: String,
    pub status: String,       // modified/added/deleted/renamed/untracked
    pub staged: bool,
    pub file_kind: String,    // text/binary/large/unknown
    pub risk: String,         // normal/secret/workflow/binary/large/unknown
    pub risk_reason: Option<String>,
}

/// Composite repository snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct GitRepositorySnapshot {
    pub repo_root: String,
    pub branch: String,
    pub head: Option<GitHeadMetadata>,
    pub dirty: bool,
    pub staged_count: u32,
    pub unstaged_count: u32,
    pub untracked_count: u32,
    pub changed_files: Vec<GitChangedFile>,
    pub provider: String,
    pub fallback_used: bool,
    pub parity_status: String,
    pub latency_ms: u64,
    pub diagnostics: Vec<String>,
}

// ---------------------------------------------------------------------------
// Risk classification
// ---------------------------------------------------------------------------

const SECRET_PATTERNS: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    ".env.staging",
    ".env.development",
    ".env.test",
];

const SECRET_EXTENSIONS: &[&str] = &[
    ".pem", ".key", ".p12", ".pfx", ".p8",
];

const SECRET_FILENAMES: &[&str] = &[
    "id_rsa", "id_ed25519", "id_ecdsa", "id_dsa",
];

/// Classify a file's risk based on path alone.
/// Never opens the file for classification.
pub fn classify_risk_by_path(path: &str) -> (&'static str, Option<String>) {
    let filename = path.rsplit('/').next().unwrap_or(path);
    let lower = filename.to_lowercase();

    // Exact secret filename match
    if SECRET_FILENAMES.iter().any(|s| lower == *s) {
        return ("secret", Some("secret-like filename".into()));
    }

    // .env and variants
    if SECRET_PATTERNS.iter().any(|p| lower.starts_with(&format!("{}.", p.trim_start_matches('.')))) {
        return ("secret", Some("env-like path".into()));
    }
    // Exact .env match (no extension)
    if lower == ".env" || lower.starts_with(".env.") {
        return ("secret", Some("env-like path".into()));
    }

    // Secret extensions
    if SECRET_EXTENSIONS.iter().any(|ext| lower.ends_with(ext)) {
        return ("secret", Some("secret-like extension".into()));
    }

    // Workflow risk
    if path.contains(".github/workflows/") {
        return ("workflow", Some("workflow definition path".into()));
    }

    ("normal", None)
}

/// Detect binary file by sampling first 8 KiB.
///
/// Safety constraints:
/// - Reads at most 8192 bytes
/// - Skips files above 10 MiB (classified as "large" by metadata)
/// - Never reads secret-risk paths
/// - Never follows symlinks
const BINARY_SAMPLE_SIZE: usize = 8192;
const LARGE_FILE_THRESHOLD: u64 = 10 * 1024 * 1024; // 10 MiB

fn detect_file_kind(repo_root: &Path, relative_path: &str, risk: &str) -> String {
    // Never open secret-risk files
    if risk == "secret" {
        return "unknown".into();
    }

    let file_path = repo_root.join(relative_path);

    // Never follow symlinks
    if file_path.symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false) {
        return "unknown".into();
    }

    // Check size via metadata
    let metadata = match std::fs::metadata(&file_path) {
        Ok(m) => m,
        Err(_) => return "unknown".into(),
    };

    if metadata.len() > LARGE_FILE_THRESHOLD {
        return "large".into();
    }

    // For deleted/untracked files that may not exist at this path, return unknown
    if !file_path.exists() {
        return "unknown".into();
    }

    // Sample first 8 KiB for null byte detection
    let mut buf = [0u8; BINARY_SAMPLE_SIZE];
    match std::fs::File::open(&file_path) {
        Ok(mut f) => {
            use std::io::Read;
            match f.read(&mut buf) {
                Ok(n) => {
                    if buf[..n].contains(&0) {
                        return "binary".into();
                    }
                    "text".into()
                }
                Err(_) => "unknown".into(),
            }
        }
        Err(_) => "unknown".into(),
    }
}

// ---------------------------------------------------------------------------
// Changed-file list from CLI porcelain v2
// ---------------------------------------------------------------------------

/// Parse `git status --porcelain=v2` into per-file entries with risk classification.
pub fn parse_changed_files(
    output: &str,
    repo_root: &Path,
) -> Vec<GitChangedFile> {
    let mut files = Vec::new();

    for line in output.lines() {
        if line.starts_with("1 ") {
            // Ordinary entry: 1 XY sub perm hm noi path
            let parts: Vec<&str> = line.splitn(9, ' ').collect();
            if parts.len() < 9 {
                continue;
            }
            let xy = parts[1];
            let path = parts[8];

            let (staged, status) = match xy.chars().next() {
                Some('M') => (true, "modified"),
                Some('A') => (true, "added"),
                Some('D') => (true, "deleted"),
                Some('R') => (true, "renamed"),
                Some('.') => match xy.chars().nth(1) {
                    Some('M') => (false, "modified"),
                    Some('D') => (false, "deleted"),
                    _ => (false, "unknown"),
                },
                _ => (false, "unknown"),
            };

            let (risk, risk_reason) = classify_risk_by_path(path);
            let file_kind = detect_file_kind(repo_root, path, risk);

            files.push(GitChangedFile {
                path: path.into(),
                status: status.into(),
                staged,
                file_kind,
                risk: risk.into(),
                risk_reason,
            });
        } else if line.starts_with("u ") {
            // Unmerged entry — mark as unknown risk
            let parts: Vec<&str> = line.splitn(10, ' ').collect();
            let path = parts.last().unwrap_or(&"");
            files.push(GitChangedFile {
                path: path.to_string(),
                status: "unmerged".into(),
                staged: false,
                file_kind: "unknown".into(),
                risk: "unknown".into(),
                risk_reason: Some("unmerged conflict".into()),
            });
        } else if line.starts_with("? ") {
            // Untracked
            let path = &line[2..];
            let (risk, risk_reason) = classify_risk_by_path(path);
            let file_kind = detect_file_kind(repo_root, path, risk);

            files.push(GitChangedFile {
                path: path.into(),
                status: "untracked".into(),
                staged: false,
                file_kind,
                risk: risk.into(),
                risk_reason,
            });
        }
    }

    files
}

// ---------------------------------------------------------------------------
// HEAD metadata via gix
// ---------------------------------------------------------------------------

/// Read HEAD commit metadata using gix.
/// Returns None for unborn branches (no commits yet).
pub fn read_head_metadata(project_root: &Path) -> Result<Option<GitHeadMetadata>, GitBridgeError> {
    let repo = gix::open(project_root)
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to open repo: {e}")))?;

    let head_id = match repo.head_id() {
        Ok(id) => id,
        Err(_) => {
            // Unborn branch — no commits yet
            return Ok(None);
        }
    };

    let detached = repo.head_name().ok().flatten().is_none();
    let obj = repo.find_object(head_id)
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to find HEAD: {e}")))?;

    let commit = obj.try_into_commit()
        .map_err(|e| GitBridgeError::GitOperation(format!("HEAD is not a commit: {e}")))?;

    let message = commit.message()
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to read commit message: {e}")))?;

    let sha = head_id.detach().to_hex().to_string();
    let short_sha = sha.chars().take(7).collect();

    // Extract author from commit signature
    let author = commit.author()
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to read commit author: {e}")))?;
    let identity = author.actor();
    let author_name = identity.name.to_string();
    let author_email = identity.email.to_string();

    // Timestamp
    let time = author.time()
        .map_err(|e| GitBridgeError::GitOperation(format!("Failed to read commit time: {e}")))?;
    let timestamp = format!(
        "{}",
        gix::date::Time::new(time.seconds, time.offset)
    );

    Ok(Some(GitHeadMetadata {
        sha,
        short_sha,
        message: message.title.to_string().trim().to_string(),
        author_name: author_name.to_string().trim().to_string(),
        author_email: author_email.to_string().trim().to_string(),
        timestamp,
        detached,
    }))
}

// ---------------------------------------------------------------------------
// Snapshot command
// ---------------------------------------------------------------------------

/// Build a repository snapshot — composite read-only view.
pub fn repository_snapshot(project_root: &Path) -> Result<GitRepositorySnapshot, GitBridgeError> {
    let start = Instant::now();
    let mut diagnostics = Vec::new();

    // Get status (reuses existing pilot)
    let status = gix_ops::native_git_status(project_root)?;

    // Get HEAD metadata (may be None for fresh repos)
    let head = match read_head_metadata(project_root) {
        Ok(h) => h,
        Err(e) => {
            diagnostics.push(format!("HEAD metadata fallback: {e}"));
            None
        }
    };

    // Get changed files via CLI porcelain (for per-file detail)
    let changed_files = match gix_ops::git_status_porcelain_output(project_root) {
        Ok(output) => parse_changed_files(&output, project_root),
        Err(e) => {
            diagnostics.push(format!("Changed files fallback: {e}"));
            Vec::new()
        }
    };

    let latency_ms = start.elapsed().as_millis() as u64;

    Ok(GitRepositorySnapshot {
        repo_root: project_root.to_string_lossy().to_string(),
        branch: status.branch.clone(),
        head,
        dirty: status.dirty,
        staged_count: status.staged_count,
        unstaged_count: status.unstaged_count,
        untracked_count: status.untracked_count,
        changed_files,
        provider: status.provider.clone(),
        fallback_used: status.fallback_used,
        parity_status: status.parity_status.clone(),
        latency_ms,
        diagnostics,
    })
}
