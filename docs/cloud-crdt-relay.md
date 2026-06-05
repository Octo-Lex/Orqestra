# Cloud CRDT Relay (v2.1.0)

## Three distinct collaboration layers

### 1. Local CRDT (Loro)

- Per-file LoroDoc documents
- Two-peer offline merge verified (12 tests)
- No network dependency
- Token management for scoped access
- Snapshot persistence to `.Orqestra/crdt/`

### 2. Cloudflare Relay (Durable Object)

- WebSocket-based sync relay
- Each workspace gets a unique Durable Object instance
- Scoped tokens: read (receive only), write (push/pull), admin (reset)
- **Master secret lives only in Worker environment** — desktop never holds it
- Protocol v1 with message_id for idempotency
- Payload bounds: 1 MiB delta, 10 MiB snapshot
- Snapshot persistence in DO storage (30-day GC)
- Max 20 peers per room

### 3. Dashboard Static Deployment

- CI-built static site deployed to Cloudflare Pages
- **Not real-time** — rebuilt on master push
- Does not consume relay state
- Refreshed on deployment, not on sync

## Protocol v1

Every message includes `protocol_version: 1` and `message_id` (UUID).

Client → Server: join, delta, snapshot, leave
Server → Client: welcome, delta, peer_join, peer_leave, ack, error

Unsupported versions → error. Oversized payloads → error. Duplicate message_ids → ack without reapplying.

## Token Trust Boundary

```
Worker environment:  ORQESTRA_SYNC_MASTER (master secret)
Token generation:    POST /token/generate (server-side only)
Desktop storage:     workspace-scoped tokens only (ork_write_*, ork_read_*)
Token validation:    Worker validates HMAC before allowing sync
```

Desktop never stores or derives the master secret.

## Sync Diagnostics

`sync-status.json` in the beta diagnostics bundle contains:
- `relay_url_host` — hostname only (no protocol, no path)
- `workspace_id_hash` — SHA-256 hash (not the actual workspace ID)
- `peer_id`, `connected`, `queued_deltas`, `token_scope`, `last_sync`

No token values, no source bodies, no delta payloads, no unredacted workspace IDs.

## Local-First Guarantee

All CRDT operations work without the relay. The relay is optional collaboration infrastructure. If the relay is unavailable:
- Local CRDT continues to work
- Deltas are queued locally (max 100)
- Reconnection replays queued deltas with new message_ids
- No data loss
