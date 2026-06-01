use graph_store::{index_commits, TripleStore};
use std::path::PathBuf;
use tauri::State;
use tokio::sync::Mutex;

pub struct GraphState {
    pub store: Mutex<Option<TripleStore>>,
}

#[derive(serde::Serialize)]
pub struct TripleResult {
    pub uuid: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub commit: Option<String>,
    pub timestamp: Option<String>,
}

impl From<graph_store::Triple> for TripleResult {
    fn from(t: graph_store::Triple) -> Self {
        TripleResult {
            uuid: t.uuid,
            subject: t.subject,
            predicate: t.predicate,
            object: t.object,
            commit: t.commit,
            timestamp: t.timestamp,
        }
    }
}

#[tauri::command]
pub async fn index_graph_cmd(
    project_root: String,
    state: State<'_, GraphState>,
) -> Result<serde_json::Value, String> {
    let root = PathBuf::from(&project_root);
    let triples_dir = root.join(".Orqestra/graph/triples");
    let commits_dir = root.join(".Orqestra/graph/commits");

    let store = TripleStore::load(triples_dir).map_err(|e| e.to_string())?;

    let count = if commits_dir.exists() {
        index_commits(&store, &commits_dir).map_err(|e| e.to_string())?
    } else {
        0
    };

    let total = store.len();
    *state.store.lock().await = Some(store);

    Ok(serde_json::json!({
        "indexed": count,
        "total_triples": total,
    }))
}

#[tauri::command]
pub async fn query_graph_cmd(
    project_root: String,
    subject: Option<String>,
    predicate: Option<String>,
    object: Option<String>,
    state: State<'_, GraphState>,
) -> Result<Vec<TripleResult>, String> {
    let mut guard = state.store.lock().await;
    if guard.is_none() {
        // Auto-index if not yet loaded
        let root = PathBuf::from(&project_root);
        let triples_dir = root.join(".Orqestra/graph/triples");
        let store = TripleStore::load(triples_dir).map_err(|e| e.to_string())?;
        *guard = Some(store);
    }

    if let Some(ref store) = *guard {
        let results = store.query(
            subject.as_deref(),
            predicate.as_deref(),
            object.as_deref(),
        );
        Ok(results.into_iter().map(TripleResult::from).collect())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn query_history_cmd(
    project_root: String,
    question: String,
) -> Result<serde_json::Value, String> {
    // Forward to the Python AI service
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:8000/query-history")
        .json(&serde_json::json!({
            "question": question,
            "project_root": project_root,
        }))
        .send()
        .await
        .map_err(|e| format!("AI service error: {}", e))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(body)
}

/// Load a reasoning trace from .Orqestra/graph/reasoning/{trace_id}.txt
#[tauri::command]
pub async fn read_trace_cmd(
    project_root: String,
    trace_id: String,
) -> Result<String, String> {
    let path = PathBuf::from(&project_root)
        .join(".Orqestra/graph/reasoning")
        .join(format!("{}.txt", trace_id));

    if !path.exists() {
        return Err(format!("Trace not found: {}", trace_id));
    }

    std::fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))
}

/// Load a commit stub from .Orqestra/graph/commits/{hash}.json
#[tauri::command]
pub async fn read_commit_stub_cmd(
    project_root: String,
    hash: String,
) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&project_root)
        .join(".Orqestra/graph/commits")
        .join(format!("{}.json", hash));

    if !path.exists() {
        return Err(format!("Commit stub not found: {}", hash));
    }

    let content = std::fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))
}
