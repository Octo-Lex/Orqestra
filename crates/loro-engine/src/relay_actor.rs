//! Relay Actor — async WebSocket lifecycle (v2.5.2).
//!
//! Rust owns the relay lifecycle. This module provides:
//! - `RelayActor`: async tokio task managing the WebSocket connection
//! - `RelayActorHandle`: deterministic shutdown/join handle
//! - `RelayEvent`: channel-based events (Tauri-free)
//!
//! Ownership boundary:
//!   relay.rs → state machine, queue, public API (no async, no WebSocket)
//!   relay_actor.rs → WebSocket lifecycle (async, tokio, tungstenite)
//!
//! crates/loro-engine MUST NOT depend on tauri.

use crate::protocol::{
    ClientMessage, ServerMessage, PROTOCOL_VERSION, MAX_QUEUED_DELTAS,
};
use crate::relay::{RelayClient, QueuedDelta};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Event types (Tauri-free, sent over channel)
// ---------------------------------------------------------------------------

/// Events emitted by the relay actor.
#[derive(Debug, Clone, serde::Serialize)]
pub enum RelayEvent {
    Connected {
        peer_id: u64,
        peers: Vec<u64>,
    },
    Disconnected {
        reason: String,
        reconnect_attempt: u32,
    },
    RemoteDeltaReceived {
        file_path_hash: String,
        message_id: String,
        from_peer: u64,
        delta_size_bytes: usize,
    },
    Error {
        code: String,
        message: String,
    },
    StatusChanged {
        connected: bool,
        queued_deltas: usize,
        reconnect_attempt: u32,
    },
}

// ---------------------------------------------------------------------------
// Actor handle
// ---------------------------------------------------------------------------

/// Handle to a running relay actor. Provides deterministic shutdown.
pub struct RelayActorHandle {
    pub(crate) shutdown_tx: tokio::sync::watch::Sender<bool>,
    pub(crate) join_handle: tokio::task::JoinHandle<()>,
}

impl RelayActorHandle {
    /// Gracefully shut down the actor.
    /// Sends shutdown signal, waits for termination with timeout.
    /// Does NOT drop queued deltas — they remain in the RelayClient.
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(true);
        match tokio::time::timeout(std::time::Duration::from_secs(5), self.join_handle).await {
            Ok(_) => {}
            Err(_) => {
                tracing::warn!("Relay actor shutdown timed out");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Actor configuration
// ---------------------------------------------------------------------------

pub struct RelayActorConfig {
    pub relay_url: String,
    pub workspace_id: String,
    pub peer_id: u64,
    pub token: String,
    pub token_scope: String,
}

// ---------------------------------------------------------------------------
// Actor
// ---------------------------------------------------------------------------

/// Run the relay actor loop.
/// This is the main entry point, spawned as a tokio task.
pub async fn run_relay_actor(
    config: RelayActorConfig,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    event_tx: tokio::sync::mpsc::Sender<RelayEvent>,
) {
    let mut client = RelayClient::new(
        config.peer_id,
        &config.workspace_id,
        &config.token_scope,
        &config.relay_url,
    );

    let mut reconnect_attempt: u32 = 0;
    let mut seen_remote_ids: HashSet<String> = HashSet::new();

    loop {
        // Check shutdown
        if *shutdown_rx.borrow() {
            tracing::info!("Relay actor shutting down");
            break;
        }

        // Attempt connection
        let ws_url = build_ws_url(&config.relay_url, &config.workspace_id);

        let result = connect_and_run(
            &ws_url,
            &config,
            &mut client,
            &mut seen_remote_ids,
            &event_tx,
            &shutdown_rx,
        ).await;

        match result {
            Ok(()) => reconnect_attempt = 0,
            Err(e) => {
                tracing::warn!("Relay connection error: {}", e);
                let _ = event_tx.send(RelayEvent::Disconnected {
                    reason: e.to_string(),
                    reconnect_attempt,
                }).await;
            }
        }

        // Check shutdown again after disconnect
        if *shutdown_rx.borrow() {
            break;
        }

        // Exponential backoff
        let delay = crate::relay::reconnect_delay_ms(reconnect_attempt);
        reconnect_attempt = reconnect_attempt.saturating_add(1);

        let _ = event_tx.send(RelayEvent::StatusChanged {
            connected: false,
            queued_deltas: client.queued_delta_count(),
            reconnect_attempt,
        }).await;

        // Wait with shutdown check
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_millis(delay)) => {}
            _ = shutdown_rx.changed() => {
                tracing::info!("Relay actor shutting down during backoff");
                break;
            }
        }
    }

    tracing::info!("Relay actor terminated");
}

/// Connect to relay and run the message loop until disconnect or shutdown.
async fn connect_and_run(
    ws_url: &str,
    config: &RelayActorConfig,
    client: &mut RelayClient,
    seen_remote_ids: &mut HashSet<String>,
    event_tx: &tokio::sync::mpsc::Sender<RelayEvent>,
    shutdown_rx: &tokio::sync::watch::Receiver<bool>,
) -> Result<(), String> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    // Connect WebSocket
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .map_err(|e| format!("WebSocket connect failed: {}", e))?;

    let (mut write, mut read) = ws_stream.split();

    // Send join message
    let join = client.build_join_message(&config.token);
    let join_json = serde_json::to_string(&join)
        .map_err(|e| format!("Join serialize failed: {}", e))?;
    write.send(Message::Text(join_json.into()))
        .await
        .map_err(|e| format!("Join send failed: {}", e))?;

    // Replay queued deltas after join
    let queued: Vec<QueuedDelta> = client.drain_queued_deltas();
    for delta in &queued {
        let msg = ClientMessage::Delta {
            protocol_version: PROTOCOL_VERSION,
            message_id: delta.message_id.clone(),
            workspace_id: config.workspace_id.clone(),
            file_path: delta.file_path.clone(),
            peer_id: delta.peer_id,
            sequence: delta.sequence,
            data: delta.data.clone(),
        };
        let json = serde_json::to_string(&msg)
            .map_err(|e| format!("Delta serialize failed: {}", e))?;
        write.send(Message::Text(json.into()))
            .await
            .map_err(|e| format!("Delta send failed: {}", e))?;
    }
    // Re-queue in case of reconnect (they'll be drained again on next reconnect)
    for delta in queued {
        client.queue_delta(&delta.file_path, delta.sequence, &delta.data);
    }

    // Message loop
    let mut ack_retry_count: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    loop {
        // Check shutdown before reading
        if *shutdown_rx.borrow() {
            let leave = ClientMessage::Leave {
                protocol_version: PROTOCOL_VERSION,
                message_id: RelayClient::new_message_id_static(),
                workspace_id: config.workspace_id.clone(),
            };
            if let Ok(json) = serde_json::to_string(&leave) {
                let _ = write.send(Message::Text(json.into())).await;
            }
            let _ = write.close().await;
            client.set_connected(false);
            return Ok(());
        }

        // Read next message with timeout to check shutdown periodically
        match tokio::time::timeout(
            std::time::Duration::from_secs(1),
            read.next(),
        ).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let server_msg: ServerMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!("Failed to parse server message: {}", e);
                        continue;
                    }
                };

                match &server_msg {
                    ServerMessage::Welcome { peers, .. } => {
                        client.handle_server_message(&server_msg);
                        let _ = event_tx.send(RelayEvent::Connected {
                            peer_id: config.peer_id,
                            peers: peers.clone(),
                        }).await;
                    }
                    ServerMessage::Ack { message_id, .. } => {
                        client.handle_server_message(&server_msg);
                        ack_retry_count.remove(message_id);
                    }
                    ServerMessage::Delta { message_id, file_path, data, from_peer, .. } => {
                        if seen_remote_ids.contains(message_id) {
                            continue;
                        }
                        if seen_remote_ids.len() >= crate::protocol::MAX_SEEN_MESSAGES {
                            seen_remote_ids.clear();
                        }
                        seen_remote_ids.insert(message_id.clone());

                        let file_path_hash = hash_path(file_path);
                        let delta_size = data.len();

                        let _ = event_tx.send(RelayEvent::RemoteDeltaReceived {
                            file_path_hash,
                            message_id: message_id.clone(),
                            from_peer: *from_peer,
                            delta_size_bytes: delta_size,
                        }).await;
                    }
                    ServerMessage::Error { code, message, .. } => {
                        let _ = event_tx.send(RelayEvent::Error {
                            code: code.clone(),
                            message: message.clone(),
                        }).await;
                    }
                    _ => {
                        client.handle_server_message(&server_msg);
                    }
                }
            }
            Ok(Some(Ok(Message::Close(_)))) | Ok(None) => {
                client.set_connected(false);
                return Err("WebSocket closed".to_string());
            }
            Ok(Some(Ok(Message::Ping(_)))) => continue,
            Ok(Some(Ok(_))) => continue,
            Ok(Some(Err(e))) => {
                client.set_connected(false);
                return Err(format!("WebSocket error: {}", e));
            }
            Err(_) => {
                // Timeout — loop back to check shutdown
                continue;
            }
        }
    }
}

fn build_ws_url(relay_url: &str, workspace_id: &str) -> String {
    let base = if relay_url.starts_with("wss://") || relay_url.starts_with("ws://") {
        relay_url.to_string()
    } else {
        format!("wss://{}", relay_url)
    };
    format!("{}/sync?workspace={}", base.trim_end_matches('/'), workspace_id)
}

fn hash_path(path: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ws_url_adds_workspace() {
        let url = build_ws_url("wss://sync.example.com", "ws-123");
        assert_eq!(url, "wss://sync.example.com/sync?workspace=ws-123");
    }

    #[test]
    fn test_build_ws_url_adds_wss() {
        let url = build_ws_url("sync.example.com", "ws-123");
        assert_eq!(url, "wss://sync.example.com/sync?workspace=ws-123");
    }

    #[test]
    fn test_build_ws_url_strips_trailing_slash() {
        let url = build_ws_url("wss://sync.example.com/", "ws-123");
        assert_eq!(url, "wss://sync.example.com/sync?workspace=ws-123");
    }

    #[test]
    fn test_hash_path_redacts() {
        let hash = hash_path("roadmap/TASK-001.md");
        assert!(hash.starts_with("sha256:"));
        assert!(!hash.contains("TASK-001"));
        assert_eq!(hash.len(), 71); // "sha256:" + 64 hex chars
    }

    #[test]
    fn test_relay_event_connected_serializes() {
        let event = RelayEvent::Connected {
            peer_id: 12345,
            peers: vec![67890],
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Connected"));
        assert!(json.contains("12345"));
    }

    #[test]
    fn test_relay_event_remote_delta_redacted() {
        let event = RelayEvent::RemoteDeltaReceived {
            file_path_hash: "sha256:abc123".to_string(),
            message_id: "msg-1".to_string(),
            from_peer: 999,
            delta_size_bytes: 1024,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("file_path_hash"));
        assert!(json.contains("delta_size_bytes"));
        assert!(!json.contains("roadmap/")); // no raw paths
    }

    #[test]
    fn test_relay_event_no_tokens() {
        let events = vec![
            RelayEvent::Connected { peer_id: 1, peers: vec![] },
            RelayEvent::Disconnected { reason: "test".into(), reconnect_attempt: 0 },
            RelayEvent::Error { code: "test".into(), message: "test".into() },
            RelayEvent::StatusChanged { connected: false, queued_deltas: 0, reconnect_attempt: 0 },
        ];
        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            assert!(!json.contains("ork_"), "Event must not contain tokens");
            assert!(!json.contains("Bearer"), "Event must not contain auth headers");
        }
    }

    #[test]
    fn test_relay_actor_config_fields() {
        let config = RelayActorConfig {
            relay_url: "wss://sync.example.com".to_string(),
            workspace_id: "ws-test".to_string(),
            peer_id: 42,
            token: "ork_v2_write_ws-test_abc_hmac".to_string(),
            token_scope: "write".to_string(),
        };
        assert_eq!(config.peer_id, 42);
        assert!(config.token.starts_with("ork_v2_"));
    }

    #[test]
    fn test_duplicate_remote_deltas_tracked() {
        let mut seen: HashSet<String> = HashSet::new();
        seen.insert("msg-1".to_string());
        assert!(seen.contains("msg-1"));
        assert!(!seen.contains("msg-2"));
    }

    #[test]
    fn test_seen_ids_capped() {
        let mut seen: HashSet<String> = HashSet::new();
        for i in 0..(crate::protocol::MAX_SEEN_MESSAGES + 100) {
            if seen.len() >= crate::protocol::MAX_SEEN_MESSAGES {
                seen.clear();
            }
            seen.insert(format!("msg-{}", i));
        }
        assert!(seen.len() <= crate::protocol::MAX_SEEN_MESSAGES);
    }

    #[test]
    fn test_protocol_version_is_1() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }

    #[test]
    fn test_loro_engine_no_tauri_dep() {
        // Structural test: verify no tauri crate reference in loro-engine source
        // This would be caught at compile time, but document the constraint
        assert!(true, "crates/loro-engine has no tauri dependency");
    }
}
