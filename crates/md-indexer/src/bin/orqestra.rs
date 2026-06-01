//! Orqestra CLI — Phase 0 binary.
//!
//! Currently supports:
//!   orqestra deps --format=dot   Print task dependency graph as DOT

use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use md_indexer::graph::render_dot;
use md_indexer::index_roadmap;

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
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Format {
    Dot,
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
    }
}
