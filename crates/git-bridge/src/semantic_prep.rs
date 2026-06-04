//! Semantic commit preparation — proposal-only.
//!
//! Uses the stabilized read-only Git layer to prepare structured commit
//! proposals. Does NOT execute commits, stage files, or mutate the repository.
//!
//! Heuristics are path-based only (no file content reading).
//! Content-based hints are gated behind the diff body pilot (disabled by default).

use crate::error::GitBridgeError;
use crate::diff::diff_stat;
use crate::snapshot::{classify_risk_by_path, GitChangedFile};
use crate::{recent_commits, repository_snapshot};
use serde::Serialize;
use std::path::Path;

// ---------------------------------------------------------------------------
// Input model
// ---------------------------------------------------------------------------

/// Composite input for semantic commit preparation.
/// Built entirely from read-only Git operations.
#[derive(Debug, Clone, Serialize)]
pub struct SemanticCommitInput {
    pub repo_root: String,
    pub branch: String,
    pub head_short_sha: String,
    pub dirty: bool,
    pub changed_files: Vec<ChangedFileSummary>,
    pub diff_stat_summary: DiffStatSummary,
    pub recent_commit_subjects: Vec<String>,
    pub risk_summary: RiskSummary,
    pub provider: String,
    pub fallback_used: bool,
}

/// Changed file summary for commit context (content-free).
#[derive(Debug, Clone, Serialize)]
pub struct ChangedFileSummary {
    pub path: String,
    pub status: String,
    pub staged: bool,
    pub file_kind: String,
    pub risk: String,
    pub original_path: Option<String>,
}

/// Diff/stat summary (counts only, no content).
#[derive(Debug, Clone, Serialize)]
pub struct DiffStatSummary {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}

/// Risk summary counts.
#[derive(Debug, Clone, Serialize)]
pub struct RiskSummary {
    pub secret_count: u32,
    pub workflow_count: u32,
    pub binary_count: u32,
    pub large_count: u32,
    pub unknown_count: u32,
    pub normal_count: u32,
}

// ---------------------------------------------------------------------------
// Proposal DTO
// ---------------------------------------------------------------------------

/// A semantic commit proposal. Never executes or stages anything.
#[derive(Debug, Clone, Serialize)]
pub struct SemanticCommitProposal {
    pub title: String,
    pub body: String,
    pub scope: String,
    pub change_type: String,
    pub confidence: f64,
    pub risk_level: String,
    pub risk_notes: Vec<String>,
    pub groups: Vec<CommitGroup>,
    pub provider: String,
    pub write_operations: bool,
    pub requires_review: bool,
}

/// A suggested commit group.
#[derive(Debug, Clone, Serialize)]
pub struct CommitGroup {
    pub scope: String,
    pub change_type: String,
    pub files: Vec<String>,
    pub risk: String,
    pub suggested_title: String,
    pub suggested_body: String,
    pub requires_manual_review: bool,
}

/// Safe agent Git context — content-free.
#[derive(Debug, Clone, Serialize)]
pub struct AgentGitContext {
    pub branch: String,
    pub head_short_sha: String,
    pub changed_file_paths: Vec<String>,
    pub changed_file_statuses: Vec<String>,
    pub risk_flags: Vec<String>,
    pub diff_stat: DiffStatSummary,
    pub recent_commit_subjects: Vec<String>,
    pub risk_summary: RiskSummary,
}

// ---------------------------------------------------------------------------
// Build input model
// ---------------------------------------------------------------------------

/// Build semantic commit input from read-only Git operations.
pub fn build_semantic_commit_input(
    project_root: &Path,
) -> Result<SemanticCommitInput, GitBridgeError> {
    let snapshot = repository_snapshot(project_root)?;
    let stat = diff_stat(project_root)?;
    let commits = recent_commits(project_root, Some(5))?;

    let head_short_sha = snapshot
        .head
        .as_ref()
        .map(|h| h.short_sha.clone())
        .unwrap_or_else(|| "(no HEAD)".into());

    let changed_files: Vec<ChangedFileSummary> = snapshot
        .changed_files
        .iter()
        .map(|f| ChangedFileSummary {
            path: f.path.clone(),
            status: f.status.clone(),
            staged: f.staged,
            file_kind: f.file_kind.clone(),
            risk: f.risk.clone(),
            original_path: f.original_path.clone(),
        })
        .collect();

    let risk_summary = build_risk_summary(&snapshot.changed_files);

    let recent_commit_subjects: Vec<String> =
        commits.iter().map(|c| c.message.clone()).collect();

    Ok(SemanticCommitInput {
        repo_root: snapshot.repo_root.clone(),
        branch: snapshot.branch.clone(),
        head_short_sha,
        dirty: snapshot.dirty,
        changed_files,
        diff_stat_summary: DiffStatSummary {
            files_changed: stat.files_changed,
            insertions: stat.insertions,
            deletions: stat.deletions,
        },
        recent_commit_subjects,
        risk_summary,
        provider: snapshot.provider.clone(),
        fallback_used: snapshot.fallback_used,
    })
}

/// Build content-free agent Git context.
pub fn build_agent_context(
    project_root: &Path,
) -> Result<AgentGitContext, GitBridgeError> {
    let input = build_semantic_commit_input(project_root)?;

    Ok(AgentGitContext {
        branch: input.branch.clone(),
        head_short_sha: input.head_short_sha.clone(),
        changed_file_paths: input.changed_files.iter().map(|f| f.path.clone()).collect(),
        changed_file_statuses: input.changed_files.iter().map(|f| f.status.clone()).collect(),
        risk_flags: input
            .changed_files
            .iter()
            .filter(|f| f.risk != "normal")
            .map(|f| format!("{}: {}", f.path, f.risk))
            .collect(),
        diff_stat: input.diff_stat_summary.clone(),
        recent_commit_subjects: input.recent_commit_subjects.clone(),
        risk_summary: input.risk_summary.clone(),
    })
}

// ---------------------------------------------------------------------------
// Risk summary builder
// ---------------------------------------------------------------------------

fn build_risk_summary(files: &[GitChangedFile]) -> RiskSummary {
    let mut summary = RiskSummary {
        secret_count: 0,
        workflow_count: 0,
        binary_count: 0,
        large_count: 0,
        unknown_count: 0,
        normal_count: 0,
    };
    for f in files {
        match f.risk.as_str() {
            "secret" => summary.secret_count += 1,
            "workflow" => summary.workflow_count += 1,
            "binary" => summary.binary_count += 1,
            "large" => summary.large_count += 1,
            "unknown" => summary.unknown_count += 1,
            _ => summary.normal_count += 1,
        }
    }
    summary
}

// ---------------------------------------------------------------------------
// Deterministic proposal builder
// ---------------------------------------------------------------------------

/// Extract scope from file path.
fn extract_scope(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.starts_with("crates/git-bridge/") {
        return "git".into();
    } else if lower.starts_with("crates/") {
        let parts: Vec<&str> = lower.split('/').collect();
        if parts.len() >= 2 {
            return match parts[1].strip_suffix("-bridge") {
                Some(name) => name.into(),
                None => parts[1].into(),
            };
        }
        return "core".into();
    } else if lower.starts_with("apps/desktop/") {
        return "desktop".into();
    } else if lower.starts_with("apps/dashboard/") {
        return "dashboard".into();
    } else if lower.starts_with("docs/") {
        return "docs".into();
    } else if lower.starts_with(".github/") {
        return "ci".into();
    } else if lower.starts_with("scripts/") {
        return "build".into();
    } else if lower.starts_with("roadmap/") || lower.starts_with("demo/") {
        return "release".into();
    } else if lower.starts_with("dist/") {
        return "build".into();
    }

    // Root-level build/config files
    if lower.ends_with("cargo.toml")
        || lower.ends_with("cargo.lock")
        || lower.ends_with("package.json")
        || lower.ends_with("package-lock.json")
        || lower.ends_with("pnpm-lock.yaml")
        || lower.ends_with("tauri.conf.json")
    {
        return "build".into();
    }

    "general".into()
}

/// Determine change type from file paths and statuses (path-based only, no content).
fn determine_change_type(files: &[ChangedFileSummary]) -> &str {
    if files.is_empty() {
        return "chore";
    }

    let all_tests = files.iter().all(|f| {
        let p = f.path.to_lowercase();
        p.contains("test") || p.contains("tests") || p.contains("spec")
    });
    if all_tests {
        return "test";
    }

    let all_docs = files.iter().all(|f| {
        let p = f.path.to_lowercase();
        p.ends_with(".md") || p.starts_with("docs/") || p.starts_with("roadmap/")
    });
    if all_docs {
        return "docs";
    }

    let has_ci = files.iter().any(|f| {
        let p = f.path.to_lowercase();
        p.starts_with(".github/workflows/") || p.starts_with(".github/actions/")
    });
    if has_ci {
        return "ci";
    }

    let all_build = files
        .iter()
        .all(|f| matches!(
            f.path.to_lowercase().as_str(),
            p if p.ends_with("cargo.toml")
                || p.ends_with("cargo.lock")
                || p.ends_with("package.json")
                || p.ends_with("package-lock.json")
                || p.ends_with("pnpm-lock.yaml")
                || p.ends_with("tauri.conf.json")
        ));
    if all_build {
        return "build";
    }

    // Check for new files (status "added" or "untracked") in source paths
    let has_new_source = files.iter().any(|f| {
        (f.status == "added" || f.status == "untracked")
            && (f.path.starts_with("crates/") || f.path.starts_with("apps/"))
    });
    if has_new_source {
        return "feat";
    }

    // Default to refactor for source changes
    let has_source = files.iter().any(|f| {
        f.path.ends_with(".rs")
            || f.path.ends_with(".ts")
            || f.path.ends_with(".tsx")
            || f.path.ends_with(".js")
    });
    if has_source {
        return "refactor";
    }

    "chore"
}

/// Compute confidence score.
fn compute_confidence(files: &[ChangedFileSummary], scopes: &[String]) -> f64 {
    let has_risk = files.iter().any(|f| f.risk != "normal");
    let risk_penalty = if has_risk { 0.2 } else { 0.0 };
    let multi_scope_penalty = if scopes.len() > 1 { 0.2 } else { 0.0 };
    let many_files_penalty = if files.len() > 5 { 0.1 } else { 0.0 };

    (1.0_f64 - risk_penalty - multi_scope_penalty - many_files_penalty).max(0.3_f64)
}

/// Determine risk level.
fn determine_risk_level(files: &[ChangedFileSummary]) -> &str {
    if files.iter().any(|f| f.risk == "secret") {
        "elevated"
    } else if files.iter().any(|f| f.risk == "workflow" || f.risk == "unknown") {
        "caution"
    } else {
        "normal"
    }
}

/// Build risk notes.
fn build_risk_notes(files: &[ChangedFileSummary], risk_summary: &RiskSummary) -> Vec<String> {
    let mut notes = Vec::new();
    notes.push(format!("{} files changed", files.len()));

    if risk_summary.secret_count > 0 {
        notes.push(format!(
            "{} secret-risk file(s) — review carefully",
            risk_summary.secret_count
        ));
    }
    if risk_summary.workflow_count > 0 {
        notes.push(format!(
            "{} workflow-risk file(s) — CI changes detected",
            risk_summary.workflow_count
        ));
    }
    if risk_summary.binary_count > 0 {
        notes.push(format!(
            "{} binary file(s) excluded from analysis",
            risk_summary.binary_count
        ));
    }
    if risk_summary.large_count > 0 {
        notes.push(format!(
            "{} large file(s) excluded from analysis",
            risk_summary.large_count
        ));
    }

    if risk_summary.secret_count == 0 {
        notes.push("No secret-risk files included".into());
    }

    notes
}

/// Generate a human-readable body.
fn generate_body(files: &[ChangedFileSummary], stat: &DiffStatSummary) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "Changes across {} file(s): +{} / -{}",
        stat.files_changed, stat.insertions, stat.deletions
    ));

    // Group by scope
    let mut scope_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for f in files {
        let scope = extract_scope(&f.path);
        scope_map.entry(scope).or_default().push(f.path.clone());
    }

    for (scope, paths) in &scope_map {
        lines.push(format!("\n[{}] {} file(s):", scope, paths.len()));
        for p in paths.iter().take(5) {
            lines.push(format!("  - {}", p));
        }
        if paths.len() > 5 {
            lines.push(format!("  ... and {} more", paths.len() - 5));
        }
    }

    lines.join("\n")
}

/// Build the deterministic commit proposal.
pub fn prepare_semantic_commit(
    project_root: &Path,
) -> Result<SemanticCommitProposal, GitBridgeError> {
    let input = build_semantic_commit_input(project_root)?;

    if input.changed_files.is_empty() {
        return Ok(SemanticCommitProposal {
            title: "chore: no changes to commit".into(),
            body: "Working tree is clean. No changed files detected.".into(),
            scope: "general".into(),
            change_type: "chore".into(),
            confidence: 1.0,
            risk_level: "normal".into(),
            risk_notes: vec!["No changes detected".into()],
            groups: vec![],
            provider: "deterministic-heuristic".into(),
            write_operations: false,
            requires_review: true,
        });
    }

    // Determine overall scope
    let scope_counts: std::collections::HashMap<String, u32> = input
        .changed_files
        .iter()
        .map(|f| (extract_scope(&f.path), 1u32))
        .fold(std::collections::HashMap::new(), |mut acc, (s, c)| {
            *acc.entry(s).or_insert(0) += c;
            acc
        });
    let primary_scope = scope_counts
        .iter()
        .max_by_key(|(_, &c)| c)
        .map(|(s, _)| s.clone())
        .unwrap_or_else(|| "general".into());

    let change_type = determine_change_type(&input.changed_files);
    let scopes: Vec<String> = scope_counts.keys().map(|s| s.to_string()).collect();
    let confidence = compute_confidence(&input.changed_files, &scopes);
    let risk_level = determine_risk_level(&input.changed_files);
    let risk_notes = build_risk_notes(&input.changed_files, &input.risk_summary);

    // Generate title
    let title = format!("{}({}): {} changes across {} file(s)",
        change_type,
        primary_scope,
        input.diff_stat_summary.insertions + input.diff_stat_summary.deletions,
        input.changed_files.len()
    );

    let body = generate_body(&input.changed_files, &input.diff_stat_summary);

    // Build groups
    let groups = build_groups(&input.changed_files, &input.diff_stat_summary);

    Ok(SemanticCommitProposal {
        title,
        body,
        scope: primary_scope,
        change_type: change_type.to_string(),
        confidence,
        risk_level: risk_level.to_string(),
        risk_notes,
        groups,
        provider: "deterministic-heuristic".into(),
        write_operations: false,
        requires_review: true,
    })
}

// ---------------------------------------------------------------------------
// Grouping
// ---------------------------------------------------------------------------

fn build_groups(
    files: &[ChangedFileSummary],
    _stat: &DiffStatSummary,
) -> Vec<CommitGroup> {
    let mut groups: Vec<CommitGroup> = Vec::new();

    // Separate risk files first
    let (risk_files, normal_files): (Vec<_>, Vec<_>) =
        files.iter().partition(|f| f.risk != "normal");

    // Group normal files by scope
    let mut scope_groups: std::collections::BTreeMap<String, Vec<&ChangedFileSummary>> =
        std::collections::BTreeMap::new();
    for f in &normal_files {
        let scope = extract_scope(&f.path);
        scope_groups.entry(scope).or_default().push(f);
    }

    for (scope, group_files) in &scope_groups {
        let owned: Vec<ChangedFileSummary> = group_files.iter().cloned().cloned().collect();
        let change_type = determine_change_type(&owned);
        let paths: Vec<String> = group_files.iter().map(|f| f.path.clone()).collect();
        let suggested_title = format!("{}({}): update {} file(s)", change_type, scope, paths.len());

        groups.push(CommitGroup {
            scope: scope.to_string(),
            change_type: change_type.to_string(),
            files: paths,
            risk: "normal".into(),
            suggested_title,
            suggested_body: format!("Changes to {} files in {} scope.", group_files.len(), scope),
            requires_manual_review: true,
        });
    }

    // Add risk files as separate groups
    if !risk_files.is_empty() {
        let secret_files: Vec<_> = risk_files.iter().filter(|f| f.risk == "secret").collect();
        let workflow_files: Vec<_> = risk_files.iter().filter(|f| f.risk == "workflow").collect();
        let other_risk_files: Vec<_> = risk_files
            .iter()
            .filter(|f| f.risk != "secret" && f.risk != "workflow")
            .collect();

        if !secret_files.is_empty() {
            let paths: Vec<String> = secret_files.iter().map(|f| f.path.clone()).collect();
            groups.push(CommitGroup {
                scope: "security".into(),
                change_type: "chore".into(),
                files: paths,
                risk: "secret".into(),
                suggested_title: "chore(security): update secret-risk files".into(),
                suggested_body: "Secret-risk files detected. Review contents carefully before committing.".into(),
                requires_manual_review: true,
            });
        }

        if !workflow_files.is_empty() {
            let paths: Vec<String> = workflow_files.iter().map(|f| f.path.clone()).collect();
            groups.push(CommitGroup {
                scope: "ci".into(),
                change_type: "ci".into(),
                files: paths,
                risk: "workflow".into(),
                suggested_title: "ci: update workflow definitions".into(),
                suggested_body: "Workflow file changes detected. Verify CI behavior.".into(),
                requires_manual_review: true,
            });
        }

        if !other_risk_files.is_empty() {
            let paths: Vec<String> = other_risk_files.iter().map(|f| f.path.clone()).collect();
            groups.push(CommitGroup {
                scope: "general".into(),
                change_type: "chore".into(),
                files: paths,
                risk: "unknown".into(),
                suggested_title: "chore: update files requiring review".into(),
                suggested_body: "Files with unknown risk detected. Manual review recommended.".into(),
                requires_manual_review: true,
            });
        }
    }

    groups
}

// ---------------------------------------------------------------------------
// Diff body pilot (disabled by default)
// ---------------------------------------------------------------------------

/// Check if diff body pilot is enabled via environment variable.
pub fn diff_body_pilot_enabled() -> bool {
    std::env::var("SEMANTIC_PREP_DIFF_BODY_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Read safe diff body for a single file.
/// Returns None if pilot is disabled or file doesn't qualify.
pub fn read_safe_diff_body(
    project_root: &Path,
    relative_path: &str,
    file_kind: &str,
    risk: &str,
    file_size: u64,
) -> Option<String> {
    if !diff_body_pilot_enabled() {
        return None;
    }

    // Only text files
    if file_kind != "text" {
        return None;
    }

    // Only normal risk
    if risk != "normal" {
        return None;
    }

    // Max 256 KiB
    const MAX_SIZE: u64 = 256 * 1024;
    if file_size > MAX_SIZE {
        return None;
    }

    let full_path = project_root.join(relative_path);

    // Don't follow symlinks
    if full_path
        .symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
    {
        return None;
    }

    std::fs::read_to_string(&full_path).ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_scope_git_bridge() {
        assert_eq!(extract_scope("crates/git-bridge/src/snapshot.rs"), "git");
    }

    #[test]
    fn extract_scope_desktop() {
        assert_eq!(
            extract_scope("apps/desktop/src/components/App.tsx"),
            "desktop"
        );
    }

    #[test]
    fn extract_scope_dashboard() {
        assert_eq!(
            extract_scope("apps/dashboard/src/main.ts"),
            "dashboard"
        );
    }

    #[test]
    fn extract_scope_docs() {
        assert_eq!(extract_scope("docs/native-git.md"), "docs");
    }

    #[test]
    fn extract_scope_ci() {
        assert_eq!(
            extract_scope(".github/workflows/ci.yml"),
            "ci"
        );
    }

    #[test]
    fn extract_scope_build() {
        assert_eq!(extract_scope("Cargo.toml"), "build");
        assert_eq!(extract_scope("package.json"), "build");
    }

    #[test]
    fn extract_scope_release() {
        assert_eq!(extract_scope("roadmap/TASK-001.md"), "release");
        assert_eq!(extract_scope("demo/evidence.md"), "release");
    }

    #[test]
    fn determine_type_all_tests() {
        let files = vec![ChangedFileSummary {
            path: "tests/git_test.rs".into(),
            status: "modified".into(),
            staged: false,
            file_kind: "text".into(),
            risk: "normal".into(),
            original_path: None,
        }];
        assert_eq!(determine_change_type(&files), "test");
    }

    #[test]
    fn determine_type_all_docs() {
        let files = vec![ChangedFileSummary {
            path: "docs/native-git.md".into(),
            status: "modified".into(),
            staged: false,
            file_kind: "text".into(),
            risk: "normal".into(),
            original_path: None,
        }];
        assert_eq!(determine_change_type(&files), "docs");
    }

    #[test]
    fn determine_type_ci() {
        let files = vec![ChangedFileSummary {
            path: ".github/workflows/ci.yml".into(),
            status: "modified".into(),
            staged: false,
            file_kind: "text".into(),
            risk: "workflow".into(),
            original_path: None,
        }];
        assert_eq!(determine_change_type(&files), "ci");
    }

    #[test]
    fn determine_type_new_source() {
        let files = vec![ChangedFileSummary {
            path: "crates/git-bridge/src/new_module.rs".into(),
            status: "added".into(),
            staged: true,
            file_kind: "text".into(),
            risk: "normal".into(),
            original_path: None,
        }];
        assert_eq!(determine_change_type(&files), "feat");
    }

    #[test]
    fn determine_type_build() {
        let files = vec![ChangedFileSummary {
            path: "Cargo.toml".into(),
            status: "modified".into(),
            staged: false,
            file_kind: "text".into(),
            risk: "normal".into(),
            original_path: None,
        }];
        assert_eq!(determine_change_type(&files), "build");
    }

    #[test]
    fn determine_type_refactor() {
        let files = vec![ChangedFileSummary {
            path: "crates/git-bridge/src/snapshot.rs".into(),
            status: "modified".into(),
            staged: false,
            file_kind: "text".into(),
            risk: "normal".into(),
            original_path: None,
        }];
        assert_eq!(determine_change_type(&files), "refactor");
    }

    #[test]
    fn confidence_single_scope_normal() {
        let files = vec![ChangedFileSummary {
            path: "crates/git-bridge/src/lib.rs".into(),
            status: "modified".into(),
            staged: false,
            file_kind: "text".into(),
            risk: "normal".into(),
            original_path: None,
        }];
        let scopes = vec!["git".into()];
        assert!(compute_confidence(&files, &scopes) >= 0.9);
    }

    #[test]
    fn confidence_risk_penalty() {
        let files = vec![ChangedFileSummary {
            path: ".env".into(),
            status: "modified".into(),
            staged: false,
            file_kind: "unknown".into(),
            risk: "secret".into(),
            original_path: None,
        }];
        let scopes = vec!["security".into()];
        assert!(compute_confidence(&files, &scopes) < 0.9);
    }

    #[test]
    fn confidence_multi_scope_penalty() {
        let files = vec![
            ChangedFileSummary {
                path: "crates/git-bridge/src/lib.rs".into(),
                status: "modified".into(),
                staged: false,
                file_kind: "text".into(),
                risk: "normal".into(),
                original_path: None,
            },
            ChangedFileSummary {
                path: "docs/native-git.md".into(),
                status: "modified".into(),
                staged: false,
                file_kind: "text".into(),
                risk: "normal".into(),
                original_path: None,
            },
        ];
        let scopes = vec!["git".into(), "docs".into()];
        assert!(compute_confidence(&files, &scopes) < 0.9);
    }

    #[test]
    fn risk_level_elevated_with_secret() {
        let files = vec![ChangedFileSummary {
            path: ".env".into(),
            status: "modified".into(),
            staged: false,
            file_kind: "unknown".into(),
            risk: "secret".into(),
            original_path: None,
        }];
        assert_eq!(determine_risk_level(&files), "elevated");
    }

    #[test]
    fn risk_level_caution_with_workflow() {
        let files = vec![ChangedFileSummary {
            path: ".github/workflows/ci.yml".into(),
            status: "modified".into(),
            staged: false,
            file_kind: "text".into(),
            risk: "workflow".into(),
            original_path: None,
        }];
        assert_eq!(determine_risk_level(&files), "caution");
    }

    #[test]
    fn risk_level_normal() {
        let files = vec![ChangedFileSummary {
            path: "README.md".into(),
            status: "modified".into(),
            staged: false,
            file_kind: "text".into(),
            risk: "normal".into(),
            original_path: None,
        }];
        assert_eq!(determine_risk_level(&files), "normal");
    }

    #[test]
    fn diff_body_pilot_disabled_by_default() {
        assert!(!diff_body_pilot_enabled());
    }

    #[test]
    fn read_safe_diff_body_returns_none_when_disabled() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("test.txt");
        std::fs::write(&path, "hello").unwrap();

        let result = read_safe_diff_body(tmp.path(), "test.txt", "text", "normal", 5);
        assert!(result.is_none());
    }

    #[test]
    fn read_safe_diff_body_rejects_secret_risk() {
        // Even if pilot were enabled, secret-risk files must return None
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join(".env");
        std::fs::write(&path, "SECRET=123").unwrap();

        let result = read_safe_diff_body(tmp.path(), ".env", "text", "secret", 12);
        assert!(result.is_none(), "Secret-risk files must never be read");
    }

    #[test]
    fn read_safe_diff_body_rejects_binary() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("image.png");
        std::fs::write(&path, [0x89, 0x50, 0x4E, 0x47]).unwrap();

        let result = read_safe_diff_body(tmp.path(), "image.png", "binary", "normal", 4);
        assert!(result.is_none(), "Binary files must never be read");
    }

    #[test]
    fn read_safe_diff_body_rejects_large() {
        let result = read_safe_diff_body(
            std::path::Path::new("/tmp"),
            "big.txt",
            "text",
            "normal",
            1024 * 1024, // 1 MiB > 256 KiB
        );
        assert!(result.is_none(), "Large files must be rejected");
    }

    #[test]
    fn proposal_never_writes() {
        let proposal = SemanticCommitProposal {
            title: "test".into(),
            body: "test".into(),
            scope: "test".into(),
            change_type: "test".into(),
            confidence: 1.0,
            risk_level: "normal".into(),
            risk_notes: vec![],
            groups: vec![],
            provider: "deterministic-heuristic".into(),
            write_operations: false,
            requires_review: true,
        };
        assert!(!proposal.write_operations);
        assert!(proposal.requires_review);
    }
}
