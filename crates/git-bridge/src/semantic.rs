use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCommitObject {
    pub hash: String,
    pub parent_hashes: Vec<String>,
    pub author: CommitAuthor,
    pub timestamp: DateTime<Utc>,
    pub conventional_message: String,
    pub semantic: SemanticPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub name: String,
    #[serde(rename = "type")]
    pub author_type: AuthorType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorType {
    Human,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum SemanticPayload {
    #[serde(rename = "pending")]
    Pending { task_ids: Vec<String> },
    #[serde(rename = "complete")]
    Complete {
        intent_summary: String,
        affected_concepts: Vec<String>,
        affected_apis: Vec<String>,
        risk_assessment: RiskAssessment,
        confidence: f64,
        reasoning_trace_id: String,
        task_ids: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub breaking_change: bool,
    pub migration_required: Option<String>,
    pub rollback_complexity: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_stub_serializes_correctly() {
        let stub = SemanticCommitObject {
            hash: "abc123".into(),
            parent_hashes: vec![],
            author: CommitAuthor {
                name: "agent-architect".into(),
                author_type: AuthorType::Agent,
            },
            timestamp: Utc::now(),
            conventional_message: "feat(auth): add JWT".into(),
            semantic: SemanticPayload::Pending {
                task_ids: vec!["TASK-2026-042".into()],
            },
        };

        let json = serde_json::to_value(&stub).unwrap();
        assert_eq!(json["hash"], "abc123");
        assert_eq!(json["semantic"]["status"], "pending");
        assert_eq!(json["semantic"]["task_ids"][0], "TASK-2026-042");
        assert_eq!(json["author"]["type"], "agent");
    }

    #[test]
    fn complete_payload_serializes_correctly() {
        let complete = SemanticCommitObject {
            hash: "def456".into(),
            parent_hashes: vec!["abc123".into()],
            author: CommitAuthor {
                name: "developer".into(),
                author_type: AuthorType::Human,
            },
            timestamp: Utc::now(),
            conventional_message: "fix(cache): clear stale tokens".into(),
            semantic: SemanticPayload::Complete {
                intent_summary: "Clear stale session tokens from cache".into(),
                affected_concepts: vec!["cache".into(), "session".into()],
                affected_apis: vec![],
                risk_assessment: RiskAssessment {
                    breaking_change: false,
                    migration_required: None,
                    rollback_complexity: "low".into(),
                },
                confidence: 0.92,
                reasoning_trace_id: "trace-001".into(),
                task_ids: vec!["TASK-2026-040".into()],
            },
        };

        let json = serde_json::to_value(&complete).unwrap();
        assert_eq!(json["semantic"]["status"], "complete");
        assert_eq!(json["semantic"]["confidence"], 0.92);
        assert_eq!(json["semantic"]["affected_concepts"][0], "cache");
        assert_eq!(json["semantic"]["risk_assessment"]["breaking_change"], false);
    }

    #[test]
    fn stub_written_atomically() {
        use std::io::Write;

        let dir = std::env::temp_dir().join("git-bridge-test-atomic");
        std::fs::create_dir_all(&dir).unwrap();

        let stub = SemanticCommitObject {
            hash: "atomic-test".into(),
            parent_hashes: vec![],
            author: CommitAuthor {
                name: "tester".into(),
                author_type: AuthorType::Human,
            },
            timestamp: Utc::now(),
            conventional_message: "test: atomic write".into(),
            semantic: SemanticPayload::Pending { task_ids: vec![] },
        };

        let final_path = dir.join("atomic-test.json");
        let tmp_path = dir.join(".tmp-atomic-test.json");

        let content = serde_json::to_string_pretty(&stub).unwrap();
        std::fs::write(&tmp_path, &content).unwrap();
        std::fs::rename(&tmp_path, &final_path).unwrap();

        // tmp file must not exist
        assert!(!tmp_path.exists(), "temp file should have been renamed");
        // final file must exist and be valid JSON
        let read_back: SemanticCommitObject =
            serde_json::from_str(&std::fs::read_to_string(&final_path).unwrap()).unwrap();
        assert_eq!(read_back.hash, "atomic-test");

        std::fs::remove_dir_all(&dir).ok();
    }
}
