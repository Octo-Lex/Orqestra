//! # md-indexer
//!
//! Markdown roadmap indexer for Orqestra. Parses `roadmap/*.md` files with YAML
//! frontmatter into structured [`Task`] objects.

pub mod coordinator;
pub mod duration;
pub mod error;
pub mod evidence_schema;
pub mod graph;
pub mod indexer;
pub mod parser;
pub mod types;

// Public API — what external crates (e.g., the Tauri app) import.
pub use coordinator::{parse_coordinator, CoordinatorIndex, SprintEntry, EpicEntry, TeamMember};
pub use error::IndexerError;
pub use indexer::index_roadmap;
pub use types::{IndexResult, Task, TaskStatus};
