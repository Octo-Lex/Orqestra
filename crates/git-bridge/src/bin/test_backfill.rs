// Temporary test binary — delete after Phase 1 verification
use git_bridge::{
    backfill_semantic_stub, semantic_commit,
    AuthorType, BackfillRequest, CommitRequest,
};
use std::path::PathBuf;

fn main() {
    let project_root = PathBuf::from(
        std::env::args().nth(1).expect("pass project root as arg")
    );

    // Step 1: make a semantic commit
    let result = semantic_commit(CommitRequest {
        project_root: project_root.clone(),
        message: "test(phase-1): verify semantic commit pipeline".to_string(),
        author_name: "orqestra-test".to_string(),
        author_type: AuthorType::Human,
        task_ids: vec!["TASK-2026-045".to_string()],
        files_to_stage: vec![PathBuf::from("roadmap/TASK-2026-045.md")],
    }).expect("semantic_commit failed");

    println!("Committed: {}", result.hash);
    println!("Stub: {:?}", result.semantic_stub_path);

    // Step 2: backfill
    let backfill = backfill_semantic_stub(BackfillRequest {
        project_root: project_root.clone(),
        commit_hash: result.hash.clone(),
        ai_service_url: "http://localhost:8000".to_string(),
        repo_context: Some("Orqestra — AI-native dev environment".to_string()),
    }).expect("backfill failed");

    println!("Confidence: {}", backfill.confidence);
    println!("Intent: {}", backfill.intent_summary);
    println!("Trace ID: {}", backfill.reasoning_trace_id);

    // Step 3: verify the stub on disk is now Complete
    let stub_path = project_root
        .join(".Orqestra/graph/commits")
        .join(format!("{}.json", result.hash));
    let content = std::fs::read_to_string(&stub_path).unwrap();
    println!("\nFinal stub:\n{content}");
}
