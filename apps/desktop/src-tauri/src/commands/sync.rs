use loro_engine::{LoroEngine, relay::RelayClient, sync::{AuthResult, TokenManager, TokenScope}};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct SyncState {
    pub engine: Mutex<Option<LoroEngine>>,
    pub token_manager: Mutex<TokenManager>,
    pub relay_client: Mutex<Option<RelayClient>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldPayload {
    pub file_path: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeltaPayload {
    pub file_path: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkdownPayload {
    pub file_path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStatus {
    pub peer_id: u64,
    pub open_docs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenGenerateRequest {
    pub scope: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenValidateRequest {
    pub token: String,
}

/// Initialize the CRDT engine for a project.
#[tauri::command]
pub fn init_sync_cmd(
    state: State<'_, SyncState>,
    project_root: String,
    master_token: String,
) -> Result<SyncStatus, String> {
    let snapshot_dir = std::path::Path::new(&project_root)
        .join(".Orqestra")
        .join("crdt");

    let engine = LoroEngine::new(&snapshot_dir);
    let peer_id = engine.peer_id();

    *state.engine.lock().unwrap() = Some(engine);
    *state.token_manager.lock().unwrap() = TokenManager::new(Some(&master_token));

    Ok(SyncStatus {
        peer_id,
        open_docs: vec![],
    })
}

/// Open a CRDT document for a task file.
#[tauri::command]
pub fn open_crdt_doc_cmd(
    state: State<'_, SyncState>,
    file_path: String,
) -> Result<(), String> {
    let mut guard = state.engine.lock().unwrap();
    let engine = guard.as_mut().ok_or("Sync engine not initialized")?;
    engine.open_doc(&file_path).map_err(|e| e.to_string())
}

/// Set a field on a CRDT document.
#[tauri::command]
pub fn set_crdt_field_cmd(
    state: State<'_, SyncState>,
    payload: FieldPayload,
) -> Result<(), String> {
    let guard = state.engine.lock().unwrap();
    let engine = guard.as_ref().ok_or("Sync engine not initialized")?;
    engine.set_field(&payload.file_path, &payload.key, &payload.value)
        .map_err(|e| e.to_string())
}

/// Get a field from a CRDT document.
#[tauri::command]
pub fn get_crdt_field_cmd(
    state: State<'_, SyncState>,
    file_path: String,
    key: String,
) -> Result<String, String> {
    let guard = state.engine.lock().unwrap();
    let engine = guard.as_ref().ok_or("Sync engine not initialized")?;
    engine.get_field(&file_path, &key).map_err(|e| e.to_string())
}

/// Get all fields from a CRDT document.
#[tauri::command]
pub fn get_all_fields_cmd(
    state: State<'_, SyncState>,
    file_path: String,
) -> Result<Vec<loro_engine::engine::TaskField>, String> {
    let guard = state.engine.lock().unwrap();
    let engine = guard.as_ref().ok_or("Sync engine not initialized")?;
    engine.get_all_fields(&file_path).map_err(|e| e.to_string())
}

/// Export CRDT delta for a document.
#[tauri::command]
pub fn export_delta_cmd(
    state: State<'_, SyncState>,
    file_path: String,
) -> Result<Vec<u8>, String> {
    let guard = state.engine.lock().unwrap();
    let engine = guard.as_ref().ok_or("Sync engine not initialized")?;
    engine.export_delta(&file_path).map_err(|e| e.to_string())
}

/// Import CRDT delta (merge remote changes).
#[tauri::command]
pub fn import_delta_cmd(
    state: State<'_, SyncState>,
    payload: DeltaPayload,
) -> Result<(), String> {
    let guard = state.engine.lock().unwrap();
    let engine = guard.as_ref().ok_or("Sync engine not initialized")?;
    engine.import_delta(&payload.file_path, &payload.data).map_err(|e| e.to_string())
}

/// Load markdown content into a CRDT document.
#[tauri::command]
pub fn load_markdown_cmd(
    state: State<'_, SyncState>,
    payload: MarkdownPayload,
) -> Result<(), String> {
    let mut guard = state.engine.lock().unwrap();
    let engine = guard.as_mut().ok_or("Sync engine not initialized")?;
    engine.load_from_markdown(&payload.file_path, &payload.content)
        .map_err(|e| e.to_string())
}

/// Export CRDT state to markdown.
#[tauri::command]
pub fn export_markdown_cmd(
    state: State<'_, SyncState>,
    file_path: String,
) -> Result<String, String> {
    let guard = state.engine.lock().unwrap();
    let engine = guard.as_ref().ok_or("Sync engine not initialized")?;
    engine.export_to_markdown(&file_path).map_err(|e| e.to_string())
}

/// Save CRDT snapshot to disk.
#[tauri::command]
pub fn save_snapshot_cmd(
    state: State<'_, SyncState>,
    file_path: String,
) -> Result<(), String> {
    let guard = state.engine.lock().unwrap();
    let engine = guard.as_ref().ok_or("Sync engine not initialized")?;
    engine.save_snapshot(&file_path).map_err(|e| e.to_string())
}

/// Get sync status.
#[tauri::command]
pub fn sync_status_cmd(
    state: State<'_, SyncState>,
) -> Result<SyncStatus, String> {
    let guard = state.engine.lock().unwrap();
    match guard.as_ref() {
        Some(engine) => Ok(SyncStatus {
            peer_id: engine.peer_id(),
            open_docs: engine.open_docs(),
        }),
        None => Err("Sync engine not initialized".to_string()),
    }
}

/// Generate an access token.
#[tauri::command]
pub fn generate_token_cmd(
    state: State<'_, SyncState>,
    request: TokenGenerateRequest,
) -> Result<String, String> {
    let guard = state.token_manager.lock().unwrap();
    let scope = match request.scope.as_str() {
        "admin" => TokenScope::Admin,
        "write" => TokenScope::Write,
        "read" => TokenScope::Read,
        _ => return Err("Invalid scope. Use: admin, write, read".to_string()),
    };
    Ok(guard.generate(scope, &request.label)?)
}

/// Validate an access token.
#[tauri::command]
pub fn validate_token_cmd(
    state: State<'_, SyncState>,
    request: TokenValidateRequest,
) -> Result<AuthResult, String> {
    let guard = state.token_manager.lock().unwrap();
    Ok(guard.validate(&request.token))
}

// ---------------------------------------------------------------------------
// v2.1.0: Relay commands
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRelayRequest {
    pub relay_url: String,
    pub workspace_id: String,
    pub token_scope: String,
}

/// Connect to a relay (stores client state, actual WebSocket managed by frontend).
#[tauri::command]
pub fn connect_relay_cmd(
    state: State<'_, SyncState>,
    request: ConnectRelayRequest,
) -> Result<loro_engine::relay::RelayStatus, String> {
    // Get peer_id from engine
    let peer_id = {
        let guard = state.engine.lock().unwrap();
        guard.as_ref().map(|e| e.peer_id()).unwrap_or(0)
    };

    let client = RelayClient::new(
        peer_id,
        &request.workspace_id,
        &request.token_scope,
        &request.relay_url,
    );

    let status = client.status();
    *state.relay_client.lock().unwrap() = Some(client);
    Ok(status)
}

/// Disconnect from relay.
#[tauri::command]
pub fn disconnect_relay_cmd(
    state: State<'_, SyncState>,
) -> Result<(), String> {
    let mut guard = state.relay_client.lock().unwrap();
    if let Some(ref mut client) = *guard {
        client.set_connected(false);
    }
    *guard = None;
    Ok(())
}

/// Get relay status (redacted).
#[tauri::command]
pub fn relay_status_cmd(
    state: State<'_, SyncState>,
) -> Result<loro_engine::relay::RelayStatus, String> {
    let guard = state.relay_client.lock().unwrap();
    match guard.as_ref() {
        Some(client) => Ok(client.status()),
        None => Ok(loro_engine::relay::RelayStatus {
            connected: false,
            peer_id: 0,
            workspace_id: String::new(),
            relay_url_host: String::new(),
            workspace_id_hash: String::new(),
            queued_deltas: 0,
            token_scope: "none".to_string(),
            last_sync: None,
            relay_available: false,
            reconnect_attempt: 0,
            last_error: None,
        }),
    }
}
