// End-to-end test of the full semantic commit + backfill + ConfidenceGate pipeline.
// This proves the Tauri command layer works without needing the UI.
//
// Run: cargo test -p orqestra-desktop --test e2e_commit_pipeline
//      (or cargo run -p git-bridge --bin test_e2e -- C:\Next-Era\Orqestra)

use git_bridge::{
    backfill_semantic_stub, semantic_commit,
    AuthorType, BackfillRequest, CommitRequest,
};
use std::path::PathBuf;

fn main() {
    let project_root = PathBuf::from(
        std::env::args().nth(1).unwrap_or_else(|| ".".to_string()),
    );

    println!("=== Orqestra E2E: Commit + Backfill + ConfidenceGate ===\n");

    // Step 1: Make a real change to a roadmap file
    let task_file = project_root.join("roadmap/TASK-2026-050.md");
    let content = std::fs::read_to_string(&task_file).expect("read task file");

    // Toggle status
    let new_status = if content.contains("status: backlog") {
        content.replace("status: backlog", "status: in-progress")
    } else if content.contains("status: in-progress") {
        content.replace("status: in-progress", "status: in-review")
    } else {
        content.replace("status: in-review", "status: backlog")
    };
    std::fs::write(&task_file, &new_status).expect("write task file");
    println!("Modified roadmap/TASK-2026-050.md");

    // Step 2: Semantic commit
    let commit_result = semantic_commit(CommitRequest {
        project_root: project_root.clone(),
        message: "feat(ui): advance TASK-2026-050 status via pipeline".to_string(),
        author_name: "orqestra-e2e".to_string(),
        author_type: AuthorType::Human,
        task_ids: vec!["TASK-2026-050".to_string()],
        files_to_stage: vec![PathBuf::from("roadmap/TASK-2026-050.md")],
    }).expect("semantic_commit failed");

    println!("Commit: {}", commit_result.hash);
    println!("Stub:   {}\n", commit_result.semantic_stub_path.display());

    // Step 3: Backfill
    let backfill_result = backfill_semantic_stub(BackfillRequest {
        project_root: project_root.clone(),
        commit_hash: commit_result.hash.clone(),
        ai_service_url: "http://localhost:8000".to_string(),
        repo_context: Some("Orqestra — AI-native development environment".to_string()),
    }).expect("backfill failed");

    println!("--- Backfill Result ---");
    println!("Confidence:      {:.1}%", backfill_result.confidence * 100.0);
    println!("Intent:          {}", backfill_result.intent_summary);
    println!("Reasoning Trace: {}", backfill_result.reasoning_trace_id);

    // Step 4: ConfidenceGate logic (mirrors TypeScript)
    let confidence = backfill_result.confidence;
    let _breaking = false;
    let (action, label) = if confidence >= 0.90 {
        ("auto_commit", "Auto-Committed ✅")
    } else if confidence >= 0.70 {
        ("propose", "Proposed — Review Required ⚠️")
    } else if confidence >= 0.50 {
        ("flag", "Flagged — Human Review 🚩")
    } else {
        ("abort", "Aborted — Low Confidence 🛑")
    };
    println!("\n--- ConfidenceGate ---");
    println!("Action: {action}");
    println!("Label:  {label}");

    // Step 5: Verify files on disk
    let stub_path = project_root
        .join(".Orqestra/graph/commits")
        .join(format!("{}.json", commit_result.hash));
    let stub_content = std::fs::read_to_string(&stub_path).expect("read stub");
    assert!(stub_content.contains("\"status\": \"complete\""), "stub not complete!");
    println!("\nStub:   ✅ Complete");

    let trace_path = project_root
        .join(".Orqestra/graph/reasoning")
        .join(format!("{}.txt", backfill_result.reasoning_trace_id));
    assert!(trace_path.exists(), "reasoning trace missing!");
    println!("Trace:  ✅ Exists");

    println!("\n=== DONE ===");
    println!("Final stub:\n{}", stub_content);
}
