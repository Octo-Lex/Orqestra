/**
 * v2.14.8: Relay security regression tests.
 *
 * Tests verify:
 * - validateToken("", "") returns null (not admin)
 * - Empty master secret rejects all tokens
 * - Short master secret rejects all tokens
 * - Unjoined socket cannot push delta
 * - Read-only peer cannot impersonate write peer
 * - Wrong-workspace token cannot enter another workspace's room
 * - Delta/snapshot authorization uses WebSocket-bound peer
 */

import { describe, it, expect } from 'vitest';
import { SyncRoom } from '../src/SyncRoom';
import { validateToken, generateToken, canWrite } from '../src/auth';
import type { Env } from '../src/types';
import { PROTOCOL_VERSION } from '../src/protocol';

const MASTER_SECRET = 'test-master-secret-for-relay-security-tests';

// ---------------------------------------------------------------------------
// Mock helpers
// ---------------------------------------------------------------------------

interface MockWebSocket {
  send: (data: string) => void;
  close: (code?: number, reason?: string) => void;
  readyState: number;
  _sent: string[];
  _closed: { code: number; reason: string } | null;
}

function createMockWebSocket(): MockWebSocket {
  return {
    _sent: [],
    _closed: null,
    readyState: 1,
    send(data: string) { this._sent.push(data); },
    close(code?: number, reason?: string) {
      this._closed = { code: code ?? 1000, reason: reason ?? '' };
      this.readyState = 3;
    },
  };
}

function createMockState() {
  const storage = new Map<string, unknown>();
  const makeAsyncIterable = () => ({
    [Symbol.asyncIterator]() {
      const entries = [...storage.entries()];
      let i = 0;
      return {
        async next() {
          if (i >= entries.length) return { done: true, value: undefined };
          return { done: false, value: entries[i++] };
        },
      };
    },
  });
  return {
    acceptWebSocket: () => {},
    storage: {
      put: async (key: string, value: unknown) => { storage.set(key, value); },
      get: async (key: string) => storage.get(key),
      delete: async (key: string) => { storage.delete(key); },
      list: (_options?: { prefix?: string }) => makeAsyncIterable(),
    },
  } as unknown as DurableObjectState;
}

function createMockEnv(secret = MASTER_SECRET): Env {
  return {
    SYNC_ROOM: {} as DurableObjectNamespace,
    ORQESTRA_SYNC_MASTER: secret,
  };
}

function buildJoinMessage(token: string, workspaceId = 'ws-test', peerId = 1) {
  return JSON.stringify({
    type: 'join',
    protocol_version: PROTOCOL_VERSION,
    message_id: `test-msg-${Date.now()}`,
    workspace_id: workspaceId,
    peer_id: peerId,
    token,
  });
}

function buildDeltaMessage(peerId: number, messageId?: string) {
  return JSON.stringify({
    type: 'delta',
    protocol_version: PROTOCOL_VERSION,
    message_id: messageId || `delta-${Date.now()}`,
    workspace_id: 'ws-test',
    file_path: 'test.md',
    peer_id: peerId,
    sequence: 1,
    data: 'dGVzdA==', // "test" in base64
  });
}

// ---------------------------------------------------------------------------
// P0: Relay auth fail-closed
// ---------------------------------------------------------------------------

describe('v2.14.8: Relay auth fail-closed', () => {
  it('validateToken("", "") returns null, not admin', async () => {
    const result = await validateToken('', '');
    expect(result).toBeNull();
  });

  it('validateToken with empty token returns null', async () => {
    const result = await validateToken('', MASTER_SECRET);
    expect(result).toBeNull();
  });

  it('validateToken with empty master secret returns null', async () => {
    const token = await generateToken('write', 'ws-test', MASTER_SECRET);
    const result = await validateToken(token, '');
    expect(result).toBeNull();
  });

  it('validateToken with short master secret (<16) returns null', async () => {
    const result = await validateToken('short-secret', 'short-secret');
    expect(result).toBeNull();
  });

  it('valid master secret authenticates as admin', async () => {
    const result = await validateToken(MASTER_SECRET, MASTER_SECRET);
    expect(result).not.toBeNull();
    expect(result!.scope).toBe('admin');
  });

  it('absent master secret (undefined passed as empty) rejects admin', async () => {
    const result = await validateToken(MASTER_SECRET, '');
    expect(result).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// P0: WebSocket-bound peer authorization
// ---------------------------------------------------------------------------

describe('v2.14.8: WebSocket-bound peer auth', () => {
  it('unjoined socket cannot push delta', async () => {
    const env = createMockEnv();
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    // Send delta WITHOUT joining first
    const deltaMsg = buildDeltaMessage(999);
    await room.webSocketMessage(ws as unknown as WebSocket, deltaMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const errorMsg = sent.find(m => m.type === 'error');
    expect(errorMsg).toBeDefined();
    expect(errorMsg!.code).toBe('UNAUTHORIZED');
    expect(errorMsg!.message).toContain('not joined');
  });

  it('read-only peer cannot push delta even with correct peer_id', async () => {
    const env = createMockEnv();
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    // Join with read-only token
    const readToken = await generateToken('read', 'ws-test', MASTER_SECRET);
    const joinMsg = buildJoinMessage(readToken, 'ws-test', 1);
    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    // Clear sent messages (welcome)
    ws._sent.length = 0;

    // Try to push delta
    const deltaMsg = buildDeltaMessage(1);
    await room.webSocketMessage(ws as unknown as WebSocket, deltaMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const errorMsg = sent.find(m => m.type === 'error');
    expect(errorMsg).toBeDefined();
    expect(errorMsg!.code).toBe('READ_ONLY_SCOPE');
  });

  it('write peer CAN push delta after joining', async () => {
    const env = createMockEnv();
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    // Join with write token
    const writeToken = await generateToken('write', 'ws-test', MASTER_SECRET);
    const joinMsg = buildJoinMessage(writeToken, 'ws-test', 1);
    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    ws._sent.length = 0;

    // Push delta
    const deltaMsg = buildDeltaMessage(1);
    await room.webSocketMessage(ws as unknown as WebSocket, deltaMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const ack = sent.find(m => m.type === 'ack');
    expect(ack).toBeDefined();
  });

  it('peer cannot impersonate another peer_id in delta', async () => {
    const env = createMockEnv();
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';

    // Peer 1 joins with write
    const ws1 = createMockWebSocket();
    const writeToken1 = await generateToken('write', 'ws-test', MASTER_SECRET);
    await room.webSocketMessage(ws1 as unknown as WebSocket, buildJoinMessage(writeToken1, 'ws-test', 1));

    // Peer 2 joins with read
    const ws2 = createMockWebSocket();
    const readToken2 = await generateToken('read', 'ws-test', MASTER_SECRET);
    await room.webSocketMessage(ws2 as unknown as WebSocket, buildJoinMessage(readToken2, 'ws-test', 2));

    ws2._sent.length = 0;

    // Peer 2 tries to send delta claiming to be peer 1 (write peer)
    const deltaMsg = buildDeltaMessage(1); // Claims peer_id=1 but socket is peer 2
    await room.webSocketMessage(ws2 as unknown as WebSocket, deltaMsg);

    const sent = ws2._sent.map(s => JSON.parse(s));
    // Should be rejected because ws2 is registered as peer 2 (read-only)
    const errorMsg = sent.find(m => m.type === 'error');
    expect(errorMsg).toBeDefined();
    expect(errorMsg!.code).toBe('READ_ONLY_SCOPE');
  });
});

// ---------------------------------------------------------------------------
// P0: Workspace isolation
// ---------------------------------------------------------------------------

describe('v2.14.8: Workspace isolation', () => {
  it('token for workspace B cannot join workspace A room', async () => {
    const env = createMockEnv();
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    // Set room workspace to ws-A (simulating URL routing)
    room['roomWorkspaceId'] = 'ws-A';

    // Try to join with token for ws-B
    const tokenB = await generateToken('write', 'ws-B', MASTER_SECRET);
    const joinMsg = buildJoinMessage(tokenB, 'ws-B', 1);
    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const errorMsg = sent.find(m => m.type === 'error');
    expect(errorMsg).toBeDefined();
    expect(errorMsg!.code).toBe('UNAUTHORIZED');

    expect(ws._closed).not.toBeNull();
    expect(ws._closed!.code).toBe(4001);
  });

  it('admin token can join any workspace', async () => {
    const env = createMockEnv();
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    room['roomWorkspaceId'] = 'ws-A';

    // Master token has workspace_id='*' so can join any room
    const joinMsg = buildJoinMessage(MASTER_SECRET, 'ws-A', 99);
    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const welcome = sent.find(m => m.type === 'welcome');
    expect(welcome).toBeDefined();
    expect(ws._closed).toBeNull();
  });

  it('matching workspace token can join', async () => {
    const env = createMockEnv();
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    room['roomWorkspaceId'] = 'ws-test';

    const token = await generateToken('write', 'ws-test', MASTER_SECRET);
    const joinMsg = buildJoinMessage(token, 'ws-test', 1);
    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const welcome = sent.find(m => m.type === 'welcome');
    expect(welcome).toBeDefined();
  });
});


// ---------------------------------------------------------------------------
// P0: Room workspace authority (no client fallback)
// ---------------------------------------------------------------------------

describe('v2.14.8: Room workspace authority', () => {
  it('SyncRoom without roomWorkspaceId rejects join (no client fallback)', async () => {
    const env = createMockEnv();
    const state = createMockState();
    const room = new SyncRoom(state, env);
    // Do NOT set roomWorkspaceId — testing fail-closed behavior
    const ws = createMockWebSocket();

    const token = await generateToken('write', 'ws-test', MASTER_SECRET);
    const joinMsg = buildJoinMessage(token, 'ws-test', 1);
    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const errorMsg = sent.find(m => m.type === 'error');
    expect(errorMsg).toBeDefined();
    expect(errorMsg!.code).toBe('ROOM_WORKSPACE_MISSING');

    expect(ws._closed).not.toBeNull();
    expect(ws._closed!.code).toBe(4003);
  });

  it('SyncRoom with roomWorkspaceId accepts matching token', async () => {
    const env = createMockEnv();
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    room['roomWorkspaceId'] = 'ws-test';

    const token = await generateToken('write', 'ws-test', MASTER_SECRET);
    const joinMsg = buildJoinMessage(token, 'ws-test', 1);
    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const welcome = sent.find(m => m.type === 'welcome');
    expect(welcome).toBeDefined();
    expect(ws._closed).toBeNull();
  });
});
