//! Dependency graph generation for task roadmaps.
//!
//! Produces DOT (Graphviz) output from the `dependencies` and `blocks` fields
//! of parsed tasks. Edges point from dependency → dependent (i.e., "must finish
//! before").

use crate::types::Task;
use std::collections::{BTreeMap, BTreeSet};
use std::io;

/// Render tasks as a DOT digraph to the given writer.
///
/// Nodes are task IDs. Edges represent dependency relationships:
/// if task B lists task A in `dependencies`, the edge is `A -> B`.
///
/// Phantom nodes are emitted for any dependency ID that does not correspond
/// to a parsed task (e.g., a file that failed to parse or is missing).
pub fn render_dot(tasks: &[Task], writer: &mut dyn io::Write) -> io::Result<()> {
    // Collect all known task IDs
    let known_ids: BTreeSet<&str> = tasks.iter().map(|t| t.frontmatter.id.as_str()).collect();

    // Collect all referenced IDs (dependencies + blocks)
    let mut referenced_ids: BTreeSet<&str> = BTreeSet::new();
    for task in tasks {
        for dep in &task.frontmatter.dependencies {
            referenced_ids.insert(dep.as_str());
        }
        for block in &task.frontmatter.blocks {
            referenced_ids.insert(block.as_str());
        }
    }

    // Build adjacency: for each task, edge from each dependency -> task
    let mut edges: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for task in tasks {
        for dep in &task.frontmatter.dependencies {
            edges
                .entry(dep.as_str())
                .or_default()
                .insert(task.frontmatter.id.as_str());
        }
    }

    // Collect phantom IDs (referenced but not parsed)
    let phantom_ids: BTreeSet<&&str> = referenced_ids
        .iter()
        .filter(|id| !known_ids.contains(**id))
        .collect();

    writeln!(writer, "digraph roadmap {{")?;
    writeln!(writer, "    rankdir=LR;")?;
    writeln!(writer, "    node [shape=box, style=filled, fontname=\"sans-serif\"];")?;
    writeln!(writer)?;

    // Emit nodes with status-based styling
    for task in tasks {
        let id = &task.frontmatter.id;
        let title = &task.frontmatter.title;
        let status = serde_json::to_string(&task.frontmatter.status)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();
        let fillcolor = match task.frontmatter.status {
            crate::types::TaskStatus::Done => "#10b981",
            crate::types::TaskStatus::InProgress => "#f59e0b",
            crate::types::TaskStatus::InReview => "#8b5cf6",
            crate::types::TaskStatus::Ready => "#3b82f6",
            crate::types::TaskStatus::Backlog => "#9ca3af",
            crate::types::TaskStatus::Cancelled => "#ef4444",
        };
        let fontcolor = if fillcolor == "#9ca3af" { "#1f2937" } else { "#ffffff" };
        writeln!(
            writer,
            "    \"{}\" [label=\"{}\\n({})\", fillcolor=\"{}\", fontcolor=\"{}\"];",
            id, title, status, fillcolor, fontcolor
        )?;
    }

    // Emit phantom nodes (referenced but not found)
    for id in &phantom_ids {
        writeln!(
            writer,
            "    \"{}\" [label=\"{}\\n(missing)\", fillcolor=\"#fecaca\", fontcolor=\"#991b1b\", style=\"filled,dashed\"];",
            id, id
        )?;
    }

    writeln!(writer)?;

    // Emit edges
    for (from, targets) in &edges {
        for to in targets {
            writeln!(writer, "    \"{}\" -> \"{}\";", from, to)?;
        }
    }

    writeln!(writer, "}}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_task_content;
    use std::path::Path;

    fn task_from(id: &str, deps: &[&str], blocks: &[&str]) -> Task {
        let yaml_deps: Vec<String> = deps.iter().map(|s| s.to_string()).collect();
        let yaml_blocks: Vec<String> = blocks.iter().map(|s| s.to_string()).collect();

        let frontmatter = format!(
            r#"---
pm-task: true
id: {}
title: "Task {}"
status: backlog
priority: Low
progress: 0
created: "2026-01-01T00:00:00Z"
updated: "2026-01-01T00:00:00Z"
dependencies: {:?}
blocks: {:?}
---
"#,
            id, id, yaml_deps, yaml_blocks
        );
        parse_task_content(&frontmatter, Path::new(&format!("{}.md", id))).unwrap()
    }

    #[test]
    fn dot_single_task_no_deps() {
        let tasks = vec![task_from("T-1", &[], &[])];
        let mut buf = Vec::new();
        render_dot(&tasks, &mut buf).unwrap();
        let dot = String::from_utf8(buf).unwrap();
        assert!(dot.contains("digraph roadmap"));
        assert!(dot.contains("\"T-1\""));
        assert!(!dot.contains("->"));
    }

    #[test]
    fn dot_chain_dependency() {
        let tasks = vec![
            task_from("T-1", &[], &["T-2"]),
            task_from("T-2", &["T-1"], &["T-3"]),
            task_from("T-3", &["T-2"], &[]),
        ];
        let mut buf = Vec::new();
        render_dot(&tasks, &mut buf).unwrap();
        let dot = String::from_utf8(buf).unwrap();
        assert!(dot.contains("\"T-1\" -> \"T-2\""));
        assert!(dot.contains("\"T-2\" -> \"T-3\""));
    }

    #[test]
    fn dot_phantom_node_for_missing_dependency() {
        let tasks = vec![task_from("T-1", &["T-MISSING"], &[])];
        let mut buf = Vec::new();
        render_dot(&tasks, &mut buf).unwrap();
        let dot = String::from_utf8(buf).unwrap();
        assert!(dot.contains("T-MISSING"));
        assert!(dot.contains("missing"));
        assert!(dot.contains("dashed"));
        assert!(dot.contains("\"T-MISSING\" -> \"T-1\""));
    }

    #[test]
    fn dot_diamond_dependency() {
        // T-1 -> T-2, T-1 -> T-3, T-2 + T-3 -> T-4
        let tasks = vec![
            task_from("T-1", &[], &["T-2", "T-3"]),
            task_from("T-2", &["T-1"], &["T-4"]),
            task_from("T-3", &["T-1"], &["T-4"]),
            task_from("T-4", &["T-2", "T-3"], &[]),
        ];
        let mut buf = Vec::new();
        render_dot(&tasks, &mut buf).unwrap();
        let dot = String::from_utf8(buf).unwrap();
        assert!(dot.contains("\"T-1\" -> \"T-2\""));
        assert!(dot.contains("\"T-1\" -> \"T-3\""));
        assert!(dot.contains("\"T-2\" -> \"T-4\""));
        assert!(dot.contains("\"T-3\" -> \"T-4\""));
    }

    #[test]
    fn dot_empty_tasks() {
        let tasks: Vec<Task> = vec![];
        let mut buf = Vec::new();
        render_dot(&tasks, &mut buf).unwrap();
        let dot = String::from_utf8(buf).unwrap();
        assert!(dot.contains("digraph roadmap"));
        assert!(dot.trim().ends_with('}'));
    }
}
