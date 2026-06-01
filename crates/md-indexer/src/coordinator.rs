//! Coordinator document parser for `roadmap/_index.md`.
//!
//! The coordinator is the only file where cross-file references are
//! authoritative (spec §3.6.1). It defines sprints, epics, and team members.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::IndexerError;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintEntry {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub tasks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicEntry {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub tasks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    #[serde(default)]
    pub role: Option<String>,
}

/// Parsed `_index.md` coordinator document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorIndex {
    #[serde(rename = "pm-index")]
    pub pm_index: bool,
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default)]
    pub sprints: Vec<SprintEntry>,
    #[serde(default)]
    pub epics: Vec<EpicEntry>,
    #[serde(default)]
    pub team: Vec<TeamMember>,
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse a `_index.md` file into a [`CoordinatorIndex`].
///
/// Returns `Err` if the file does not have `pm-index: true`.
pub fn parse_coordinator(raw: &str, source_path: &Path) -> Result<CoordinatorIndex, IndexerError> {
    let (yaml, _) = split_coordinator_frontmatter(raw, source_path)?;

    let coord: CoordinatorIndex = serde_yaml::from_str(yaml).map_err(|e| {
        IndexerError::InvalidFrontmatter(
            source_path.to_path_buf(),
            format!("{}", e),
        )
    })?;

    if !coord.pm_index {
        return Err(IndexerError::ParseError(
            source_path.to_path_buf(),
            "not a coordinator file (pm-index is not true)".into(),
        ));
    }

    Ok(coord)
}

fn split_coordinator_frontmatter<'a>(
    content: &'a str,
    path: &Path,
) -> Result<(&'a str, &'a str), IndexerError> {
    if !content.starts_with("---") {
        return Err(IndexerError::ParseError(
            path.to_path_buf(),
            "file does not start with --- frontmatter delimiter".into(),
        ));
    }

    let after_opening = &content[3..];
    let newline_after_opening = after_opening
        .find(|c: char| c != '\n' && c != '\r')
        .unwrap_or(0);
    let search_start = 3 + newline_after_opening;

    let closing = content[search_start..]
        .find("\n---")
        .map(|pos| search_start + pos)
        .or_else(|| {
            content[search_start..]
                .find("---")
                .map(|pos| search_start + pos)
                .filter(|&pos| pos > 3)
        });

    let closing = closing.ok_or_else(|| {
        IndexerError::ParseError(
            path.to_path_buf(),
            "no closing --- delimiter found".into(),
        )
    })?;

    let yaml_content = &content[3..closing].trim_start_matches('\n').trim_start_matches('\r');
    let body_start = content[closing..]
        .find('\n')
        .map(|offset| closing + offset + 1)
        .unwrap_or(content.len());

    Ok((yaml_content, &content[body_start..]))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_minimal_coordinator() {
        let yaml = r#"---
pm-index: true
version: 1
sprints:
  - id: "Sprint 1"
    title: "First"
    tasks: ["TASK-001"]
epics:
  - id: "epic-core"
    title: "Core"
    tasks: []
team:
  - id: "alice"
    role: "tech-lead"
---
Some body text
"#;
        let coord = parse_coordinator(yaml, Path::new("_index.md")).unwrap();
        assert!(coord.pm_index);
        assert_eq!(coord.version, Some(1));
        assert_eq!(coord.sprints.len(), 1);
        assert_eq!(coord.sprints[0].id, "Sprint 1");
        assert_eq!(coord.sprints[0].tasks, vec!["TASK-001"]);
        assert_eq!(coord.epics.len(), 1);
        assert_eq!(coord.team.len(), 1);
        assert_eq!(coord.team[0].id, "alice");
    }

    #[test]
    fn non_coordinator_returns_error() {
        let yaml = "---\npm-task: true\nid: TASK-001\n---\nBody";
        let result = parse_coordinator(yaml, Path::new("_index.md"));
        assert!(result.is_err());
    }
}
