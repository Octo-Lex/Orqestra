//! Recent commit metadata reads — read-only.
//!
//! Reads recent commit history via gix (native) with CLI fallback.
//! No diff body, no remote calls, no credential access.

use crate::error::GitBridgeError;
use serde::Serialize;
use std::path::Path;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// Summary of a single commit.
#[derive(Debug, Clone, Serialize)]
pub struct GitCommitSummary {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: String,
    pub parents: Vec<String>,
    pub provider: String,
}

// ---------------------------------------------------------------------------
// Limits
// ---------------------------------------------------------------------------

const DEFAULT_LIMIT: usize = 10;
const MAX_LIMIT: usize = 100;

// ---------------------------------------------------------------------------
// Native gix path
// ---------------------------------------------------------------------------

fn recent_commits_gix(
    project_root: &Path,
    limit: usize,
) -> Result<Vec<GitCommitSummary>, GitBridgeError> {
    let repo = gix::open(project_root)
        .map_err(|e| GitBridgeError::GitOperation(format!("gix open failed: {e}")))?;

    let head_id = repo.head_id()
        .map_err(|e| GitBridgeError::GitOperation(format!("HEAD unavailable: {e}")))?;

    let first = repo.find_object(head_id)
        .map_err(|e| GitBridgeError::GitOperation(format!("HEAD object not found: {e}")))?
        .try_into_commit()
        .map_err(|e| GitBridgeError::GitOperation(format!("HEAD is not a commit: {e}")))?;

    let mut commits = Vec::new();
    let mut current = Some(first);
    let mut visited = std::collections::HashSet::new();

    while let Some(commit) = current.take() {
        if commits.len() >= limit || visited.contains(&commit.id) {
            break;
        }
        visited.insert(commit.id);

        let sha = commit.id.to_hex().to_string();
        let short_sha: String = sha.chars().take(7).collect();

        let message = commit.message()
            .map(|m| m.title.to_string().trim().to_string())
            .unwrap_or_else(|_| "(no message)".into());

        let author = commit.author()
            .map_err(|e| GitBridgeError::GitOperation(format!("Author read failed: {e}")))?;
        let identity = author.actor();
        let author_name = identity.name.to_string();
        let author_email = identity.email.to_string();

        let time = author.time()
            .map_err(|e| GitBridgeError::GitOperation(format!("Time read failed: {e}")))?;
        let timestamp = format!("{}", gix::date::Time::new(time.seconds, time.offset));

        let parent_ids: Vec<String> = commit.parent_ids()
            .map(|id| id.to_hex().to_string())
            .collect();

        commits.push(GitCommitSummary {
            sha,
            short_sha,
            message,
            author_name,
            author_email,
            timestamp,
            parents: parent_ids,
            provider: "gix".into(),
        });

        // Walk first parent for linear history
        if let Some(parent_id) = commit.parent_ids().next() {
            let parent_id = parent_id.detach();
            if let Ok(obj) = repo.find_object(parent_id) {
                if let Ok(parent_commit) = obj.try_into_commit() {
                    current = Some(parent_commit);
                }
            }
        }
    }

    Ok(commits)
}

// ---------------------------------------------------------------------------
// CLI fallback
// ---------------------------------------------------------------------------

fn recent_commits_cli(
    project_root: &Path,
    limit: usize,
) -> Result<Vec<GitCommitSummary>, GitBridgeError> {
    let output = std::process::Command::new("git")
        .current_dir(project_root)
        .args([
            "log",
            &format!("-{limit}"),
            "--format=%H%n%h%n%s%n%an%n%ae%n%aI%n%P%n---COMMIT_END---",
        ])
        .output()
        .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;

    if !output.status.success() {
        return Err(GitBridgeError::GitOperation(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for block in stdout.split("---COMMIT_END---") {
        let lines: Vec<&str> = block.lines().collect();
        if lines.len() < 6 {
            continue;
        }

        let sha = lines[0].trim().to_string();
        let short_sha = lines[1].trim().to_string();
        let message = lines[2].trim().to_string();
        let author_name = lines[3].trim().to_string();
        let author_email = lines[4].trim().to_string();
        let timestamp = lines[5].trim().to_string();
        let parents: Vec<String> = lines.get(6)
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        if sha.is_empty() {
            continue;
        }

        commits.push(GitCommitSummary {
            sha,
            short_sha,
            message,
            author_name,
            author_email,
            timestamp,
            parents,
            provider: "git-cli-fallback".into(),
        });
    }

    Ok(commits)
}

// ---------------------------------------------------------------------------
// Response wrapper (v1.6.0)
// ---------------------------------------------------------------------------

/// Response wrapper for recent commits.
/// Carries provider even when commit list is empty.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecentCommitsResult {
    pub provider: String,
    pub commits: Vec<GitCommitSummary>,
    pub fallback_used: bool,
    pub latency_ms: u64,
}

// ---------------------------------------------------------------------------
// Public command
// ---------------------------------------------------------------------------

/// Read recent commits with bounded limit.
/// Tries gix native first, falls back to CLI.
pub fn recent_commits(
    project_root: &Path,
    limit: Option<usize>,
) -> Result<Vec<GitCommitSummary>, GitBridgeError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT).max(1);

    // Try native gix first
    match recent_commits_gix(project_root, limit) {
        Ok(commits) => Ok(commits),
        Err(_) => {
            // CLI fallback
            recent_commits_cli(project_root, limit)
        }
    }
}

/// Read recent commits with provider metadata.
/// Returns a wrapper that carries provider even when commit list is empty.
pub fn recent_commits_with_provider(
    project_root: &Path,
    limit: Option<usize>,
) -> Result<RecentCommitsResult, GitBridgeError> {
    let start = std::time::Instant::now();
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT).max(1);

    // Try native gix first
    match recent_commits_gix(project_root, limit) {
        Ok(commits) => Ok(RecentCommitsResult {
            provider: "gix".into(),
            commits,
            fallback_used: false,
            latency_ms: start.elapsed().as_millis() as u64,
        }),
        Err(_) => {
            // CLI fallback
            let commits = recent_commits_cli(project_root, limit)?;
            Ok(RecentCommitsResult {
                provider: "git-cli-fallback".into(),
                commits,
                fallback_used: true,
                latency_ms: start.elapsed().as_millis() as u64,
            })
        }
    }
}
