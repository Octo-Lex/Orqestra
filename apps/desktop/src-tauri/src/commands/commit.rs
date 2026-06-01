//! Tauri commands wrapping git-bridge's semantic commit pipeline.
//!
//! These commands bridge the TypeScript UI to git-bridge's library functions.
//! git-bridge itself has zero Tauri dependencies.

use git_bridge::{
    backfill_semantic_stub, semantic_commit, AuthorType, BackfillRequest, CommitRequest,
};
use serde::Serialize;
use std::path::PathBuf;
use tauri::command;

#[derive(Debug, Serialize)]
pub struct SemanticCommitResult {
    pub hash: String,
    pub stub_path: String,
}

#[derive(Debug, Serialize)]
pub struct BackfillResult {
    pub confidence: f64,
    pub intent_summary: String,
    pub reasoning_trace_id: String,
}

/// Commit staged roadmap/ changes with a semantic stub.
///
/// Called from TypeScript as:
///   invoke('semantic_commit_cmd', { projectRoot, message, taskIds })
#[command]
pub fn semantic_commit_cmd(
    project_root: String,
    message: String,
    task_ids: Vec<String>,
) -> Result<SemanticCommitResult, String> {
    let request = CommitRequest {
        project_root: PathBuf::from(&project_root),
        message,
        author_name: "orqestra-user".to_string(),
        author_type: AuthorType::Human,
        task_ids,
        files_to_stage: vec![PathBuf::from("roadmap/")],
    };

    let result = semantic_commit(request).map_err(|e| e.to_string())?;

    Ok(SemanticCommitResult {
        hash: result.hash,
        stub_path: result.semantic_stub_path.to_string_lossy().to_string(),
    })
}

/// Backfill a pending semantic stub with AI-generated intent.
///
/// Called from TypeScript as:
///   invoke('backfill_cmd', { projectRoot, commitHash, aiServiceUrl })
///
/// This is a blocking call (reqwest::blocking). Tauri runs commands on a
/// threadpool, so this won't block the UI.
#[command]
pub fn backfill_cmd(
    project_root: String,
    commit_hash: String,
    ai_service_url: String,
) -> Result<BackfillResult, String> {
    let request = BackfillRequest {
        project_root: PathBuf::from(&project_root),
        commit_hash,
        ai_service_url,
        repo_context: None,
    };

    let result = backfill_semantic_stub(request).map_err(|e| e.to_string())?;

    Ok(BackfillResult {
        confidence: result.confidence,
        intent_summary: result.intent_summary,
        reasoning_trace_id: result.reasoning_trace_id,
    })
}
