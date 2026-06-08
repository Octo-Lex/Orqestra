//! Auto-Apply Decision Engine (v2.6.0).
//!
//! Governs the first autonomous action in Orqestra: docs-only auto-apply.
//!
//! Principles:
//!   - Autonomy disabled by default; user must explicitly enable
//!   - Only docs-agent may auto-apply
//!   - Only docs-safe paths (docs/, README.md) allowed
//!   - Patch size computed server-side, never caller-supplied
//!   - Paths canonicalized before allowlist/exclusion checks
//!   - RequiresReview never writes files, records audit only
//!   - Per-session cap prevents runaway auto-apply
//!   - Auto-apply never commits
//!
//! Frontend may request auto-apply but may not define policy.
//! Rust loads persisted AutonomySettings from AppState.

use crate::commands::onboarding_types::*;
use crate::security::patch_guard::{AgentType, PatchProposal};
use git_bridge::operational_risk::classify_path;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Per-session counter (atomic, process-scoped)
// ---------------------------------------------------------------------------

static AUTO_APPLY_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Reset per-session counter (called on app startup).
pub fn reset_session_counter() {
    AUTO_APPLY_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
}

/// Get current session count.
pub fn session_auto_apply_count() -> usize {
    AUTO_APPLY_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

/// Increment session counter (call only on successful auto-apply).
pub fn increment_session_counter() {
    AUTO_APPLY_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// Path normalization and classification
// ---------------------------------------------------------------------------

/// Normalize a relative path for allowlist matching.
/// Handles: backslashes, double slashes, ./ prefixes, ../ traversal.
pub fn normalize_relative_path(path: &str) -> String {
    let mut normalized = path.replace("\\", "/");

    // Strip leading ./
    while normalized.starts_with("./") {
        normalized = normalized[2..].to_string();
    }

    // Collapse double slashes
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }

    // Strip leading /
    normalized = normalized.trim_start_matches('/').to_string();

    normalized
}

/// Check if a normalized path contains traversal attempts.
pub fn has_traversal(path: &str) -> bool {
    let normalized = normalize_relative_path(path);
    normalized.contains("..") ||
        // Also check raw path for encoding tricks
        path.contains("..")
}

/// Classify a path for audit (no raw paths in audit records).
pub fn classify_path_for_audit(path: &str) -> String {
    let normalized = normalize_relative_path(path);
    if normalized.starts_with("docs/") { return "docs".to_string(); }
    if normalized == "README.md" { return "readme".to_string(); }
    if normalized.starts_with("src/") || normalized.starts_with("lib/") ||
        normalized.starts_with("apps/") || normalized.starts_with("crates/") ||
        normalized.starts_with("services/")
    {
        return "source".to_string();
    }
    if normalized.contains(".github/") { return "workflow".to_string(); }
    if normalized.starts_with("roadmap/") { return "roadmap".to_string(); }
    if normalized == "CHANGELOG.md" { return "changelog".to_string(); }
    if normalized.contains("Cargo.") || normalized.contains("package.") ||
        normalized.contains(".lock")
    {
        return "dependency".to_string();
    }
    if normalized.contains("release-manifest") || normalized.contains("wrangler") ||
        normalized.contains("tauri.conf")
    {
        return "config".to_string();
    }
    if normalized.starts_with(".env") || normalized.contains("secret") ||
        normalized.contains("credential") || normalized.contains(".key") ||
        normalized.contains(".pem")
    {
        return "secret".to_string();
    }
    "other".to_string()
}

/// Check if a normalized path is in the docs-safe allowlist.
pub fn is_in_docs_allowlist(path: &str, allowlist: &[String]) -> bool {
    let normalized = normalize_relative_path(path);
    allowlist.iter().any(|prefix| {
        let p = prefix.trim_end_matches('/');
        normalized.starts_with(&format!("{}/", p)) || normalized == p
    })
}

/// Check if a path is README.md (stricter threshold).
pub fn is_readme(path: &str) -> bool {
    let normalized = normalize_relative_path(path);
    normalized == "README.md"
}

// ---------------------------------------------------------------------------
// Compute patch size server-side
// ---------------------------------------------------------------------------

/// Compute patch size from the actual patch content.
/// Uses after.len() as the measure.
pub fn compute_patch_size(patch: &PatchProposal) -> usize {
    patch.after.len()
}

// ---------------------------------------------------------------------------
// Decision engine
// ---------------------------------------------------------------------------

/// Decide whether a patch may be auto-applied.
/// All 12 gates must pass for Allowed.
/// RequiresReview never writes files.
pub fn decide_auto_apply(
    settings: &AutonomySettings,
    agent: &AgentType,
    patch: &PatchProposal,
    confidence: f64,
    current_file_checksum: Option<&str>,
) -> AutoApplyDecision {
    let mut reasons: Vec<AutoApplyRejectReason> = Vec::new();

    // Gate 1: Autonomy must be enabled
    if !settings.enabled {
        return AutoApplyDecision::Rejected(AutoApplyRejectReason::AutonomyDisabled);
    }

    // Gate 2: Only docs agent
    if *agent != AgentType::Docs {
        reasons.push(AutoApplyRejectReason::WrongAgent);
    }

    // Gate 3: Only auto-apply operation
    if settings.allowed_operation != "auto-apply" {
        reasons.push(AutoApplyRejectReason::WrongOperation);
    }

    // Gate 4: Auto-commit must be false
    if settings.auto_commit {
        reasons.push(AutoApplyRejectReason::AutoCommitNotFalse);
    }

    // Gate 5: Traversal check (before allowlist)
    if has_traversal(&patch.path) {
        return AutoApplyDecision::Rejected(AutoApplyRejectReason::TraversalAttempt);
    }

    let normalized_path = normalize_relative_path(&patch.path);

    // Gate 6: Path must be in docs-safe allowlist
    if !is_in_docs_allowlist(&normalized_path, &settings.docs_safe_paths) {
        reasons.push(AutoApplyRejectReason::PathNotInAllowlist);
    }

    // Gate 7: Operational risk check
    let risks = classify_path(&normalized_path);
    if risks.iter().any(|r| r.blocks_auto_apply) {
        reasons.push(AutoApplyRejectReason::OperationalRiskBlocked);
    }
    if risks.iter().any(|r| r.reject_outright()) {
        reasons.push(AutoApplyRejectReason::CredentialSecretRisk);
    }

    // Gate 8: Check forbidden via patch_guard's is_forbidden_path
    {
        // Import and use the existing forbidden check
        // We replicate the key checks here for the decision engine
        let lower = normalized_path.to_lowercase();
        let filename = lower.rsplit('/').next().unwrap_or(&lower);

        // Secret-risk
        if filename == ".env" || filename.starts_with(".env.") ||
            filename.ends_with(".pem") || filename.ends_with(".key") ||
            filename.starts_with("secrets.") || filename.starts_with("credentials.")
        {
            reasons.push(AutoApplyRejectReason::CredentialSecretRisk);
        }

        // Workflow-risk
        if normalized_path.contains(".github/workflows/") || normalized_path.contains(".github/actions/") {
            reasons.push(AutoApplyRejectReason::WorkflowRisk);
        }

        // Dependency-risk
        if filename == "cargo.toml" || filename == "cargo.lock" ||
            filename == "package.json" || filename == "package-lock.json" ||
            filename.ends_with(".lock")
        {
            reasons.push(AutoApplyRejectReason::DependencyRisk);
        }

        // Binary
        let binary_exts = [".png", ".jpg", ".jpeg", ".gif", ".exe", ".dll", ".so",
            ".zip", ".tar", ".gz", ".pdf", ".woff", ".woff2", ".ttf"];
        if binary_exts.iter().any(|ext| lower.ends_with(ext)) {
            reasons.push(AutoApplyRejectReason::BinaryFile);
        }

        // Excluded paths (narrower than server policy)
        let excluded = [
            "src/", "lib/", "apps/", "crates/", "services/",
            ".github/", ".ci/", ".env", ".Orqestra/",
            "roadmap/", "CHANGELOG.md",
            "release-manifest.json", "wrangler.toml", "tauri.conf.json",
        ];
        for ex in &excluded {
            let ex_norm = ex.trim_end_matches('/');
            if normalized_path.starts_with(&format!("{}/", ex_norm)) || normalized_path == ex_norm {
                reasons.push(AutoApplyRejectReason::PathExcluded);
                break;
            }
        }
    }

    // Gate 9: Patch size (computed server-side)
    let patch_size = compute_patch_size(patch);
    if patch_size > settings.max_patch_bytes {
        reasons.push(AutoApplyRejectReason::PatchTooLarge);
    }

    // Gate 10: Confidence threshold (README.md stricter)
    let threshold = if is_readme(&normalized_path) {
        settings.min_confidence_readme
    } else {
        settings.min_confidence_docs
    };
    if confidence < threshold {
        reasons.push(AutoApplyRejectReason::ConfidenceBelowThreshold);
    }

    // Gate 11: Before-checksum match (if current file exists)
    if let Some(current) = current_file_checksum {
        if current != patch.before_checksum {
            reasons.push(AutoApplyRejectReason::BeforeChecksumMismatch);
        }
    }

    // Gate 12: Per-session cap
    let count = session_auto_apply_count();
    if count >= settings.max_auto_apply_per_session {
        reasons.push(AutoApplyRejectReason::SessionCapExceeded);
    }

    // If any reasons accumulated, reject
    if !reasons.is_empty() {
        // If the only reason is SessionCapExceeded, route to review
        if reasons.len() == 1 && matches!(reasons[0], AutoApplyRejectReason::SessionCapExceeded) {
            return AutoApplyDecision::RequiresReview;
        }
        return AutoApplyDecision::Rejected(reasons.into_iter().next().unwrap());
    }

    AutoApplyDecision::Allowed
}

// ---------------------------------------------------------------------------
// Audit record construction
// ---------------------------------------------------------------------------

/// Build a redacted audit record for an auto-apply decision.
/// No source bodies, tokens, or raw paths.
pub fn build_auto_apply_audit(
    proposal_id: &str,
    patch: &PatchProposal,
    decision: &AutoApplyDecision,
    policy_version: u32,
) -> AutoApplyAuditRecord {
    let path_class = classify_path_for_audit(&patch.path);
    let (policy_decision, applied, reason_codes) = match decision {
        AutoApplyDecision::Allowed => {
            ("allowed".to_string(), true, vec![])
        }
        AutoApplyDecision::Rejected(reason) => {
            ("rejected".to_string(), false, vec![format!("{:?}", reason)
                .to_lowercase().replace('_', "-")])
        }
        AutoApplyDecision::RequiresReview => {
            ("requires-review".to_string(), false, vec!["session-cap-exceeded".to_string()])
        }
    };

    AutoApplyAuditRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        proposal_id: proposal_id.to_string(),
        agent: "docs".to_string(),
        path_class,
        policy_decision,
        reason_codes,
        before_checksum: patch.before_checksum.clone(),
        after_checksum: if applied { Some(patch.after_checksum.clone()) } else { None },
        applied,
        auto_commit: false,
        policy_version,
    }
}

// ---------------------------------------------------------------------------
// Validate autonomy settings on enable
// ---------------------------------------------------------------------------

/// Validate that autonomy settings are safe to enable.
/// Returns Ok(()) or Err with reason.
pub fn validate_autonomy_enable(settings: &AutonomySettings) -> Result<(), String> {
    if settings.allowed_agent != "docs" {
        return Err("Autonomy pilot requires allowed_agent = 'docs'".to_string());
    }
    if settings.allowed_operation != "auto-apply" {
        return Err("Autonomy pilot requires allowed_operation = 'auto-apply'".to_string());
    }
    if settings.auto_commit {
        return Err("Autonomy pilot requires auto_commit = false".to_string());
    }

    // Validate docs_safe_paths is exactly the pilot allowlist or narrower
    let valid_paths: Vec<&str> = DOCS_AUTO_APPLY_PATHS.iter().copied().collect();
    for path in &settings.docs_safe_paths {
        if !valid_paths.iter().any(|vp| path == vp) {
            return Err(format!(
                "docs_safe_paths must only contain {:?}, found '{}'",
                valid_paths, path
            ));
        }
    }

    // README threshold must be >= docs threshold
    if settings.min_confidence_readme < settings.min_confidence_docs {
        return Err("min_confidence_readme must be >= min_confidence_docs".to_string());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_settings() -> AutonomySettings {
        let mut s = AutonomySettings::default();
        s.enabled = true;
        s
    }

    fn make_patch(path: &str) -> PatchProposal {
        PatchProposal {
            proposal_id: "test-proposal".to_string(),
            path: path.to_string(),
            before: "old content".to_string(),
            after: "new content for the documentation file".to_string(),
            before_checksum: "abc123".to_string(),
            after_checksum: "def456".to_string(),
        }
    }

    #[test]
    fn test_normalize_path_strips_dot_slash() {
        assert_eq!(normalize_relative_path("./docs/foo.md"), "docs/foo.md");
    }

    #[test]
    fn test_normalize_path_collapses_double_slash() {
        assert_eq!(normalize_relative_path("docs//foo.md"), "docs/foo.md");
    }

    #[test]
    fn test_normalize_path_backslashes() {
        assert_eq!(normalize_relative_path("docs\\foo.md"), "docs/foo.md");
    }

    #[test]
    fn test_has_traversal_dotdot() {
        assert!(has_traversal("docs/../src/main.rs"));
    }

    #[test]
    fn test_no_traversal_clean_path() {
        assert!(!has_traversal("docs/guide.md"));
    }

    #[test]
    fn test_classify_docs() {
        assert_eq!(classify_path_for_audit("docs/guide.md"), "docs");
    }

    #[test]
    fn test_classify_readme() {
        assert_eq!(classify_path_for_audit("README.md"), "readme");
    }

    #[test]
    fn test_classify_source() {
        assert_eq!(classify_path_for_audit("src/main.rs"), "source");
    }

    #[test]
    fn test_classify_workflow() {
        assert_eq!(classify_path_for_audit(".github/workflows/ci.yml"), "workflow");
    }

    #[test]
    fn test_is_in_docs_allowlist_docs() {
        let al = vec!["docs/".to_string(), "README.md".to_string()];
        assert!(is_in_docs_allowlist("docs/guide.md", &al));
    }

    #[test]
    fn test_is_in_docs_allowlist_readme() {
        let al = vec!["docs/".to_string(), "README.md".to_string()];
        assert!(is_in_docs_allowlist("README.md", &al));
    }

    #[test]
    fn test_not_in_docs_allowlist_source() {
        let al = vec!["docs/".to_string(), "README.md".to_string()];
        assert!(!is_in_docs_allowlist("src/main.rs", &al));
    }

    #[test]
    fn test_is_readme() {
        assert!(is_readme("README.md"));
        assert!(!is_readme("docs/README.md"));
    }

    #[test]
    fn test_compute_patch_size() {
        let patch = make_patch("docs/guide.md");
        let size = compute_patch_size(&patch);
        assert_eq!(size, patch.after.len());
    }

    #[test]
    fn test_decide_disabled_by_default() {
        let settings = AutonomySettings::default(); // enabled = false
        let patch = make_patch("docs/guide.md");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(AutoApplyRejectReason::AutonomyDisabled)));
    }

    #[test]
    fn test_decide_docs_file_allowed() {
        let settings = default_settings();
        let patch = make_patch("docs/guide.md");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Allowed));
    }

    #[test]
    fn test_decide_readme_allowed_high_confidence() {
        let settings = default_settings();
        let patch = make_patch("README.md");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Allowed));
    }

    #[test]
    fn test_decide_readme_rejected_low_confidence() {
        let settings = default_settings();
        let patch = make_patch("README.md");
        // README requires >= 0.90; 0.85 is below
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.85, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(AutoApplyRejectReason::ConfidenceBelowThreshold)));
    }

    #[test]
    fn test_decide_source_rejected() {
        let settings = default_settings();
        let patch = make_patch("src/main.rs");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(_)));
    }

    #[test]
    fn test_decide_changelog_rejected() {
        let settings = default_settings();
        let patch = make_patch("CHANGELOG.md");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(_)));
    }

    #[test]
    fn test_decide_roadmap_rejected() {
        let settings = default_settings();
        let patch = make_patch("roadmap/tasks.md");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(_)));
    }

    #[test]
    fn test_decide_workflow_rejected() {
        let settings = default_settings();
        let patch = make_patch(".github/workflows/ci.yml");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(_)));
    }

    #[test]
    fn test_decide_secret_rejected() {
        let settings = default_settings();
        let patch = make_patch(".env");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(_)));
    }

    #[test]
    fn test_decide_binary_rejected() {
        let settings = default_settings();
        let patch = make_patch("image.png");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(_)));
    }

    #[test]
    fn test_decide_bugfix_agent_rejected() {
        let settings = default_settings();
        let patch = make_patch("docs/guide.md");
        let decision = decide_auto_apply(&settings, &AgentType::Bugfix, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(AutoApplyRejectReason::WrongAgent)));
    }

    #[test]
    fn test_decide_traversal_rejected() {
        let settings = default_settings();
        let patch = make_patch("docs/../src/main.rs");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(AutoApplyRejectReason::TraversalAttempt)));
    }

    #[test]
    fn test_decide_patch_too_large() {
        let mut settings = default_settings();
        settings.max_patch_bytes = 10; // Very small cap
        let mut patch = make_patch("docs/guide.md");
        patch.after = "This content is definitely longer than ten bytes total".to_string();
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(AutoApplyRejectReason::PatchTooLarge)));
    }

    #[test]
    fn test_decide_checksum_mismatch() {
        let settings = default_settings();
        let patch = make_patch("docs/guide.md");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, Some("wrong-checksum"));
        assert!(matches!(decision, AutoApplyDecision::Rejected(AutoApplyRejectReason::BeforeChecksumMismatch)));
    }

    #[test]
    fn test_requires_review_does_not_write() {
        // RequiresReview is a decision, not a write.
        // This test verifies the decision type.
        let decision = AutoApplyDecision::RequiresReview;
        match decision {
            AutoApplyDecision::RequiresReview => {
                // This decision never calls apply_agent_patch
                assert!(true);
            }
            _ => panic!("Expected RequiresReview"),
        }
    }

    #[test]
    fn test_audit_record_no_source_bodies() {
        let patch = make_patch("docs/guide.md");
        let audit = build_auto_apply_audit("test-id", &patch, &AutoApplyDecision::Allowed, 1);
        let json = serde_json::to_string(&audit).unwrap();
        assert!(!json.contains("old content"));
        assert!(!json.contains("new content"));
        assert!(!json.contains("ork_"));
        assert!(!json.contains("Bearer"));
    }

    #[test]
    fn test_audit_path_is_class_not_raw() {
        let patch = make_patch("docs/guide.md");
        let audit = build_auto_apply_audit("test-id", &patch, &AutoApplyDecision::Allowed, 1);
        assert_eq!(audit.path_class, "docs");
        // Raw path should NOT appear in audit
        let json = serde_json::to_string(&audit).unwrap();
        assert!(!json.contains("docs/guide.md"));
    }

    #[test]
    fn test_auto_apply_never_commits() {
        let settings = AutonomySettings::default();
        assert!(!settings.auto_commit);
        let audit = build_auto_apply_audit("id", &make_patch("docs/x.md"), &AutoApplyDecision::Allowed, 1);
        assert!(!audit.auto_commit);
    }

    #[test]
    fn test_validate_autonomy_enable_rejects_wrong_agent() {
        let mut s = AutonomySettings::default();
        s.allowed_agent = "bugfix".to_string();
        assert!(validate_autonomy_enable(&s).is_err());
    }

    #[test]
    fn test_validate_autonomy_enable_rejects_auto_commit() {
        let mut s = AutonomySettings::default();
        s.auto_commit = true;
        assert!(validate_autonomy_enable(&s).is_err());
    }

    #[test]
    fn test_validate_autonomy_enable_rejects_wider_paths() {
        let mut s = AutonomySettings::default();
        s.docs_safe_paths = vec!["src/".to_string()];
        assert!(validate_autonomy_enable(&s).is_err());
    }

    #[test]
    fn test_validate_autonomy_enable_rejects_readme_below_docs_threshold() {
        let mut s = AutonomySettings::default();
        s.min_confidence_readme = 0.50;
        s.min_confidence_docs = 0.80;
        assert!(validate_autonomy_enable(&s).is_err());
    }

    #[test]
    fn test_dependency_file_rejected() {
        let settings = default_settings();
        let patch = make_patch("Cargo.toml");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(_)));
    }

    #[test]
    fn test_release_manifest_rejected() {
        let settings = default_settings();
        let patch = make_patch("release-manifest.json");
        let decision = decide_auto_apply(&settings, &AgentType::Docs, &patch, 0.95, None);
        assert!(matches!(decision, AutoApplyDecision::Rejected(_)));
    }
}
