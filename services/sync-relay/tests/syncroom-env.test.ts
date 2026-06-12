/**
 * v2.14.6: SyncRoom Durable Object integration tests.
 *
 * Tests exercise SyncRoom through the actual production path:
 * webSocketMessage → handleJoin → validateToken (using stored env).
 *
 * Uses mock WebSocket objects to verify:
 * - Invalid token → error + close(4001)
 * - Valid token → welcome message
 * - Master token → admin welcome
 * - Wrong workspace → rejected
 */

import { describe, it, expect } from 'vitest';
import { SyncRoom } from '../src/SyncRoom';
import { generateToken } from '../src/auth';
import type { Env } from '../src/types';
import { PROTOCOL_VERSION } from '../src/protocol';

const MASTER_SECRET = 'test-master-for-do-integration';

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
    readyState: 1, // OPEN
    send(data: string) { this._sent.push(data); },
    close(code?: number, reason?: string) {
      this._closed = { code: code ?? 1000, reason: reason ?? '' };
      this.readyState = 3; // CLOSED
    },
  };
}

function createMockState() {
  const storage = new Map<string, unknown>();
  // Cloudflare DO storage.list() returns an async iterable directly (not Promise)
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

// ---------------------------------------------------------------------------
// Production-path integration tests
// ---------------------------------------------------------------------------

describe('v2.14.6: SyncRoom handleJoin production path', () => {
  it('rejects invalid token with error and close(4001)', async () => {
    const env = createMockEnv(MASTER_SECRET);
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    // Token generated with wrong master
    const badToken = await generateToken('write', 'ws-test', 'wrong-secret');
    const joinMsg = buildJoinMessage(badToken);

    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    // Should have received an error message
    expect(ws._sent.length).toBeGreaterThanOrEqual(1);

    const sent = ws._sent.map(s => JSON.parse(s));
    const errorMsg = sent.find(m => m.type === 'error');
    expect(errorMsg).toBeDefined();
    expect(errorMsg!.code).toBe('UNAUTHORIZED');

    // Should have been closed with 4001
    expect(ws._closed).not.toBeNull();
    expect(ws._closed!.code).toBe(4001);
  });

  it('accepts valid token and sends welcome', async () => {
    const env = createMockEnv(MASTER_SECRET);
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    const validToken = await generateToken('write', 'ws-test', MASTER_SECRET);
    const joinMsg = buildJoinMessage(validToken);

    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    // Should have received a welcome message
    expect(ws._sent.length).toBeGreaterThanOrEqual(1);

    const sent = ws._sent.map(s => JSON.parse(s));
    const welcome = sent.find(m => m.type === 'welcome');
    expect(welcome).toBeDefined();
    expect(welcome!.protocol_version).toBe(PROTOCOL_VERSION);
    expect(welcome!.peers).toBeDefined();

    // Should NOT have been closed
    expect(ws._closed).toBeNull();
  });

  it('accepts master token as admin and sends welcome', async () => {
    const env = createMockEnv(MASTER_SECRET);
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    // Master token is the master secret itself
    const joinMsg = buildJoinMessage(MASTER_SECRET, 'ws-admin', 99);

    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const welcome = sent.find(m => m.type === 'welcome');
    expect(welcome).toBeDefined();
    expect(ws._closed).toBeNull();
  });

  it('rejects valid token for wrong workspace', async () => {
    const env = createMockEnv(MASTER_SECRET);
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    // Token for ws-correct, but joining ws-wrong
    const token = await generateToken('write', 'ws-correct', MASTER_SECRET);
    const joinMsg = buildJoinMessage(token, 'ws-wrong', 42);

    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const errorMsg = sent.find(m => m.type === 'error');
    expect(errorMsg).toBeDefined();
    expect(errorMsg!.code).toBe('UNAUTHORIZED');

    expect(ws._closed).not.toBeNull();
    expect(ws._closed!.code).toBe(4001);
  });

  it('rejects garbage token with unauthorized', async () => {
    const env = createMockEnv(MASTER_SECRET);
    const state = createMockState();
    const room = new SyncRoom(state, env);
    room['roomWorkspaceId'] = 'ws-test';
    const ws = createMockWebSocket();

    const joinMsg = buildJoinMessage('garbage-token-value');

    await room.webSocketMessage(ws as unknown as WebSocket, joinMsg);

    const sent = ws._sent.map(s => JSON.parse(s));
    const errorMsg = sent.find(m => m.type === 'error');
    expect(errorMsg).toBeDefined();
    expect(errorMsg!.code).toBe('UNAUTHORIZED');

    expect(ws._closed).not.toBeNull();
    expect(ws._closed!.code).toBe(4001);
  });
});
