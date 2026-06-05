//! Diff/stat read pilot — CLI-backed, labeled read-only fallback.
//!
//! Provides per-file change statistics without exposing file contents.
//! Provider is always "git-cli-fallback" for v1.2.0.

use crate::error::GitBridgeError;
use crate::snapshot::classify_risk_by_path;
use serde::Serialize;
use std::path::Path;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// Per-file diff statistics.
#[derive(Debug, Clone, Serialize)]
pub struct GitDiffFileStat {
    pub path: String,
    pub insertions: u32,
    pub deletions: u32,
    pub binary: bool,
    pub risk: String,
}

/// Aggregate diff/stat summary.
#[derive(Debug, Clone, Serialize)]
pub struct GitDiffStat {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
    pub files: Vec<GitDiffFileStat>,
    pub provider: String,
    pub fallback_used: bool,
    pub parity_status: String,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

/// Read diff/stat using `git diff --stat HEAD`.
///
/// Does not expose file contents. Binary files are detected by git's
/// `Bin ... -> ...` output format. Secret-risk paths are flagged by
/// path classification without reading contents.
pub fn diff_stat(project_root: &Path) -> Result<GitDiffStat, GitBridgeError> {
    // Try diff against HEAD first; if no HEAD (fresh repo), use --no-index diff against /dev/null equivalent
    let output = std::process::Command::new("git")
        .current_dir(project_root)
        .args(["diff", "--stat", "--numstat", "HEAD"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => return Err(GitBridgeError::Io(project_root.to_owned(), e)),
    };

    // HEAD may not exist in fresh repos
    let stdout = if !output.status.success() {
        // Try cached diff for fresh repos
        let cached = std::process::Command::new("git")
            .current_dir(project_root)
            .args(["diff", "--stat", "--numstat", "--cached"])
            .output()
            .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;
        String::from_utf8_lossy(&cached.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };

    let mut files = Vec::new();
    let mut total_insertions = 0u32;
    let mut total_deletions = 0u32;

    for line in stdout.lines() {
        // numstat format: additions\tdeletions\tfilename
        // Binary files show: -\t-\tfilename
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() == 3 {
            let path = parts[2].trim();
            if path.is_empty() {
                continue;
            }

            let (insertions, deletions, binary) = if parts[0] == "-" && parts[1] == "-" {
                (0, 0, true)
            } else {
                let ins: u32 = parts[0].trim().parse().unwrap_or(0);
                let del: u32 = parts[1].trim().parse().unwrap_or(0);
                (ins, del, false)
            };

            let (risk, _) = classify_risk_by_path(path);

            total_insertions += insertions;
            total_deletions += deletions;

            files.push(GitDiffFileStat {
                path: path.into(),
                insertions,
                deletions,
                binary,
                risk: risk.into(),
            });
        }
    }

    let stat = GitDiffStat {
        files_changed: files.len() as u32,
        insertions: total_insertions,
        deletions: total_deletions,
        files,
        provider: "git-cli-fallback".into(),
        fallback_used: true,
        parity_status: "not-tested".into(),
    };
    Ok(stat)
}

/// Response wrapper for diff/stat.
/// Carries provider metadata and latency.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffStatResult {
    pub provider: String,
    pub stat: GitDiffStat,
    pub latency_ms: u64,
}

/// Read diff/stat with provider metadata.
pub fn diff_stat_with_provider(project_root: &Path) -> Result<DiffStatResult, GitBridgeError> {
    let start = std::time::Instant::now();
    let stat = diff_stat(project_root)?;
    Ok(DiffStatResult {
        provider: "git-cli-fallback".into(),
        stat,
        latency_ms: start.elapsed().as_millis() as u64,
    })
}
