//! Orient stage — mechanical repo analysis (v2.15.0)
//!
//! Generates project-profile.json, repo-map.json, and conventions draft
//! without requiring AI. Uses filesystem scanning only.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::*;
use super::event_log;

// ---------------------------------------------------------------------------
// Project Profile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectProfile {
    pub schema_version: u32,
    pub generated_at: String,
    pub generated_by: String,
    pub project_name: String,
    pub languages: Vec<LanguageInfo>,
    pub frameworks: Vec<String>,
    pub build_system: String,
    pub test_commands: Vec<String>,
    pub package_managers: Vec<String>,
    pub total_files: usize,
    pub total_loc: usize,
    pub is_git_repo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub file_count: usize,
    pub percentage: f64,
    pub extensions: Vec<String>,
}

// ---------------------------------------------------------------------------
// Repo Map
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoMap {
    pub schema_version: u32,
    pub generated_at: String,
    pub root: String,
    pub directories: Vec<DirEntry>,
    pub file_type_summary: HashMap<String, usize>,
    pub total_dirs: usize,
    pub total_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub path: String,
    pub depth: u32,
    pub file_count: usize,
    pub subdirs: usize,
}

// ---------------------------------------------------------------------------
// Scanning
// ---------------------------------------------------------------------------

/// Extension → language mapping (mechanical, no AI)
fn language_for_extension(ext: &str) -> Option<&'static str> {
    match ext.to_lowercase().as_str() {
        "rs" => Some("Rust"),
        "ts" | "tsx" => Some("TypeScript"),
        "js" | "jsx" => Some("JavaScript"),
        "py" => Some("Python"),
        "go" => Some("Go"),
        "java" => Some("Java"),
        "c" | "h" => Some("C"),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some("C++"),
        "cs" => Some("C#"),
        "rb" => Some("Ruby"),
        "swift" => Some("Swift"),
        "kt" => Some("Kotlin"),
        "css" | "scss" | "sass" => Some("CSS"),
        "html" | "htm" => Some("HTML"),
        "json" => Some("JSON"),
        "yaml" | "yml" => Some("YAML"),
        "toml" => Some("TOML"),
        "md" => Some("Markdown"),
        "sql" => Some("SQL"),
        "sh" | "bash" => Some("Shell"),
        "dockerfile" => Some("Dockerfile"),
        _ => None,
    }
}

/// Files/directories to skip during scanning
fn should_skip(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | ".venv" | "__pycache__"
        | ".next" | "dist" | "build" | ".cache" | ".benchmarks"
        | ".pytest_cache" | ".ruff_cache" | "venv" | ".mypy_cache"
        | "coverage" | ".nyc_output" | ".turbo" | ".parcel-cache"
        | ".gradle" | ".idea" | ".vscode" | "*.egg-info"
    ) || name.ends_with(".egg-info")
}

/// Scan a repo and produce a ProjectProfile + RepoMap.
pub fn scan_repo(project_root: &Path) -> Result<(ProjectProfile, RepoMap), String> {
    let project_name = project_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let is_git = project_root.join(".git").exists();

    let mut lang_counts: HashMap<String, (usize, Vec<String>)> = HashMap::new();
    let mut file_type_counts: HashMap<String, usize> = HashMap::new();
    let mut dir_entries: Vec<DirEntry> = Vec::new();
    let mut total_files = 0usize;
    let mut total_dirs = 0usize;

    scan_dir(
        project_root,
        project_root,
        0,
        &mut lang_counts,
        &mut file_type_counts,
        &mut dir_entries,
        &mut total_files,
        &mut total_dirs,
    );

    // Build language info
    let total_for_pct = total_files.max(1);
    let mut languages: Vec<LanguageInfo> = lang_counts
        .iter()
        .map(|(name, (count, exts))| LanguageInfo {
            name: name.clone(),
            file_count: *count,
            percentage: (*count as f64 / total_for_pct as f64) * 100.0,
            extensions: exts.clone(),
        })
        .collect();
    languages.sort_by(|a, b| b.file_count.cmp(&a.file_count));

    // Detect frameworks and build system
    let (frameworks, build_system, test_commands, package_managers) = detect_tooling(project_root);

    let timestamp = chrono::Utc::now().to_rfc3339();

    let profile = ProjectProfile {
        schema_version: 1,
        generated_at: timestamp.clone(),
        generated_by: "orqestra-orient-mechanical".to_string(),
        project_name,
        languages,
        frameworks,
        build_system,
        test_commands,
        package_managers,
        total_files,
        total_loc: 0, // LOC counting is expensive — skip for mechanical scan
        is_git_repo: is_git,
    };

    let repo_map = RepoMap {
        schema_version: 1,
        generated_at: timestamp,
        root: project_root.to_string_lossy().to_string(),
        directories: dir_entries,
        file_type_summary: file_type_counts,
        total_dirs,
        total_files,
    };

    Ok((profile, repo_map))
}

fn scan_dir(
    root: &Path,
    current: &Path,
    depth: u32,
    lang_counts: &mut HashMap<String, (usize, Vec<String>)>,
    file_type_counts: &mut HashMap<String, usize>,
    dir_entries: &mut Vec<DirEntry>,
    total_files: &mut usize,
    total_dirs: &mut usize,
) {
    // Limit depth to avoid infinite recursion in symlink loops
    if depth > 5 {
        return;
    }

    let entries = match std::fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut file_count = 0usize;
    let mut subdir_count = 0usize;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        if should_skip(&name) {
            continue;
        }

        let path = entry.path();

        if path.is_dir() {
            subdir_count += 1;
            *total_dirs += 1;

            // Record this directory
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            dir_entries.push(DirEntry {
                path: rel_path,
                depth,
                file_count: 0, // Will be approximate — not recursing to count
                subdirs: 0,
            });

            scan_dir(
                root,
                &path,
                depth + 1,
                lang_counts,
                file_type_counts,
                dir_entries,
                total_files,
                total_dirs,
            );
        } else if path.is_file() {
            file_count += 1;
            *total_files += 1;

            // Track file type
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_else(|| "(no ext)".to_string());

            *file_type_counts.entry(ext.clone()).or_insert(0) += 1;

            // Track language
            if let Some(lang) = language_for_extension(&ext) {
                let entry = lang_counts.entry(lang.to_string()).or_insert((0, Vec::new()));
                entry.0 += 1;
                if !entry.1.contains(&ext) {
                    entry.1.push(ext);
                }
            }
        }
    }

    // Update the last dir entry's file count
    if let Some(last) = dir_entries.last_mut() {
        if last.depth == depth {
            last.file_count = file_count;
            last.subdirs = subdir_count;
        }
    }
}

fn detect_tooling(project_root: &Path) -> (Vec<String>, String, Vec<String>, Vec<String>) {
    let mut frameworks = Vec::new();
    let mut build_system = String::new();
    let mut test_commands = Vec::new();
    let mut package_managers = Vec::new();

    // Cargo.toml → Rust + Cargo
    if project_root.join("Cargo.toml").exists() {
        build_system = "Cargo".to_string();
        package_managers.push("Cargo".to_string());
        test_commands.push("cargo test --workspace".to_string());

        // Check for Tauri
        if has_file_matching(project_root, "Cargo.toml", "tauri") {
            frameworks.push("Tauri".to_string());
        }
    }

    // package.json → Node.js
    if project_root.join("package.json").exists() {
        package_managers.push("npm".to_string());
        test_commands.push("npm test".to_string());

        if has_file_matching(project_root, "package.json", "react") {
            frameworks.push("React".to_string());
        }
        if has_file_matching(project_root, "package.json", "vite") {
            frameworks.push("Vite".to_string());
        }
    }

    // pyproject.toml or setup.py → Python
    if project_root.join("pyproject.toml").exists() || project_root.join("setup.py").exists() {
        if build_system.is_empty() {
            build_system = "Python".to_string();
        }
        package_managers.push("pip".to_string());
        test_commands.push("pytest".to_string());
    }

    // pnpm-lock.yaml
    if project_root.join("pnpm-lock.yaml").exists() {
        package_managers.push("pnpm".to_string());
    }

    // yarn.lock
    if project_root.join("yarn.lock").exists() {
        package_managers.push("yarn".to_string());
    }

    // go.mod
    if project_root.join("go.mod").exists() {
        build_system = "Go Modules".to_string();
        test_commands.push("go test ./...".to_string());
    }

    if frameworks.is_empty() {
        frameworks.push("(none detected)".to_string());
    }

    if build_system.is_empty() {
        build_system = "(unknown)".to_string();
    }

    if test_commands.is_empty() {
        test_commands.push("(unknown)".to_string());
    }

    if package_managers.is_empty() {
        package_managers.push("(unknown)".to_string());
    }

    (frameworks, build_system, test_commands, package_managers)
}

fn has_file_matching(root: &Path, filename: &str, needle: &str) -> bool {
    let content = std::fs::read_to_string(root.join(filename)).unwrap_or_default();
    content.to_lowercase().contains(&needle.to_lowercase())
}

// ---------------------------------------------------------------------------
// Orient artifact generation
// ---------------------------------------------------------------------------

/// Run the Orient stage: scan repo, write artifacts, record events.
pub fn run_orient(project_root: &Path) -> Result<ProjectProfile, String> {
    // Ensure lifecycle dirs exist
    event_log::ensure_lifecycle_dirs(project_root).map_err(|e| e.to_string())?;

    // Scan
    let (profile, repo_map) = scan_repo(project_root)?;

    // Write project-profile.json
    let lifecycle = event_log::lifecycle_root(project_root);
    let profile_path = lifecycle.join("project/project-profile.json");
    let profile_json = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
    std::fs::write(&profile_path, &profile_json).map_err(|e| e.to_string())?;

    // Write repo-map.json
    let repo_map_path = lifecycle.join("project/repo-map.json");
    let repo_map_json = serde_json::to_string_pretty(&repo_map).map_err(|e| e.to_string())?;
    std::fs::write(&repo_map_path, &repo_map_json).map_err(|e| e.to_string())?;

    // Write conventions.md draft
    let conventions = generate_conventions_draft(&profile);
    let conv_path = lifecycle.join("project/conventions.md");
    std::fs::write(&conv_path, &conventions).map_err(|e| e.to_string())?;

    // Write risk-map.md draft
    let risk_map = generate_risk_map_draft(&profile);
    let risk_path = lifecycle.join("project/risk-map.md");
    std::fs::write(&risk_path, &risk_map).map_err(|e| e.to_string())?;

    // Record artifacts in event log
    let timestamp = chrono::Utc::now().to_rfc3339();
    for (art_type, path) in [
        (ArtifactType::ProjectProfile, "project/project-profile.json"),
        (ArtifactType::RepoMap, "project/repo-map.json"),
        (ArtifactType::Conventions, "project/conventions.md"),
        (ArtifactType::RiskMap, "project/risk-map.md"),
    ] {
        let event = LifecycleEvent::ArtifactCreated {
            artifact_type: art_type,
            path: path.to_string(),
            feature_id: None,
            timestamp: timestamp.clone(),
            actor: "repo-analyst".to_string(),
        };
        let _ = event_log::append_event(project_root, &event);
    }

    Ok(profile)
}

fn generate_conventions_draft(profile: &ProjectProfile) -> String {
    let mut md = String::new();
    md.push_str("# Conventions (Draft — Auto-generated)\n\n");
    md.push_str("This is a mechanical draft based on detected file types and tooling.\n");
    md.push_str("Review and refine.\n\n");

    md.push_str("## Languages\n\n");
    for lang in &profile.languages {
        md.push_str(&format!("- **{}**: {} files ({:.1}%)\n", lang.name, lang.file_count, lang.percentage));
    }

    md.push_str("\n## Build System\n\n");
    md.push_str(&format!("- {}\n", profile.build_system));

    if !profile.test_commands.is_empty() {
        md.push_str("\n## Test Commands\n\n");
        for cmd in &profile.test_commands {
            md.push_str(&format!("- `{}`\n", cmd));
        }
    }

    md.push_str("\n## Package Managers\n\n");
    for pm in &profile.package_managers {
        md.push_str(&format!("- {}\n", pm));
    }

    md.push_str("\n---\n*Generated by Orqestra Orient. This is a draft — review and edit.*\n");

    md
}

fn generate_risk_map_draft(profile: &ProjectProfile) -> String {
    let mut md = String::new();
    md.push_str("# Risk Map (Draft — Auto-generated)\n\n");
    md.push_str("Areas flagged as potentially risky based on project structure.\n");
    md.push_str("Review and refine.\n\n");

    // Flag areas that typically carry risk
    if profile.is_git_repo {
        md.push_str("## Git\n\n- ✅ Repository is a Git repo\n");
    } else {
        md.push_str("## Git\n\n- ⚠️ Not a Git repo — version control recommended\n");
    }

    // Language-specific risks
    let has_rust = profile.languages.iter().any(|l| l.name == "Rust");
    let has_ts = profile.languages.iter().any(|l| l.name == "TypeScript");
    let has_py = profile.languages.iter().any(|l| l.name == "Python");

    if has_rust {
        md.push_str("\n## Rust\n\n- `unsafe` blocks should be reviewed\n");
        md.push_str("- Test coverage (`cargo test --workspace`) should be maintained\n");
    }

    if has_ts {
        md.push_str("\n## TypeScript\n\n- Type safety: avoid `any` in production code\n");
        md.push_str("- Build: ensure `tsc` passes with strict mode\n");
    }

    if has_py {
        md.push_str("\n## Python\n\n- Type hints recommended (mypy)\n");
        md.push_str("- Linting: ruff/flake8 recommended\n");
    }

    md.push_str("\n---\n*Generated by Orqestra Orient. This is a draft — review and edit.*\n");

    md
}
