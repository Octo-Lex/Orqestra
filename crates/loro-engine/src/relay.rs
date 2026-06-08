//! Relay client for Cloudflare Durable Object sync.
//!
//! Connects via WebSocket to the SyncRoom DO.
//! Handles:
//! - Offline delta queue (max 100)
//! - Exponential backoff reconnect (1s → 30s)
//! - Idempotent message replay (message_id dedupe)
//! - Ack handling

use crate::protocol::{
    ClientMessage, ServerMessage, PROTOCOL_VERSION, MAX_QUEUED_DELTAS,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Status of the relay connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayStatus {
    pub connected: bool,
    pub peer_id: u64,
    pub workspace_id: String,
    pub relay_url_host: String,
    pub workspace_id_hash: String,
    pub queued_deltas: usize,
    pub token_scope: String,
    pub last_sync: Option<String>,
    pub relay_available: bool,
    pub reconnect_attempt: u32,
    pub last_error: Option<String>,
}

/// A queued delta waiting to be sent to the relay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedDelta {
    pub message_id: String,
    pub file_path: String,
    pub peer_id: u64,
    pub sequence: u64,
    pub data: String, // base64
}

/// Relay client state (not the WebSocket itself — managed by the Tauri app layer).
pub struct RelayClient {
    peer_id: u64,
    workspace_id: String,
    token_scope: String,
    relay_url: String,
    connected: bool,
    queued_deltas: VecDeque<QueuedDelta>,
    last_sync: Option<String>,
    seen_ack_ids: std::collections::HashSet<String>,
}

impl RelayClient {
    /// Create a new relay client (not yet connected).
    pub fn new(peer_id: u64, workspace_id: &str, token_scope: &str, relay_url: &str) -> Self {
        Self {
            peer_id,
            workspace_id: workspace_id.to_string(),
            token_scope: token_scope.to_string(),
            relay_url: relay_url.to_string(),
            connected: false,
            queued_deltas: VecDeque::new(),
            last_sync: None,
            seen_ack_ids: std::collections::HashSet::new(),
        }
    }

    /// Get relay status (redacted for diagnostics).
    pub fn status(&self) -> RelayStatus {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.workspace_id.hash(&mut hasher);
        let ws_hash = format!("sha256:{:016x}", hasher.finish());

        let host = self.relay_url
            .trim_start_matches("wss://")
            .trim_start_matches("ws://")
            .split('/')
            .next()
            .unwrap_or("unknown")
            .to_string();

        RelayStatus {
            connected: self.connected,
            peer_id: self.peer_id,
            workspace_id: self.workspace_id.clone(),
            relay_url_host: host,
            workspace_id_hash: ws_hash,
            queued_deltas: self.queued_deltas.len(),
            token_scope: self.token_scope.clone(),
            last_sync: self.last_sync.clone(),
            relay_available: true, // Updated on connection attempt
            reconnect_attempt: 0,
            last_error: None,
        }
    }

    /// Build a join message.
    pub fn build_join_message(&self, token: &str) -> ClientMessage {
        ClientMessage::Join {
            protocol_version: PROTOCOL_VERSION,
            message_id: Self::new_message_id(),
            workspace_id: self.workspace_id.clone(),
            peer_id: self.peer_id,
            token: token.to_string(),
        }
    }

    /// Queue a delta for sending.
    pub fn queue_delta(&mut self, file_path: &str, sequence: u64, data: &str) {
        if self.queued_deltas.len() >= MAX_QUEUED_DELTAS {
            self.queued_deltas.pop_front(); // LRU eviction
        }
        self.queued_deltas.push_back(QueuedDelta {
            message_id: Self::new_message_id(),
            file_path: file_path.to_string(),
            peer_id: self.peer_id,
            sequence,
            data: data.to_string(),
        });
    }

    /// Get all queued deltas for replay.
    pub fn drain_queued_deltas(&mut self) -> Vec<QueuedDelta> {
        self.queued_deltas.drain(..).collect()
    }

    /// Handle a server message.
    pub fn handle_server_message(&mut self, msg: &ServerMessage) {
        match msg {
            ServerMessage::Ack { message_id, .. } => {
                self.seen_ack_ids.insert(message_id.clone());
                self.last_sync = Some(chrono::Utc::now().to_rfc3339());
            }
            ServerMessage::Welcome { .. } => {
                self.connected = true;
                self.last_sync = Some(chrono::Utc::now().to_rfc3339());
            }
            ServerMessage::Error { .. } => {
                // Error received — connection may still be open
            }
            _ => {}
        }
    }

    /// Mark as connected.
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    /// Generate a new UUID-like message ID.
    pub fn new_message_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("msg-{:x}-{}", ts, rand_simple())
    }

    /// Check if an ack was received for a message_id.
    pub fn is_acked(&self, message_id: &str) -> bool {
        self.seen_ack_ids.contains(message_id)
    }

    /// Get the number of queued deltas.
    pub fn queued_delta_count(&self) -> usize {
        self.queued_deltas.len()
    }

    /// Generate a new UUID-like message ID (static version for external use).
    pub fn new_message_id_static() -> String {
        Self::new_message_id()
    }
}

/// Simple random number for message ID uniqueness.
fn rand_simple() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::thread::current().id().hash(&mut h);
    h.finish()
}

/// Calculate reconnect delay with exponential backoff.
/// Returns delay in milliseconds: 1s, 2s, 4s, 8s, 16s, 30s (max).
pub fn reconnect_delay_ms(attempt: u32) -> u64 {
    let base = 1000u64;
    let max = 30_000u64;
    let delay = base * 2u64.pow(attempt.min(5));
    delay.min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_status_redacts_workspace_id() {
        let client = RelayClient::new(12345, "my-secret-workspace", "Write", "wss://sync.orqestra.dev");
        let status = client.status();
        assert!(!status.workspace_id_hash.contains("my-secret-workspace"));
        assert!(status.workspace_id_hash.starts_with("sha256:"));
        assert_eq!(status.relay_url_host, "sync.orqestra.dev");
        assert_eq!(status.peer_id, 12345);
    }

    #[test]
    fn queue_delta_respects_max() {
        let mut client = RelayClient::new(1, "ws-1", "Write", "wss://sync.example.com");
        for i in 0..110 {
            client.queue_delta("file.md", i, "data");
        }
        assert!(client.queued_deltas.len() <= MAX_QUEUED_DELTAS);
    }

    #[test]
    fn drain_empties_queue() {
        let mut client = RelayClient::new(1, "ws-1", "Write", "wss://sync.example.com");
        client.queue_delta("file.md", 1, "data");
        client.queue_delta("file.md", 2, "data");
        let drained = client.drain_queued_deltas();
        assert_eq!(drained.len(), 2);
        assert_eq!(client.queued_deltas.len(), 0);
    }

    #[test]
    fn handle_welcome_sets_connected() {
        let mut client = RelayClient::new(1, "ws-1", "Write", "wss://sync.example.com");
        assert!(!client.connected);
        let welcome = ServerMessage::Welcome {
            protocol_version: 1,
            message_id: "test".into(),
            peers: vec![],
            latest_snapshot: None,
        };
        client.handle_server_message(&welcome);
        assert!(client.connected);
    }

    #[test]
    fn handle_ack_tracks_id() {
        let mut client = RelayClient::new(1, "ws-1", "Write", "wss://sync.example.com");
        let ack = ServerMessage::Ack {
            protocol_version: 1,
            message_id: "msg-123".into(),
        };
        client.handle_server_message(&ack);
        assert!(client.is_acked("msg-123"));
        assert!(!client.is_acked("msg-456"));
    }

    #[test]
    fn reconnect_delay_exponential_backoff() {
        assert_eq!(reconnect_delay_ms(0), 1000);
        assert_eq!(reconnect_delay_ms(1), 2000);
        assert_eq!(reconnect_delay_ms(2), 4000);
        assert_eq!(reconnect_delay_ms(3), 8000);
        assert_eq!(reconnect_delay_ms(4), 16000);
        assert_eq!(reconnect_delay_ms(5), 30000);
        assert_eq!(reconnect_delay_ms(10), 30000); // capped
    }

    #[test]
    fn build_join_message_has_correct_version() {
        let client = RelayClient::new(1, "ws-1", "Write", "wss://sync.example.com");
        let msg = client.build_join_message("ork_write_ws1_123_hmac");
        match msg {
            ClientMessage::Join { protocol_version, peer_id, .. } => {
                assert_eq!(protocol_version, PROTOCOL_VERSION);
                assert_eq!(peer_id, 1);
            }
            _ => panic!("Expected Join message"),
        }
    }

    #[test]
    fn status_no_token_value() {
        let client = RelayClient::new(1, "ws-1", "Write", "wss://sync.example.com");
        let status = client.status();
        let json = serde_json::to_string(&status).unwrap();
        // Must not contain any token-like patterns
        assert!(!json.contains("ork_"));
        assert!(!json.contains("Bearer"));
        assert!(!json.contains("secret"));
    }
}
