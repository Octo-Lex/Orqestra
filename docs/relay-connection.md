# Real Desktop Relay Connection (v2.5.2+)

## Architecture

```
┌──────────────┐     WebSocket      ┌─────────────────────┐
│   Desktop    │ ◄────────────────► │  Cloudflare Worker   │
│   (Rust)     │   wss://relay/sync │  SyncRoom DO         │
│              │                    │                      │
│ RelayActor   │   protocol v1      │  Broadcast deltas    │
│ LoroEngine   │   ork_v2_ tokens   │  Dedupe messages     │
└──────────────┘                    └─────────────────────┘
```

## Design Decision

**Rust owns the relay lifecycle.** Frontend renders status only.

```
relay.rs:      RelayClient, queue, status (no async, no WebSocket)
relay_actor.rs: RelayActor, WebSocket, connect/reconnect (async, tokio)
```

`crates/loro-engine` has **zero** tauri dependencies.

## Channel Boundary

```
RelayActor receives ServerDelta
  → sends RelayEvent over tokio::sync::mpsc channel
Tauri sync command layer
  → receives from channel
  → imports delta into engine
  → emits redacted frontend event
```

## Actor Lifecycle

1. `connect_relay_cmd()` spawns `RelayActor` as tokio task
2. Actor opens WebSocket to `{relay_url}/sync?workspace={id}`
3. Sends `join` with workspace-scoped token (protocol v1)
4. Receives `welcome` → transitions to connected
5. On reconnect → replays queued deltas after `welcome`
6. `disconnect_relay_cmd()` sends shutdown via `watch` channel
7. Actor sends `leave`, closes WebSocket, terminates

Double-connect guard: if already connected, returns existing status.

## Queue Semantics

- Deltas queued while offline
- Queue preserved on disconnect (NOT dropped)
- On reconnect: drain and replay all queued deltas
- Max 100 queued deltas (LRU eviction)
- Unacked deltas remain queued for retry

## Redacted Events

Tauri events contain metadata only:

```json
{
  "file_path_hash": "sha256:...",
  "message_id": "...",
  "from_peer": 12345,
  "delta_size_bytes": 1024
}
```

No raw file paths, no delta data, no tokens, no workspace IDs.

## Graceful Degradation

- Relay unavailable: local-first CRDT works without relay
- Deltas queued for later replay
- Status shows `relay_available: false`
- No errors thrown to user

## Dependencies

```toml
tokio = { version = "1", features = ["rt", "sync", "time", "macros"] }
tokio-tungstenite = { version = "0.26", features = ["native-tls"] }
futures-util = "0.3"
sha2 = "0.10"
```

## Test Evidence

- Local mock relay tests: 25 (12 relay_actor + 13 desktop)
- Cloudflare deployed relay: manual smoke test, status recorded
