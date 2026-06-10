//! Orqestra CLI — roadmap tooling.
//!
//! Supports:
//!   orqestra deps --format=dot       Print task dependency graph as DOT
//!   orqestra export --format=json    Export roadmap as JSON for dashboard

use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::Value;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use md_indexer::coordinator::parse_coordinator;
use md_indexer::evidence_schema::validate_evidence_dir;
use md_indexer::graph::render_dot;
use md_indexer::index_roadmap;
use md_indexer::types::TaskStatus;

#[derive(Parser)]
#[command(name = "orqestra")]
#[command(about = "Orqestra roadmap tooling")]
#[command(version)]
struct Cli {
    /// Path to the project root (defaults to current directory).
    #[arg(long, global = true, default_value = ".")]
    project_root: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show task dependency graph.
    Deps {
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Dot)]
        format: Format,
    },
    /// Export roadmap data for dashboard consumption.
    Export {
        /// Output format.
        #[arg(long, value_enum, default_value_t = ExportFormat::Json)]
        format: ExportFormat,

        /// Output file path. Prints to stdout if omitted.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Format {
    Dot,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ExportFormat {
    Json,
}

// ---------------------------------------------------------------------------
// Export types (spec §5.3)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct RoadmapExport {
    generated_at: String,
    release: ExportRelease,
    source: ExportSource,
    summary: ExportSummary,
    evidence: Option<ExportEvidence>,
    sprints: Vec<ExportSprint>,
    tasks: Vec<ExportTask>,
}

/// Evidence section embedded from docs/evidence/*.json files.
#[derive(Serialize)]
struct ExportEvidence {
    schema_version: u32,
    generated_from: EvidenceGeneratedFrom,
    release_history: Value,
    test_counts: Value,
    security_boundaries: Value,
    autonomy_policy: Value,
    runtime_evidence: Value,
}

#[derive(Serialize)]
struct EvidenceGeneratedFrom {
    source: String,
    commit: String,
    generated_at: String,
}

#[derive(Serialize)]
struct ExportRelease {
    version: String,
    source_commit: String,
    generated_at: String,
    generated_by: String,
}

#[derive(Serialize)]
struct ExportSource {
    repo: String,
    branch: String,
    commit: String,
}

#[derive(Serialize)]
struct ExportSummary {
    total_tasks: usize,
    done: usize,
    backlog: usize,
    in_progress: usize,
    blocked: usize,
    ready: usize,
}

#[derive(Serialize)]
struct ExportSprint {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    tasks: Vec<String>,
}

#[derive(Serialize)]
struct ExportTask {
    id: String,
    title: String,
    status: String,
    priority: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    epic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee: Option<String>,
    progress: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
    dependencies: Vec<String>,
    blocks: Vec<String>,
    labels: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    let roadmap_dir = cli.project_root.join("roadmap");

    let result = match index_roadmap(&roadmap_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    // Report parse warnings to stderr
    for (path, err) in &result.errors {
        eprintln!("warning: {}: {}", path.display(), err);
    }

    if result.tasks.is_empty() && !result.errors.is_empty() {
        eprintln!("error: no tasks parsed, all {} files failed", result.errors.len());
        process::exit(1);
    }

    match cli.command {
        Commands::Deps { format } => match format {
            Format::Dot => {
                let stdout = io::stdout();
                let mut lock = stdout.lock();
                if let Err(e) = render_dot(&result.tasks, &mut lock) {
                    eprintln!("error writing output: {}", e);
                    process::exit(1);
                }
                lock.flush().ok();
            }
        },
        Commands::Export { format, out } => match format {
            ExportFormat::Json => {
                let json = build_export(&result.tasks, &roadmap_dir, &cli.project_root);

                let output = serde_json::to_string_pretty(&json).unwrap();

                match out {
                    Some(path) => {
                        if let Some(parent) = path.parent() {
                            std::fs::create_dir_all(parent).ok();
                        }
                        if let Err(e) = std::fs::write(&path, &output) {
                            eprintln!("error writing {}: {}", path.display(), e);
                            process::exit(1);
                        }
                        eprintln!("exported {} tasks to {}", json.summary.total_tasks, path.display());
                    }
                    None => {
                        let stdout = io::stdout();
                        let mut lock = stdout.lock();
                        write!(lock, "{}", output).ok();
                        lock.flush().ok();
                    }
                }
            }
        },
    }
}

fn build_export(
    tasks: &[md_indexer::types::Task],
    roadmap_dir: &std::path::Path,
    project_root: &std::path::Path,
) -> RoadmapExport {
    use md_indexer::types::Priority;

    // Parse coordinator if present
    let index_path = roadmap_dir.join("_index.md");
    let coord = std::fs::read_to_string(&index_path)
        .ok()
        .and_then(|content| parse_coordinator(&content, &index_path).ok());

    let sprints = coord
        .as_ref()
        .map(|c| {
            c.sprints
                .iter()
                .map(|s| ExportSprint {
                    id: s.id.clone(),
                    title: s.title.clone(),
                    start_date: s.start_date.clone(),
                    end_date: s.end_date.clone(),
                    status: s.status.clone(),
                    tasks: s.tasks.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let done = tasks.iter().filter(|t| matches!(t.frontmatter.status, TaskStatus::Done)).count();
    let in_progress = tasks.iter().filter(|t| matches!(t.frontmatter.status, TaskStatus::InProgress)).count();
    let backlog = tasks.iter().filter(|t| matches!(t.frontmatter.status, TaskStatus::Backlog)).count();
    let ready = tasks.iter().filter(|t| matches!(t.frontmatter.status, TaskStatus::Ready)).count();

    let export_tasks: Vec<ExportTask> = tasks
        .iter()
        .map(|t| {
            let fm = &t.frontmatter;
            ExportTask {
                id: fm.id.clone(),
                title: fm.title.clone(),
                status: serde_json::to_string(&fm.status)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string(),
                priority: serde_json::to_string(&fm.priority)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string(),
                sprint: fm.sprint.clone(),
                epic: fm.epic.clone(),
                assignee: fm.assignee.clone(),
                progress: fm.progress,
                start_date: fm.start_date.clone(),
                due_date: fm.due_date.clone(),
                dependencies: fm.dependencies.clone(),
                blocks: fm.blocks.clone(),
                labels: fm.labels.clone(),
            }
        })
        .collect();

    // Get git info — full SHA for source_commit
    let full_commit = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let commit = if full_commit.len() > 12 {
        full_commit[..12].to_string()
    } else {
        full_commit.clone()
    };

    let branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let repo_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("orqestra")
        .to_string();

    let generated_at = chrono::Utc::now().to_rfc3339();

    // Load evidence files from docs/evidence/
    let evidence = load_evidence(project_root, &full_commit, &generated_at);

    RoadmapExport {
        generated_at: generated_at.clone(),
        release: ExportRelease {
            version: env!("CARGO_PKG_VERSION").to_string(),
            source_commit: full_commit.clone(),
            generated_at: generated_at.clone(),
            generated_by: "orqestra roadmap export".to_string(),
        },
        source: ExportSource {
            repo: repo_name,
            branch,
            commit,
        },
        summary: ExportSummary {
            total_tasks: tasks.len(),
            done,
            backlog,
            in_progress,
            blocked: 0, // No blocked status variant in enum
            ready,
        },
        evidence,
        sprints,
        tasks: export_tasks,
    }
}

/// Load evidence files from docs/evidence/ directory.
/// Returns None if evidence directory is missing — dashboard handles gracefully.
fn load_evidence(project_root: &std::path::Path, commit: &str, generated_at: &str) -> Option<ExportEvidence> {
    let evidence_dir = project_root.join("docs/evidence");

    if !evidence_dir.is_dir() {
        eprintln!("note: docs/evidence/ not found, skipping evidence section");
        return None;
    }

    // Validate evidence schema before embedding
    let validation = validate_evidence_dir(&evidence_dir);
    if !validation.valid {
        eprintln!("warning: evidence schema validation failed:");
        for err in &validation.errors {
            eprintln!("  - {}", err);
        }
        eprintln!("note: skipping evidence section due to validation errors");
        return None;
    }

    // Validation passed, read the files
    let release_history = read_json_file(&evidence_dir.join("release-history.json"));
    let test_counts = read_json_file(&evidence_dir.join("test-count-history.json"));
    let security_boundaries = read_json_file(&evidence_dir.join("security-boundaries.json"));
    let autonomy_policy = read_json_file(&evidence_dir.join("autonomy-policy.json"));
    let runtime_evidence = read_json_file(&evidence_dir.join("runtime-decision-matrix.json"));

    Some(ExportEvidence {
        schema_version: 1,
        generated_from: EvidenceGeneratedFrom {
            source: "static-export".to_string(),
            commit: commit.to_string(),
            generated_at: generated_at.to_string(),
        },
        release_history: release_history.unwrap(),
        test_counts: test_counts.unwrap(),
        security_boundaries: security_boundaries.unwrap(),
        autonomy_policy: autonomy_policy.unwrap(),
        runtime_evidence: runtime_evidence.unwrap(),
    })
}

fn read_json_file(path: &std::path::Path) -> Option<Value> {
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("warning: failed to parse {}: {}", path.display(), e);
                None
            }
        },
        Err(e) => {
            eprintln!("warning: failed to read {}: {}", path.display(), e);
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Tests for evidence export
// ---------------------------------------------------------------------------

#[cfg(test)]
mod evidence_export_tests {
    use super::*;
    use std::fs;

    fn setup_evidence_dir(dir: &std::path::Path) {
        fs::create_dir_all(dir).unwrap();

        let release_history = r#"{"schema_version":1,"releases":{"2.9.1":{"date":"2026-06-10","type":"security-patch","label":"Test"}}}"#;
        let test_counts = r#"{"schema_version":1,"history":[{"version":"2.9.1","rust":442,"worker":24,"dashboard":12,"total":478}]}"#;
        let security_boundaries = r#"{"schema_version":1,"boundaries":{"relay_auth":{"algorithm":"HMAC-SHA256"}}}"#;
        let autonomy_policy = r#"{"schema_version":1,"status":"docs-only pilot","max_session_cap":10,"default_cap":5,"auto_commit":false,"allowed_paths":["docs/**","README.md"]}"#;
        let runtime_evidence = r#"{"schema_version":1,"evidence_type":"structural-runtime-decision-matrix","external_beta_user_data":false,"path_matrix_evaluated":50}"#;

        fs::write(dir.join("release-history.json"), release_history).unwrap();
        fs::write(dir.join("test-count-history.json"), test_counts).unwrap();
        fs::write(dir.join("security-boundaries.json"), security_boundaries).unwrap();
        fs::write(dir.join("autonomy-policy.json"), autonomy_policy).unwrap();
        fs::write(dir.join("runtime-decision-matrix.json"), runtime_evidence).unwrap();
    }

    #[test]
    fn test_export_includes_evidence_section() {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path();
        let roadmap_dir = project_root.join("roadmap");
        let evidence_dir = project_root.join("docs/evidence");
        fs::create_dir_all(&roadmap_dir).unwrap();

        // Minimal task file
        fs::write(
            roadmap_dir.join("TASK-001.md"),
            "---\npm-task: true\nid: TASK-001\ntitle: Test\ntype: Task\nstatus: done\npriority: Medium\nprogress: 100\n---\n\nContext\n",
        ).unwrap();

        setup_evidence_dir(&evidence_dir);

        // Mock git info by creating a .git directory
        let result = md_indexer::index_roadmap(&roadmap_dir).unwrap();
        let export = build_export(&result.tasks, &roadmap_dir, project_root);

        assert!(export.evidence.is_some(), "export should include evidence section");
        let evidence = export.evidence.unwrap();
        assert_eq!(evidence.schema_version, 1);
    }

    #[test]
    fn test_export_without_evidence_dir_graceful() {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path();
        let roadmap_dir = project_root.join("roadmap");
        fs::create_dir_all(&roadmap_dir).unwrap();

        fs::write(
            roadmap_dir.join("TASK-001.md"),
            "---\npm-task: true\nid: TASK-001\ntitle: Test\ntype: Task\nstatus: done\npriority: Medium\nprogress: 100\n---\n\nContext\n",
        ).unwrap();

        // No docs/evidence/ directory
        let result = md_indexer::index_roadmap(&roadmap_dir).unwrap();
        let export = build_export(&result.tasks, &roadmap_dir, project_root);

        assert!(export.evidence.is_none(), "export should handle missing evidence gracefully");
    }

    #[test]
    fn test_evidence_schema_has_required_sections() {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path();
        let roadmap_dir = project_root.join("roadmap");
        let evidence_dir = project_root.join("docs/evidence");
        fs::create_dir_all(&roadmap_dir).unwrap();

        fs::write(
            roadmap_dir.join("TASK-001.md"),
            "---\npm-task: true\nid: TASK-001\ntitle: Test\ntype: Task\nstatus: done\npriority: Medium\nprogress: 100\n---\n\nContext\n",
        ).unwrap();

        setup_evidence_dir(&evidence_dir);

        let result = md_indexer::index_roadmap(&roadmap_dir).unwrap();
        let export = build_export(&result.tasks, &roadmap_dir, project_root);

        let evidence = export.evidence.expect("evidence should be present");
        assert!(evidence.release_history.is_object());
        assert!(evidence.test_counts.is_object());
        assert!(evidence.security_boundaries.is_object());
        assert!(evidence.autonomy_policy.is_object());
        assert!(evidence.runtime_evidence.is_object());
    }

    #[test]
    fn test_runtime_evidence_marked_structural() {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path();
        let roadmap_dir = project_root.join("roadmap");
        let evidence_dir = project_root.join("docs/evidence");
        fs::create_dir_all(&roadmap_dir).unwrap();

        fs::write(
            roadmap_dir.join("TASK-001.md"),
            "---\npm-task: true\nid: TASK-001\ntitle: Test\ntype: Task\nstatus: done\npriority: Medium\nprogress: 100\n---\n\nContext\n",
        ).unwrap();

        setup_evidence_dir(&evidence_dir);

        let result = md_indexer::index_roadmap(&roadmap_dir).unwrap();
        let export = build_export(&result.tasks, &roadmap_dir, project_root);

        let evidence = export.evidence.expect("evidence should be present");
        let rt = &evidence.runtime_evidence;
        let evidence_type = rt.get("evidence_type").unwrap().as_str().unwrap();
        assert_eq!(evidence_type, "structural-runtime-decision-matrix");
        let external = rt.get("external_beta_user_data").unwrap().as_bool().unwrap();
        assert!(!external, "external_beta_user_data must be false");
    }

    #[test]
    fn test_autonomy_policy_cap_values() {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path();
        let roadmap_dir = project_root.join("roadmap");
        let evidence_dir = project_root.join("docs/evidence");
        fs::create_dir_all(&roadmap_dir).unwrap();

        fs::write(
            roadmap_dir.join("TASK-001.md"),
            "---\npm-task: true\nid: TASK-001\ntitle: Test\ntype: Task\nstatus: done\npriority: Medium\nprogress: 100\n---\n\nContext\n",
        ).unwrap();

        setup_evidence_dir(&evidence_dir);

        let result = md_indexer::index_roadmap(&roadmap_dir).unwrap();
        let export = build_export(&result.tasks, &roadmap_dir, project_root);

        let evidence = export.evidence.expect("evidence should be present");
        let policy = &evidence.autonomy_policy;
        let max_cap = policy.get("max_session_cap").unwrap().as_u64().unwrap();
        assert_eq!(max_cap, 10, "max_session_cap in evidence must be 10");
        let auto_commit = policy.get("auto_commit").unwrap().as_bool().unwrap();
        assert!(!auto_commit, "auto_commit in evidence must be false");
    }
}
