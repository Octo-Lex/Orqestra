//! Sync relay wire protocol v1.
//!
//! Shared protocol types for Cloudflare Worker ↔ Rust RelayClient.
//! Every message includes:
//!   - protocol_version (must be 1)
//!   - message_id (UUID for idempotency)
//!   - workspace_id (routing key)
//!
//! Payload bounds:
//!   - Max delta: 1 MiB
//!   - Max snapshot: 10 MiB
//!   - Max file_path length: 512 bytes
//!   - Max queued deltas per client: 100
//!   - Max connected peers per room: 20

use serde::{Deserialize, Serialize};

/// Current protocol version.
pub const PROTOCOL_VERSION: u32 = 1;

/// Max delta payload size (1 MiB).
pub const MAX_DELTA_SIZE: usize = 1024 * 1024;

/// Max snapshot payload size (10 MiB).
pub const MAX_SNAPSHOT_SIZE: usize = 10 * 1024 * 1024;

/// Max file path length.
pub const MAX_FILE_PATH_LEN: usize = 512;

/// Max queued deltas per client.
pub const MAX_QUEUED_DELTAS: usize = 100;

/// Max connected peers per room.
pub const MAX_PEERS_PER_ROOM: usize = 20;

/// Max seen message IDs to track for deduplication.
pub const MAX_SEEN_MESSAGES: usize = 10_000;

// ---------------------------------------------------------------------------
// Client → Server messages
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "join")]
    Join {
        protocol_version: u32,
        message_id: String,
        workspace_id: String,
        peer_id: u64,
        token: String,
    },

    #[serde(rename = "delta")]
    Delta {
        protocol_version: u32,
        message_id: String,
        workspace_id: String,
        file_path: String,
        peer_id: u64,
        sequence: u64,
        data: String, // base64-encoded
    },

    #[serde(rename = "snapshot")]
    Snapshot {
        protocol_version: u32,
        message_id: String,
        workspace_id: String,
        file_path: String,
        peer_id: u64,
        data: String, // base64-encoded
    },

    #[serde(rename = "leave")]
    Leave {
        protocol_version: u32,
        message_id: String,
        workspace_id: String,
    },
}

// ---------------------------------------------------------------------------
// Server → Client messages
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "welcome")]
    Welcome {
        protocol_version: u32,
        message_id: String,
        peers: Vec<u64>,
        latest_snapshot: Option<SnapshotInfo>,
    },

    #[serde(rename = "delta")]
    Delta {
        protocol_version: u32,
        message_id: String,
        from_peer: u64,
        file_path: String,
        sequence: u64,
        data: String, // base64-encoded
    },

    #[serde(rename = "peer_join")]
    PeerJoin {
        protocol_version: u32,
        message_id: String,
        peer_id: u64,
    },

    #[serde(rename = "peer_leave")]
    PeerLeave {
        protocol_version: u32,
        message_id: String,
        peer_id: u64,
    },

    #[serde(rename = "ack")]
    Ack {
        protocol_version: u32,
        message_id: String,
    },

    #[serde(rename = "error")]
    Error {
        protocol_version: u32,
        message_id: String,
        code: String,
        message: String,
    },
}

/// Snapshot metadata (no payload in welcome).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub file_path: String,
    pub sequence: u64,
    pub size_bytes: usize,
}

// ---------------------------------------------------------------------------
// Error codes
// ---------------------------------------------------------------------------

pub const ERR_UNSUPPORTED_PROTOCOL_VERSION: &str = "UNSUPPORTED_PROTOCOL_VERSION";
pub const ERR_PAYLOAD_TOO_LARGE: &str = "PAYLOAD_TOO_LARGE";
pub const ERR_UNAUTHORIZED: &str = "UNAUTHORIZED";
pub const ERR_READ_ONLY_SCOPE: &str = "READ_ONLY_SCOPE";
pub const ERR_ROOM_FULL: &str = "ROOM_FULL";
pub const ERR_INVALID_MESSAGE: &str = "INVALID_MESSAGE";
pub const ERR_INTERNAL: &str = "INTERNAL_ERROR";

// ---------------------------------------------------------------------------
// Token scope (matches loro_engine::sync::TokenScope)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolTokenScope {
    Admin,
    Write,
    Read,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate a client message against protocol rules.
pub fn validate_client_message(msg: &ClientMessage) -> Result<(), ProtocolError> {
    // Check protocol version
    let version = match msg {
        ClientMessage::Join { protocol_version, .. } => *protocol_version,
        ClientMessage::Delta { protocol_version, .. } => *protocol_version,
        ClientMessage::Snapshot { protocol_version, .. } => *protocol_version,
        ClientMessage::Leave { protocol_version, .. } => *protocol_version,
    };

    if version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion {
            received: version,
            expected: PROTOCOL_VERSION,
        });
    }

    // Check file_path length
    if let ClientMessage::Delta { file_path, .. } | ClientMessage::Snapshot { file_path, .. } = msg {
        if file_path.len() > MAX_FILE_PATH_LEN {
            return Err(ProtocolError::FilePathTooLong {
                length: file_path.len(),
                max: MAX_FILE_PATH_LEN,
            });
        }
    }

    // Check payload size (base64 is ~4/3 of raw bytes)
    match msg {
        ClientMessage::Delta { data, .. } => {
            let estimated_bytes = (data.len() as f64 * 0.75) as usize;
            if estimated_bytes > MAX_DELTA_SIZE {
                return Err(ProtocolError::PayloadTooLarge {
                    size: estimated_bytes,
                    max: MAX_DELTA_SIZE,
                    kind: "delta",
                });
            }
        }
        ClientMessage::Snapshot { data, .. } => {
            let estimated_bytes = (data.len() as f64 * 0.75) as usize;
            if estimated_bytes > MAX_SNAPSHOT_SIZE {
                return Err(ProtocolError::PayloadTooLarge {
                    size: estimated_bytes,
                    max: MAX_SNAPSHOT_SIZE,
                    kind: "snapshot",
                });
            }
        }
        _ => {}
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Unsupported protocol version: received {received}, expected {expected}")]
    UnsupportedVersion { received: u32, expected: u32 },

    #[error("File path too long: {length} bytes, max {max}")]
    FilePathTooLong { length: usize, max: usize },

    #[error("Payload too large: {kind} is {size} bytes, max {max}")]
    PayloadTooLarge { size: usize, max: usize, kind: &'static str },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_join(version: u32) -> ClientMessage {
        ClientMessage::Join {
            protocol_version: version,
            message_id: "test-uuid".into(),
            workspace_id: "ws-1".into(),
            peer_id: 1,
            token: "ork_write_ws1_123_hmac".into(),
        }
    }

    #[test]
    fn valid_join_passes() {
        let msg = make_join(PROTOCOL_VERSION);
        assert!(validate_client_message(&msg).is_ok());
    }

    #[test]
    fn wrong_version_rejected() {
        let msg = make_join(99);
        let err = validate_client_message(&msg).unwrap_err();
        assert!(matches!(err, ProtocolError::UnsupportedVersion { .. }));
    }

    #[test]
    fn oversized_delta_rejected() {
        let msg = ClientMessage::Delta {
            protocol_version: 1,
            message_id: "test".into(),
            workspace_id: "ws-1".into(),
            file_path: "roadmap/TASK-001.md".into(),
            peer_id: 1,
            sequence: 1,
            data: "A".repeat(2 * 1024 * 1024), // 2 MiB base64
        };
        let err = validate_client_message(&msg).unwrap_err();
        assert!(matches!(err, ProtocolError::PayloadTooLarge { kind: "delta", .. }));
    }

    #[test]
    fn oversized_snapshot_rejected() {
        let msg = ClientMessage::Snapshot {
            protocol_version: 1,
            message_id: "test".into(),
            workspace_id: "ws-1".into(),
            file_path: "roadmap/TASK-001.md".into(),
            peer_id: 1,
            data: "A".repeat(20 * 1024 * 1024), // 20 MiB base64
        };
        let err = validate_client_message(&msg).unwrap_err();
        assert!(matches!(err, ProtocolError::PayloadTooLarge { kind: "snapshot", .. }));
    }

    #[test]
    fn long_file_path_rejected() {
        let msg = ClientMessage::Delta {
            protocol_version: 1,
            message_id: "test".into(),
            workspace_id: "ws-1".into(),
            file_path: "A".repeat(600),
            peer_id: 1,
            sequence: 1,
            data: "small".into(),
        };
        let err = validate_client_message(&msg).unwrap_err();
        assert!(matches!(err, ProtocolError::FilePathTooLong { .. }));
    }

    #[test]
    fn serde_roundtrip_join() {
        let msg = make_join(1);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"join\""));
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ClientMessage::Join { .. }));
    }

    #[test]
    fn serde_roundtrip_server_error() {
        let msg = ServerMessage::Error {
            protocol_version: 1,
            message_id: "test".into(),
            code: ERR_UNSUPPORTED_PROTOCOL_VERSION.into(),
            message: "Version 99 not supported".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ServerMessage::Error { .. }));
    }

    #[test]
    fn protocol_version_constant_is_1() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }
}
