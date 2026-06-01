use crate::error::GitBridgeError;
use crate::semantic::{RiskAssessment, SemanticCommitObject, SemanticPayload};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Request sent to Python AI service /extract-intent
#[derive(Serialize)]
struct ExtractIntentRequest {
    diff: String,
    commit_message_draft: String,
    task_id: Option<String>,
    repo_context: Option<String>,
}

/// Response from Python AI service /extract-intent
#[derive(Deserialize)]
struct ExtractIntentResponse {
    intent_summary: String,
    affected_concepts: Vec<String>,
    affected_apis: Vec<String>,
    risk_assessment: AiRiskAssessment,
    confidence: f64,
    reasoning_trace: String,
}

#[derive(Deserialize)]
struct AiRiskAssessment {
    breaking_change: bool,
    migration_required: Option<String>,
    rollback_complexity: String,
}

pub struct BackfillRequest {
pub project_root: PathBuf,
    pub commit_hash: String,
    pub ai_service_url: String, // e.g. "http://localhost:8000"
    pub repo_context: Option<String>,
}

#[derive(Debug)]
pub struct BackfillResult {
    pub confidence: f64,
    pub intent_summary: String,
    pub reasoning_trace_id: String,
}

/// Backfill a pending semantic stub with AI-generated intent.
/// This is called asynchronously after semantic_commit() returns.
/// Uses blocking reqwest — run this in a thread, not on the main thread.
pub fn backfill_semantic_stub(
    request: BackfillRequest,
) -> Result<BackfillResult, GitBridgeError> {
    // 1. Read the existing stub
    let stub_path = request
        .project_root
        .join(".Orqestra")
        .join("graph")
        .join("commits")
        .join(format!("{}.json", request.commit_hash));

    let stub_content = std::fs::read_to_string(&stub_path)
        .map_err(|e| GitBridgeError::Io(stub_path.clone(), e))?;
    let stub: SemanticCommitObject = serde_json::from_str(&stub_content)?;

    // Extract task_ids from the pending payload
    let task_ids = match &stub.semantic {
        SemanticPayload::Pending { task_ids } => task_ids.clone(),
        SemanticPayload::Complete { .. } => {
            // Already backfilled — idempotent return
            return Err(GitBridgeError::GitOperation(
                "Stub already complete".to_string(),
            ));
        }
    };

    // 2. Get the diff via git show
    let diff = get_commit_diff(&request.project_root, &request.commit_hash)?;

    // 3. Call AI service
    let ai_request = ExtractIntentRequest {
        diff,
        commit_message_draft: stub.conventional_message.clone(),
        task_id: task_ids.first().cloned(),
        repo_context: request.repo_context,
    };

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(format!("{}/extract-intent", request.ai_service_url))
        .json(&ai_request)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .map_err(|e| GitBridgeError::GitOperation(format!("AI service unavailable: {e}")))?;

    if !response.status().is_success() {
        return Err(GitBridgeError::GitOperation(format!(
            "AI service error: {}",
            response.status()
        )));
    }

    let ai_response: ExtractIntentResponse = response
        .json()
        .map_err(|e| GitBridgeError::GitOperation(format!("AI response parse error: {e}")))?;

    // 4. Write reasoning trace
    let trace_id = Uuid::new_v4().to_string();
    write_reasoning_trace(
        &request.project_root,
        &trace_id,
        &ai_response.reasoning_trace,
    )?;

    // 5. Upgrade stub to Complete — atomic write
    let complete_stub = SemanticCommitObject {
        semantic: SemanticPayload::Complete {
            intent_summary: ai_response.intent_summary.clone(),
            affected_concepts: ai_response.affected_concepts,
            affected_apis: ai_response.affected_apis,
            risk_assessment: RiskAssessment {
                breaking_change: ai_response.risk_assessment.breaking_change,
                migration_required: ai_response.risk_assessment.migration_required,
                rollback_complexity: ai_response.risk_assessment.rollback_complexity,
            },
            confidence: ai_response.confidence,
            reasoning_trace_id: trace_id.clone(),
            task_ids,
        },
        ..stub
    };

    let tmp_path = stub_path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(&complete_stub)?;
    std::fs::write(&tmp_path, &content)
        .map_err(|e| GitBridgeError::Io(tmp_path.clone(), e))?;
    std::fs::rename(&tmp_path, &stub_path)
        .map_err(|e| GitBridgeError::Io(stub_path.clone(), e))?;

    Ok(BackfillResult {
        confidence: ai_response.confidence,
        intent_summary: ai_response.intent_summary,
        reasoning_trace_id: trace_id,
    })
}

fn get_commit_diff(
    project_root: &Path,
    hash: &str,
) -> Result<String, GitBridgeError> {
    let output = std::process::Command::new("git")
        .current_dir(project_root)
        .args(["show", "--unified=3", hash])
        .output()
        .map_err(|e| GitBridgeError::Io(project_root.to_owned(), e))?;

    if !output.status.success() {
        return Err(GitBridgeError::GitOperation(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn write_reasoning_trace(
    project_root: &Path,
    trace_id: &str,
    trace: &str,
) -> Result<(), GitBridgeError> {
    let dir = project_root
        .join(".Orqestra")
        .join("graph")
        .join("reasoning");
    std::fs::create_dir_all(&dir).map_err(|e| GitBridgeError::Io(dir.clone(), e))?;

    let path = dir.join(format!("{trace_id}.txt"));
    let tmp = dir.join(format!(".tmp-{trace_id}.txt"));

    std::fs::write(&tmp, trace).map_err(|e| GitBridgeError::Io(tmp.clone(), e))?;
    std::fs::rename(&tmp, &path).map_err(|e| GitBridgeError::Io(path.clone(), e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{CommitAuthor, SemanticCommitObject, SemanticPayload};
    use chrono::Utc;

    #[test]
    fn backfill_is_idempotent_on_complete_stub() {
        let dir = std::env::temp_dir().join("git-bridge-test-idempotent");
        std::fs::create_dir_all(&dir).unwrap();

        let commits_dir = dir.join(".Orqestra").join("graph").join("commits");
        std::fs::create_dir_all(&commits_dir).unwrap();

        let hash = "idempotent-test-hash";
        let complete_stub = SemanticCommitObject {
            hash: hash.to_string(),
            parent_hashes: vec![],
            author: CommitAuthor {
                name: "tester".into(),
                author_type: crate::semantic::AuthorType::Human,
            },
            timestamp: Utc::now(),
            conventional_message: "test: already complete".into(),
            semantic: SemanticPayload::Complete {
                intent_summary: "Already done".into(),
                affected_concepts: vec![],
                affected_apis: vec![],
                risk_assessment: RiskAssessment {
                    breaking_change: false,
                    migration_required: None,
                    rollback_complexity: "low".into(),
                },
                confidence: 0.95,
                reasoning_trace_id: "trace-000".into(),
                task_ids: vec![],
            },
        };

        let stub_path = commits_dir.join(format!("{hash}.json"));
        std::fs::write(&stub_path, serde_json::to_string_pretty(&complete_stub).unwrap()).unwrap();

        let result = backfill_semantic_stub(BackfillRequest {
            project_root: dir.clone(),
            commit_hash: hash.to_string(),
            ai_service_url: "http://localhost:99999".to_string(), // won't be called
            repo_context: None,
        });

        match result {
            Err(GitBridgeError::GitOperation(msg)) if msg.contains("already complete") => {}
            other => panic!("expected 'already complete' error, got {:?}", other),
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reasoning_trace_written_atomically() {
        let dir = std::env::temp_dir().join("git-bridge-test-trace-atomic");
        std::fs::create_dir_all(&dir).unwrap();

        let trace_id = "atomic-trace-001";
        write_reasoning_trace(&dir, trace_id, "Model thought process here").unwrap();

        let final_path = dir
            .join(".Orqestra")
            .join("graph")
            .join("reasoning")
            .join(format!("{trace_id}.txt"));
        let tmp_path = dir
            .join(".Orqestra")
            .join("graph")
            .join("reasoning")
            .join(format!(".tmp-{trace_id}.txt"));

        assert!(!tmp_path.exists(), "temp file should have been renamed");
        assert!(final_path.exists(), "final file should exist");
        assert_eq!(
            std::fs::read_to_string(&final_path).unwrap(),
            "Model thought process here"
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}
