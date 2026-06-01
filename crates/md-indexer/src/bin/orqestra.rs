//! Orqestra CLI — roadmap tooling.
//!
//! Supports:
//!   orqestra deps --format=dot       Print task dependency graph as DOT
//!   orqestra export --format=json    Export roadmap as JSON for dashboard

use clap::{Parser, Subcommand};
use serde::Serialize;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use md_indexer::coordinator::parse_coordinator;
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
    source: ExportSource,
    summary: ExportSummary,
    sprints: Vec<ExportSprint>,
    tasks: Vec<ExportTask>,
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

    // Get git info
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(project_root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

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

    RoadmapExport {
        generated_at: chrono::Utc::now().to_rfc3339(),
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
        sprints,
        tasks: export_tasks,
    }
}
