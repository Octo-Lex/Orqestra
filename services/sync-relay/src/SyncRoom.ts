/**
 * SyncRoom — Cloudflare Durable Object.
 *
 * Each workspace gets a unique DO instance.
 * - Tracks connected peers via WebSocket
 * - Broadcasts deltas (deduped by message_id)
 * - Persists latest snapshot to DO storage
 * - Enforces payload bounds and protocol version
 * - Garbage collects old snapshots (30 days)
 */

import {
  PROTOCOL_VERSION,
  MAX_PEERS_PER_ROOM,
  MAX_SEEN_MESSAGES,
  validateClientMessage,
  ERR_UNSUPPORTED_PROTOCOL_VERSION,
  ERR_ROOM_FULL,
  ERR_READ_ONLY_SCOPE,
  ERR_INVALID_MESSAGE,
  type ClientMessage,
  type ServerMessage,
  type ServerAck,
  type ServerError,
  type ServerWelcome,
  type ServerPeerJoin,
  type ServerPeerLeave,
  type ServerDelta,
} from './protocol';
import { type Env } from './types';
import { validateToken, canWrite, type TokenPayload } from './auth';

interface Peer {
  peer_id: number;
  websocket: WebSocket;
  scope: TokenPayload | null;
}

interface Snapshot {
  file_path: string;
  data: string; // base64
  sequence: number;
  stored_at: number; // epoch ms
}

export class SyncRoom implements DurableObject {
  private state: DurableObjectState;
  private peers: Map<number, Peer> = new Map();
  private seenMessages: Set<string> = new Set();
  private snapshots: Map<string, Snapshot> = new Map();

  private env: Env;

  private roomWorkspaceId: string = '';

  constructor(state: DurableObjectState, env: Env) {
    this.state = state;
    this.env = env;
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);

    // Extract authoritative workspace ID from URL (set by Worker routing)
    if (url.pathname === '/sync') {
      const wsId = url.searchParams.get('workspace');
      if (wsId) {
        this.roomWorkspaceId = wsId;
      }
    }

    // WebSocket upgrade
    if (url.pathname === '/sync' && request.headers.get('Upgrade') === 'websocket') {
      const pair = new WebSocketPair();
      const client = pair[0];
      const server = pair[1];

      // Accept without peer_id — will be assigned on join message
      this.state.acceptWebSocket(server);

      return new Response(null, { status: 101, webSocket: client });
    }

    // Token generation endpoint (server-side only)
    if (url.pathname === '/token/generate' && request.method === 'POST') {
      return new Response('Token generation requires master secret', { status: 403 });
    }

    return new Response('Not found', { status: 404 });
  }

  async webSocketMessage(ws: WebSocket, message: string | ArrayBuffer) {
    if (typeof message !== 'string') {
      this.sendError(ws, '', ERR_INVALID_MESSAGE, 'Binary messages not supported');
      return;
    }

    let msg: ClientMessage;
    try {
      msg = JSON.parse(message);
    } catch {
      this.sendError(ws, '', ERR_INVALID_MESSAGE, 'Invalid JSON');
      return;
    }

    // Validate protocol rules
    const validationError = validateClientMessage(msg);
    if (validationError) {
      this.sendError(ws, 'message_id' in msg ? msg.message_id : '', validationError, validationError);
      return;
    }

    switch (msg.type) {
      case 'join':
        await this.handleJoin(ws, msg);
        break;
      case 'delta':
        await this.handleDelta(ws, msg);
        break;
      case 'snapshot':
        await this.handleSnapshot(ws, msg);
        break;
      case 'leave':
        this.handleLeave(ws, msg);
        break;
    }
  }

  async webSocketClose(ws: WebSocket, code: number, _reason: string) {
    // Find and remove peer
    for (const [peerId, peer] of this.peers) {
      if (peer.websocket === ws) {
        this.peers.delete(peerId);
        this.broadcastPeerLeave(peerId);
        break;
      }
    }
  }

  private async handleJoin(ws: WebSocket, msg: ClientMessage & { type: 'join' }) {
    const { message_id, peer_id, token, workspace_id } = msg;

    // Check room capacity
    if (this.peers.size >= MAX_PEERS_PER_ROOM) {
      this.sendError(ws, message_id, ERR_ROOM_FULL, `Max ${MAX_PEERS_PER_ROOM} peers per room`);
      ws.close(1013, 'Room full');
      return;
    }

    // Fail closed: reject if master secret is not configured
    const masterSecret = this.env.ORQESTRA_SYNC_MASTER || '';
    if (!masterSecret) {
      this.sendError(ws, message_id, 'SERVER_CONFIG_ERROR', 'Server authentication not configured');
      ws.close(4003, 'Server config error');
      return;
    }

    // Validate token
    const tokenPayload = await validateToken(token, masterSecret);
    if (!tokenPayload) {
      this.sendError(ws, message_id, 'UNAUTHORIZED', 'Invalid token');
      ws.close(4001, 'Unauthorized');
      return;
    }

    // Check workspace scope against ROOM workspace (authoritative), not message workspace
    const authoritativeWorkspace = this.roomWorkspaceId || workspace_id;
    if (tokenPayload.workspace_id !== '*' && tokenPayload.workspace_id !== authoritativeWorkspace) {
      this.sendError(ws, message_id, 'UNAUTHORIZED', 'Token not valid for this workspace');
      ws.close(4001, 'Unauthorized');
      return;
    }

    // Register peer
    this.peers.set(peer_id, { peer_id, websocket: ws, scope: tokenPayload });

    // Load snapshots from storage
    await this.loadSnapshots();

    // Send welcome
    const welcome: ServerWelcome = {
      type: 'welcome',
      protocol_version: PROTOCOL_VERSION,
      message_id,
      peers: Array.from(this.peers.keys()).filter(id => id !== peer_id),
      latest_snapshot: null, // Could populate from snapshots
    };
    ws.send(JSON.stringify(welcome));

    // Broadcast peer_join to others
    this.broadcastPeerJoin(peer_id);
  }

  private async handleDelta(ws: WebSocket, msg: ClientMessage & { type: 'delta' }) {
    const { message_id, file_path, sequence, data } = msg;

    // Dedupe
    if (this.seenMessages.has(message_id)) {
      this.sendAck(ws, message_id);
      return;
    }

    // Authorize: find peer by WebSocket, not by message-provided peer_id
    const peer = this.findPeerBySocket(ws);
    if (!peer) {
      this.sendError(ws, message_id, 'UNAUTHORIZED', 'Socket has not joined the room');
      return;
    }
    if (!peer.scope || !canWrite(peer.scope.scope)) {
      this.sendError(ws, message_id, ERR_READ_ONLY_SCOPE, 'Read-only token cannot push deltas');
      return;
    }

    // Track message
    this.trackMessage(message_id);

    // Broadcast to other peers
    const broadcast: ServerDelta = {
      type: 'delta',
      protocol_version: PROTOCOL_VERSION,
      message_id,
      from_peer: peer.peer_id,
      file_path,
      sequence,
      data,
    };

    for (const [id, p] of this.peers) {
      if (id !== peer.peer_id && p.websocket.readyState === WebSocket.READY_STATE_OPEN) {
        p.websocket.send(JSON.stringify(broadcast));
      }
    }

    // Ack to sender
    this.sendAck(ws, message_id);
  }

  private async handleSnapshot(ws: WebSocket, msg: ClientMessage & { type: 'snapshot' }) {
    const { message_id, file_path, data } = msg;

    // Dedupe
    if (this.seenMessages.has(message_id)) {
      this.sendAck(ws, message_id);
      return;
    }

    // Authorize: find peer by WebSocket, not by message-provided peer_id
    const peer = this.findPeerBySocket(ws);
    if (!peer) {
      this.sendError(ws, message_id, 'UNAUTHORIZED', 'Socket has not joined the room');
      return;
    }
    if (!peer.scope || !canWrite(peer.scope.scope)) {
      this.sendError(ws, message_id, ERR_READ_ONLY_SCOPE, 'Read-only token cannot push snapshots');
      return;
    }

    // Track message
    this.trackMessage(message_id);

    // Persist snapshot
    const snapshot: Snapshot = {
      file_path,
      data,
      sequence: Date.now(),
      stored_at: Date.now(),
    };
    this.snapshots.set(file_path, snapshot);

    // Persist to DO storage
    await this.state.storage.put(`snapshot:${file_path}`, snapshot);

    // GC old snapshots (30 days)
    const thirtyDaysAgo = Date.now() - 30 * 24 * 60 * 60 * 1000;
    for (const [key, snap] of this.snapshots) {
      if (snap.stored_at < thirtyDaysAgo) {
        this.snapshots.delete(key);
        await this.state.storage.delete(`snapshot:${key}`);
      }
    }

    this.sendAck(ws, message_id);
  }

  private handleLeave(ws: WebSocket, msg: ClientMessage & { type: 'leave' }) {
    const { message_id, workspace_id } = msg;

    // Find peer by websocket
    let leavingPeerId: number | null = null;
    for (const [peerId, peer] of this.peers) {
      if (peer.websocket === ws) {
        leavingPeerId = peerId;
        break;
      }
    }

    if (leavingPeerId !== null) {
      this.peers.delete(leavingPeerId);
      this.broadcastPeerLeave(leavingPeerId);
    }

    this.sendAck(ws, message_id);
    ws.close(1000, 'Goodbye');
  }

  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------

  private sendAck(ws: WebSocket, messageId: string) {
    const ack: ServerAck = {
      type: 'ack',
      protocol_version: PROTOCOL_VERSION,
      message_id: messageId,
    };
    try { ws.send(JSON.stringify(ack)); } catch { /* ws may be closed */ }
  }

  private sendError(ws: WebSocket, messageId: string, code: string, message: string) {
    const err: ServerError = {
      type: 'error',
      protocol_version: PROTOCOL_VERSION,
      message_id: messageId,
      code,
      message,
    };
    try { ws.send(JSON.stringify(err)); } catch { /* ws may be closed */ }
  }

  private broadcastPeerJoin(peerId: number) {
    const msg: ServerPeerJoin = {
      type: 'peer_join',
      protocol_version: PROTOCOL_VERSION,
      message_id: crypto.randomUUID(),
      peer_id: peerId,
    };
    this.broadcast(JSON.stringify(msg), peerId);
  }

  private broadcastPeerLeave(peerId: number) {
    const msg: ServerPeerLeave = {
      type: 'peer_leave',
      protocol_version: PROTOCOL_VERSION,
      message_id: crypto.randomUUID(),
      peer_id: peerId,
    };
    this.broadcast(JSON.stringify(msg), null);
  }

  private broadcast(message: string, excludePeerId: number | null) {
    for (const [id, peer] of this.peers) {
      if (id !== excludePeerId && peer.websocket.readyState === WebSocket.READY_STATE_OPEN) {
        try { peer.websocket.send(message); } catch { /* ignore */ }
      }
    }
  }

  /** Find the peer associated with a given WebSocket. */
  private findPeerBySocket(ws: WebSocket): Peer | undefined {
    for (const [, peer] of this.peers) {
      if (peer.websocket === ws) {
        return peer;
      }
    }
    return undefined;
  }

  private trackMessage(messageId: string) {
    if (this.seenMessages.size >= MAX_SEEN_MESSAGES) {
      // Evict oldest (first entry in the Set)
      const first = this.seenMessages.values().next().value;
      if (first) this.seenMessages.delete(first);
    }
    this.seenMessages.add(messageId);
  }

  private async loadSnapshots() {
    const entries = this.state.storage.list({ prefix: 'snapshot:' });
    for await (const [key, value] of entries) {
      const snap = value as Snapshot;
      this.snapshots.set(key.toString().replace('snapshot:', ''), snap);
    }
  }
}
