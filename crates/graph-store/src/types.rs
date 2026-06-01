use serde::{Deserialize, Serialize};

/// A single knowledge-graph triple, stored as one JSON file in
/// `.Orqestra/graph/triples/{uuid}.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Triple {
    pub uuid: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Semantic metadata extracted from a commit stub in
/// `.Orqestra/graph/commits/{hash}.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitStub {
    pub hash: String,
    #[serde(default)]
    pub parent_hashes: Vec<String>,
    pub author: CommitAuthor,
    pub timestamp: String,
    pub conventional_message: String,
    pub semantic: CommitSemantic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    #[serde(rename = "type")]
    pub author_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSemantic {
    pub status: String,
    pub intent_summary: String,
    #[serde(default)]
    pub affected_concepts: Vec<String>,
    #[serde(default)]
    pub affected_apis: Vec<String>,
    pub risk_assessment: RiskAssessment,
    pub confidence: f64,
    #[serde(default)]
    pub reasoning_trace_id: Option<String>,
    #[serde(default)]
    pub task_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub breaking_change: bool,
    pub migration_required: Option<String>,
    pub rollback_complexity: String,
}
