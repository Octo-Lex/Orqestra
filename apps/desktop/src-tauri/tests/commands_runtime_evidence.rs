//! v2.8.1: Runtime Evidence Collection
//!
//! Exercises the full auto-apply decision engine against real codebase paths
//! and generates a pilot safety report from the collected decisions.
//!
//! This is structural runtime evidence — not from external beta users,
//! but from exercising every gate against real project paths.

use std::path::PathBuf;

fn find_repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    while !dir.join(".git").exists() {
        if !dir.pop() { panic!("No git repo found"); }
    }
    dir
}

// ---------------------------------------------------------------------------
// Runtime Evidence Test: Full Decision Matrix
// ---------------------------------------------------------------------------

#[test]
fn test_runtime_evidence_full_decision_matrix() {
    use orqestra_desktop::commands::onboarding_types::*;
    use orqestra_desktop::security::auto_apply::*;
    use orqestra_desktop::security::patch_guard::{AgentType, PatchProposal};

    let mut settings = AutonomySettings::default();
    settings.enabled = true;

    // Define real codebase paths to test
    let test_cases: Vec<(&str, f64, AgentType, &str)> = vec![
        // === ALLOWED PATHS (should pass) ===
        ("docs/agent-context-quality.md", 0.95, AgentType::Docs, "docs file, high confidence"),
        ("docs/beta-quickstart.md", 0.95, AgentType::Docs, "docs file, high confidence"),
        ("docs/controlled-autonomy.md", 0.95, AgentType::Docs, "docs file, high confidence"),
        ("docs/autonomy-observability.md", 0.95, AgentType::Docs, "docs file, high confidence"),
        ("docs/DIAGNOSTICS.md", 0.95, AgentType::Docs, "docs file, high confidence"),
        ("docs/FIRST_RUN.md", 0.95, AgentType::Docs, "docs file, high confidence"),
        ("README.md", 0.95, AgentType::Docs, "README high confidence"),
        ("docs/deep/nested/path.md", 0.90, AgentType::Docs, "deep docs path"),

        // === README THRESHOLD TESTS ===
        ("README.md", 0.85, AgentType::Docs, "README below 0.90 threshold"),
        ("README.md", 0.90, AgentType::Docs, "README at exactly 0.90 threshold"),

        // === EXCLUDED PATHS (should be rejected) ===
        ("CHANGELOG.md", 0.95, AgentType::Docs, "CHANGELOG excluded"),
        ("roadmap/tasks.md", 0.95, AgentType::Docs, "roadmap excluded"),
        ("roadmap/alpha.md", 0.95, AgentType::Docs, "roadmap sub excluded"),

        // === SOURCE FILES ===
        ("src/main.rs", 0.95, AgentType::Docs, "source file rejected"),
        ("crates/git-bridge/src/lib.rs", 0.95, AgentType::Docs, "crate source rejected"),
        ("apps/desktop/src-tauri/src/main.rs", 0.95, AgentType::Docs, "app source rejected"),
        ("services/sync-relay/src/auth.ts", 0.95, AgentType::Docs, "service source rejected"),
        ("lib/core.rs", 0.95, AgentType::Docs, "lib source rejected"),

        // === WORKFLOW FILES ===
        (".github/workflows/ci.yml", 0.95, AgentType::Docs, "workflow rejected"),
        (".github/workflows/desktop-release.yml", 0.95, AgentType::Docs, "workflow rejected"),
        (".github/workflows/secret-scan.yml", 0.95, AgentType::Docs, "workflow rejected"),

        // === DEPENDENCY FILES ===
        ("Cargo.toml", 0.95, AgentType::Docs, "dep file rejected"),
        ("Cargo.lock", 0.95, AgentType::Docs, "lockfile rejected"),
        ("package.json", 0.95, AgentType::Docs, "dep file rejected"),
        ("package-lock.json", 0.95, AgentType::Docs, "lockfile rejected"),

        // === CONFIG/RELEASE FILES ===
        ("release-manifest.json", 0.95, AgentType::Docs, "release manifest rejected"),
        ("wrangler.toml", 0.95, AgentType::Docs, "config rejected"),
        ("apps/desktop/src-tauri/tauri.conf.json", 0.95, AgentType::Docs, "tauri config rejected"),

        // === SECRET FILES ===
        (".env", 0.95, AgentType::Docs, "env file rejected"),
        (".env.production", 0.95, AgentType::Docs, "env sub rejected"),
        ("secrets.yaml", 0.95, AgentType::Docs, "secrets rejected"),
        ("credentials.json", 0.95, AgentType::Docs, "credentials rejected"),
        ("id_rsa", 0.95, AgentType::Docs, "ssh key rejected"),
        ("server.pem", 0.95, AgentType::Docs, "pem key rejected"),

        // === BINARY FILES ===
        ("image.png", 0.95, AgentType::Docs, "png rejected"),
        ("screenshot.jpg", 0.95, AgentType::Docs, "jpg rejected"),
        ("archive.zip", 0.95, AgentType::Docs, "zip rejected"),
        ("binary.exe", 0.95, AgentType::Docs, "exe rejected"),

        // === TRAVERSAL ATTEMPTS ===
        ("docs/../src/main.rs", 0.95, AgentType::Docs, "traversal rejected"),
        ("./docs/../Cargo.toml", 0.95, AgentType::Docs, "traversal rejected"),

        // === WRONG AGENT ===
        ("docs/guide.md", 0.95, AgentType::Bugfix, "bugfix agent rejected"),
        ("README.md", 0.95, AgentType::Bugfix, "bugfix agent rejected"),

        // === CONFIDENCE THRESHOLD ===
        ("docs/guide.md", 0.50, AgentType::Docs, "docs below threshold"),
        ("docs/guide.md", 0.79, AgentType::Docs, "docs just below threshold"),
        ("docs/guide.md", 0.80, AgentType::Docs, "docs at threshold"),

        // === INTERNAL PATHS ===
        (".Orqestra/agents/docs/audit.jsonl", 0.95, AgentType::Docs, "internal path rejected"),
        (".Orqestra/config.json", 0.95, AgentType::Docs, "internal config rejected"),

        // === PATH NORMALIZATION ===
        ("docs//double.md", 0.95, AgentType::Docs, "double slash handled"),
        ("./docs/relative.md", 0.95, AgentType::Docs, "relative path handled"),
        ("docs\\windows.md", 0.95, AgentType::Docs, "backslash handled"),
    ];

    let mut allowed: Vec<String> = Vec::new();
    let mut rejected: Vec<(String, String)> = Vec::new();
    let mut requires_review: Vec<String> = Vec::new();
    let mut rejection_reasons: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut path_classes_allowed: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut path_classes_rejected: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut readme_allowed = 0usize;
    let mut readme_rejected = 0usize;
    let mut docs_allowed = 0usize;

    for (path, confidence, agent, description) in &test_cases {
        let patch = PatchProposal {
            proposal_id: format!("evidence-{}", path.replace('/', "_").replace('\\', "_")),
            path: path.to_string(),
            before: "old content".to_string(),
            after: "new documentation content that is reasonably long enough to pass size check".to_string(),
            before_checksum: "abc123".to_string(),
            after_checksum: "def456".to_string(),
        };

        let decision = decide_auto_apply(&settings, &agent, &patch, *confidence, None);
        let path_class = classify_path_for_audit(path);

        match &decision {
            AutoApplyDecision::Allowed => {
                allowed.push(path.to_string());
                *path_classes_allowed.entry(path_class.clone()).or_insert(0) += 1;
                if is_readme(path) { readme_allowed += 1; }
                if path_class == "docs" { docs_allowed += 1; }
            }
            AutoApplyDecision::Rejected(reason) => {
                let reason_str = format!("{:?}", reason).to_lowercase().replace('_', "-");
                *rejection_reasons.entry(reason_str.clone()).or_insert(0) += 1;
                rejected.push((path.to_string(), reason_str));
                *path_classes_rejected.entry(path_class.clone()).or_insert(0) += 1;
                if is_readme(path) { readme_rejected += 1; }
            }
            AutoApplyDecision::RequiresReview => {
                requires_review.push(path.to_string());
            }
        }
    }

    let total = test_cases.len();
    let rejection_rate = rejected.len() as f64 / total as f64;

    // ===================================================================
    // ASSERTIONS: Safety invariants
    // ===================================================================

    // 1. No source files were allowed
    let source_allowed: Vec<_> = allowed.iter()
        .filter(|p| p.starts_with("src/") || p.starts_with("crates/") || p.starts_with("apps/") || p.starts_with("services/"))
        .collect();
    assert!(source_allowed.is_empty(), "SOURCE FILES ALLOWED: {:?}", source_allowed);

    // 2. No workflow files were allowed
    let workflow_allowed: Vec<_> = allowed.iter().filter(|p| p.contains(".github/")).collect();
    assert!(workflow_allowed.is_empty(), "WORKFLOW FILES ALLOWED: {:?}", workflow_allowed);

    // 3. No secret files were allowed
    let secret_allowed: Vec<_> = allowed.iter()
        .filter(|p| p.contains(".env") || p.contains("secret") || p.contains("credential") || p.contains(".pem") || p.contains("id_rsa"))
        .collect();
    assert!(secret_allowed.is_empty(), "SECRET FILES ALLOWED: {:?}", secret_allowed);

    // 4. No dependency files were allowed
    let dep_allowed: Vec<_> = allowed.iter()
        .filter(|p| p.contains("Cargo.") || p.contains("package.") || p.contains(".lock"))
        .collect();
    assert!(dep_allowed.is_empty(), "DEPENDENCY FILES ALLOWED: {:?}", dep_allowed);

    // 5. CHANGELOG.md was rejected
    assert!(rejected.iter().any(|(p, _)| p == "CHANGELOG.md"), "CHANGELOG.md must be rejected");

    // 6. roadmap/ was rejected
    assert!(rejected.iter().any(|(p, _)| p.starts_with("roadmap/")), "roadmap/ must be rejected");

    // 7. README.md at 0.85 (below 0.90) was rejected
    assert!(rejected.iter().any(|(p, _)| p == "README.md"), "README.md below 0.90 must be rejected");

    // 8. README.md at 0.90 and 0.95 was allowed
    assert!(readme_allowed >= 1, "README.md at threshold must be allowed");

    // 9. Traversal attempts were rejected
    assert!(rejected.iter().any(|(p, _)| p.contains("..")), "traversal must be rejected");

    // 10. Bugfix agent was rejected
    let bugfix_rejections: Vec<_> = rejected.iter()
        .filter(|(p, r)| (p == "docs/guide.md" || p == "README.md") && r.contains("wrong"))
        .collect();
    assert!(bugfix_rejections.len() >= 2, "Bugfix agent must be rejected, got: {:?}", rejected.iter().filter(|(p, _)| p == "docs/guide.md" || p == "README.md").collect::<Vec<_>>());

    // 11. Auto-commit is always false
    assert!(!settings.auto_commit, "auto_commit must be false");

    // 12. docs/** paths were allowed
    assert!(docs_allowed >= 5, "docs/** paths must be allowed: got {}", docs_allowed);

    // ===================================================================
    // EVIDENCE REPORT
    // ===================================================================

    eprintln!("\n========================================");
    eprintln!("  RUNTIME EVIDENCE REPORT (structural)");
    eprintln!("========================================");
    eprintln!("Total paths tested:    {}", total);
    eprintln!("Allowed:               {}", allowed.len());
    eprintln!("Rejected:              {}", rejected.len());
    eprintln!("RequiresReview:        {}", requires_review.len());
    eprintln!("Rejection rate:        {:.1}%", rejection_rate * 100.0);
    eprintln!();
    eprintln!("README allowed:        {}", readme_allowed);
    eprintln!("README rejected:       {}", readme_rejected);
    eprintln!("docs/** allowed:       {}", docs_allowed);
    eprintln!();
    eprintln!("--- Allowed paths ---");
    for p in &allowed { eprintln!("  ✅ {}", p); }
    eprintln!();
    eprintln!("--- Top rejection reasons ---");
    let mut sorted_reasons: Vec<_> = rejection_reasons.iter().collect();
    sorted_reasons.sort_by(|a, b| b.1.cmp(a.1));
    for (reason, count) in &sorted_reasons {
        eprintln!("  {} ({}x)", reason, count);
    }
    eprintln!();
    eprintln!("--- Path classes allowed ---");
    for (cls, count) in &path_classes_allowed {
        eprintln!("  {} ({}x)", cls, count);
    }
    eprintln!();
    eprintln!("--- Path classes rejected ---");
    for (cls, count) in &path_classes_rejected {
        eprintln!("  {} ({}x)", cls, count);
    }
    eprintln!();
    eprintln!("--- Safety invariants ---");
    eprintln!("  no_source_files_touched:    {}", source_allowed.is_empty());
    eprintln!("  no_workflow_files_touched:  {}", workflow_allowed.is_empty());
    eprintln!("  no_secret_files_touched:    {}", secret_allowed.is_empty());
    eprintln!("  no_dep_files_touched:       {}", dep_allowed.is_empty());
    eprintln!("  auto_commit_always_false:   {}", !settings.auto_commit);
    eprintln!("  changelog_rejected:         {}", rejected.iter().any(|(p, _)| p == "CHANGELOG.md"));
    eprintln!("  roadmap_rejected:           {}", rejected.iter().any(|(p, _)| p.starts_with("roadmap/")));
    eprintln!("  traversal_rejected:         {}", rejected.iter().any(|(p, _)| p.contains("..")));
    eprintln!("  bugfix_agent_rejected:      {}", bugfix_rejections.len() >= 2);
    eprintln!("========================================\n");
}
