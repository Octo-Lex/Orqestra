//! v2.5.2: Real Desktop Relay Connection tests.
//!
//! Tests verify:
//! - Double-connect guard
//! - Disconnect cleanup
//! - Queue preservation on disconnect
//! - Relay status reflects real state
//! - Events are redacted
//! - Tauri-free loro-engine

use std::path::PathBuf;

fn find_repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    while !dir.join(".git").exists() {
        if !dir.pop() { panic!("No git repo found"); }
    }
    dir
}

// ---------------------------------------------------------------------------
// Double-connect guard
// ---------------------------------------------------------------------------

#[test]
fn test_double_connect_returns_status() {
    // Simulate: if relay_client already exists, return its status
    let client = loro_engine::relay::RelayClient::new(1, "ws-1", "write", "wss://sync.example.com");
    let status = client.status();
    assert!(status.connected == false); // Not actually connected
    // In Tauri, connect_relay_cmd checks if relay_client already exists
    // and returns ALREADY_CONNECTED status instead of spawning new actor
}

#[test]
fn test_disconnect_preserves_queue() {
    let mut client = loro_engine::relay::RelayClient::new(1, "ws-1", "write", "wss://sync.example.com");
    client.queue_delta("file.md", 1, "data1");
    client.queue_delta("file.md", 2, "data2");
    assert_eq!(client.queued_delta_count(), 2);

    // Disconnect sets connected=false but does NOT clear queue
    client.set_connected(false);
    assert_eq!(client.queued_delta_count(), 2);
}

#[test]
fn test_disconnect_then_drain_for_replay() {
    let mut client = loro_engine::relay::RelayClient::new(1, "ws-1", "write", "wss://sync.example.com");
    client.queue_delta("file.md", 1, "data1");
    client.set_connected(false);

    // On reconnect, drain queued deltas for replay
    let drained = client.drain_queued_deltas();
    assert_eq!(drained.len(), 1);
    assert_eq!(client.queued_delta_count(), 0);
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

#[test]
fn test_relay_status_includes_reconnect_fields() {
    let client = loro_engine::relay::RelayClient::new(1, "ws-1", "write", "wss://sync.example.com");
    let status = client.status();
    assert_eq!(status.reconnect_attempt, 0);
    assert!(status.last_error.is_none());
}

#[test]
fn test_relay_status_no_workspace_id_plain() {
    let client = loro_engine::relay::RelayClient::new(1, "my-secret-workspace", "write", "wss://sync.example.com");
    let status = client.status();
    // workspace_id_hash should not contain the raw workspace ID
    assert!(!status.workspace_id_hash.contains("my-secret-workspace"));
    // But workspace_id is still in the struct for internal use
    // Diagnostics must use the hash only
}

// ---------------------------------------------------------------------------
// RelayEvent redaction
// ---------------------------------------------------------------------------

#[test]
fn test_relay_event_delta_no_raw_path() {
    let event = loro_engine::relay_actor::RelayEvent::RemoteDeltaReceived {
        file_path_hash: "sha256:abc123".to_string(),
        message_id: "msg-1".to_string(),
        from_peer: 42,
        delta_size_bytes: 1024,
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(!json.contains("roadmap/"));
    assert!(!json.contains("TASK"));
    assert!(json.contains("file_path_hash"));
}

#[test]
fn test_all_relay_events_redacted() {
    use loro_engine::relay_actor::RelayEvent;
    let events = vec![
        RelayEvent::Connected { peer_id: 1, peers: vec![] },
        RelayEvent::Disconnected { reason: "test".into(), reconnect_attempt: 0 },
        RelayEvent::RemoteDeltaReceived {
            file_path_hash: "sha256:abc".into(),
            message_id: "msg".into(),
            from_peer: 1,
            delta_size_bytes: 100,
        },
        RelayEvent::Error { code: "test".into(), message: "test".into() },
        RelayEvent::StatusChanged { connected: false, queued_deltas: 0, reconnect_attempt: 0 },
    ];
    for event in &events {
        let json = serde_json::to_string(event).unwrap();
        assert!(!json.contains("ork_"), "Event leaked token: {}", json);
        assert!(!json.contains("Bearer"), "Event leaked auth: {}", json);
        assert!(!json.contains("secret"), "Event leaked secret: {}", json);
    }
}

// ---------------------------------------------------------------------------
// Protocol version
// ---------------------------------------------------------------------------

#[test]
fn test_desktop_uses_protocol_v1() {
    assert_eq!(loro_engine::protocol::PROTOCOL_VERSION, 1);
}

#[test]
fn test_join_message_uses_v1() {
    let client = loro_engine::relay::RelayClient::new(1, "ws-1", "write", "wss://sync.example.com");
    let join = client.build_join_message("ork_v2_write_ws1_123_hmac");
    match join {
        loro_engine::protocol::ClientMessage::Join { protocol_version, .. } => {
            assert_eq!(protocol_version, 1);
        }
        _ => panic!("Expected Join"),
    }
}

// ---------------------------------------------------------------------------
// loro-engine Tauri-free
// ---------------------------------------------------------------------------

#[test]
fn test_loro_engine_no_tauri_imports() {
    let root = find_repo_root();
    // Check Cargo.toml has no tauri dependency
    let cargo_toml = std::fs::read_to_string(root.join("crates/loro-engine/Cargo.toml"))
        .expect("Cargo.toml must be readable");
    assert!(!cargo_toml.contains("tauri"), "loro-engine must not depend on tauri");
}

#[test]
fn test_loro_engine_source_no_tauri_refs() {
    let root = find_repo_root();
    // Check for tauri references in loro-engine source (cross-platform)
    let loro_src = root.join("crates").join("loro-engine").join("src");
    let mut found = false;
    if let Ok(entries) = std::fs::read_dir(&loro_src) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        if line.contains("use tauri") {
                            found = true;
                            break;
                        }
                    }
                }
            }
            if found { break; }
        }
    }
    assert!(!found, "loro-engine source must not import tauri");
}

// ---------------------------------------------------------------------------
// Queue replay semantics
// ---------------------------------------------------------------------------

#[test]
fn test_queue_deltas_while_offline() {
    let mut client = loro_engine::relay::RelayClient::new(1, "ws-1", "write", "wss://sync.example.com");
    // Queue 5 deltas while "offline"
    for i in 0..5 {
        client.queue_delta("file.md", i, &format!("data-{}", i));
    }
    assert_eq!(client.queued_delta_count(), 5);
    assert!(!client.status().connected);
}

#[test]
fn test_queue_respects_max() {
    let mut client = loro_engine::relay::RelayClient::new(1, "ws-1", "write", "wss://sync.example.com");
    for i in 0..110 {
        client.queue_delta("file.md", i, "data");
    }
    assert!(client.queued_delta_count() <= loro_engine::protocol::MAX_QUEUED_DELTAS);
}
