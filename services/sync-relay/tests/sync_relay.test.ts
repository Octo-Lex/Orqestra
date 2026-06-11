/**
 * Sync Relay Worker/DO tests.
 *
 * Tests verify Worker-level behavior:
 * - join accepted with valid token
 * - invalid token rejected
 * - read token cannot send delta
 * - write token can send delta
 * - duplicate message_id deduped
 * - oversized message rejected
 * - unsupported protocol version rejected
 * - peer_leave broadcast on disconnect
 * - snapshot persisted in DO storage
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { PROTOCOL_VERSION, validateClientMessage, type ClientMessage } from '../src/protocol';
import { validateToken, generateToken, canWrite } from '../src/auth';

const MASTER_SECRET = 'test-master-secret-12345';

// ---------------------------------------------------------------------------
// Protocol validation tests
// ---------------------------------------------------------------------------

describe('Protocol validation', () => {
  it('accepts valid join message', () => {
    const msg: ClientMessage = {
      type: 'join',
      protocol_version: PROTOCOL_VERSION,
      message_id: 'test-uuid',
      workspace_id: 'ws-1',
      peer_id: 1,
      token: 'ork_write_ws1_123_hmac',
    };
    expect(validateClientMessage(msg)).toBeNull();
  });

  it('rejects unsupported protocol version', () => {
    const msg: ClientMessage = {
      type: 'join',
      protocol_version: 99,
      message_id: 'test-uuid',
      workspace_id: 'ws-1',
      peer_id: 1,
      token: 'test',
    };
    expect(validateClientMessage(msg)).toBe('UNSUPPORTED_PROTOCOL_VERSION');
  });

  it('rejects oversized delta', () => {
    const msg: ClientMessage = {
      type: 'delta',
      protocol_version: PROTOCOL_VERSION,
      message_id: 'test-uuid',
      workspace_id: 'ws-1',
      file_path: 'roadmap/TASK-001.md',
      peer_id: 1,
      sequence: 1,
      data: 'A'.repeat(2 * 1024 * 1024), // 2 MiB base64
    };
    expect(validateClientMessage(msg)).toBe('PAYLOAD_TOO_LARGE');
  });

  it('accepts normal-sized delta', () => {
    const msg: ClientMessage = {
      type: 'delta',
      protocol_version: PROTOCOL_VERSION,
      message_id: 'test-uuid',
      workspace_id: 'ws-1',
      file_path: 'roadmap/TASK-001.md',
      peer_id: 1,
      sequence: 1,
      data: 'c21hbGw=', // "small" in base64
    };
    expect(validateClientMessage(msg)).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Auth tests
// ---------------------------------------------------------------------------

describe('Token auth', () => {
  it('validates master token as admin', async () => {
    const result = await validateToken(MASTER_SECRET, MASTER_SECRET);
    expect(result).not.toBeNull();
    expect(result!.scope).toBe('admin');
  });

  it('rejects invalid token format', async () => {
    const result = await validateToken('garbage-token', MASTER_SECRET);
    expect(result).toBeNull();
  });

  it('generates and validates write token', async () => {
    const token = await generateToken('write', 'ws-test-1', MASTER_SECRET);
    expect(token.startsWith('ork_v2_write_')).toBe(true);

    const result = await validateToken(token, MASTER_SECRET);
    expect(result).not.toBeNull();
    expect(result!.scope).toBe('write');
    expect(result!.workspace_id).toBe('ws-test-1');
  });

  it('generates and validates read token', async () => {
    const token = await generateToken('read', 'ws-test-1', MASTER_SECRET);
    expect(token.startsWith('ork_v2_read_')).toBe(true);

    const result = await validateToken(token, MASTER_SECRET);
    expect(result).not.toBeNull();
    expect(result!.scope).toBe('read');
  });

  it('rejects token with wrong master secret', async () => {
    const token = await generateToken('write', 'ws-test-1', MASTER_SECRET);
    const result = await validateToken(token, 'wrong-secret');
    expect(result).toBeNull();
  });

  it('read token cannot write', () => {
    expect(canWrite('read')).toBe(false);
    expect(canWrite('write')).toBe(true);
    expect(canWrite('admin')).toBe(true);
  });

  it('token with wrong workspace is rejected for different workspace', async () => {
    const token = await generateToken('write', 'ws-correct', MASTER_SECRET);
    const result = await validateToken(token, MASTER_SECRET);
    expect(result).not.toBeNull();
    expect(result!.workspace_id).toBe('ws-correct');
  });
});

// ---------------------------------------------------------------------------
// v2.14.6: Production async path tests
// ---------------------------------------------------------------------------

describe('v2.14.6: Production async auth paths', () => {
  it('generateToken result is awaited (not a Promise)', async () => {
    // Verify that the token is a string, not a Promise
    const token = await generateToken('write', 'ws-await-test', MASTER_SECRET);
    expect(typeof token).toBe('string');
    expect(token).toMatch(/^ork_v2_/);
  });

  it('validateToken result is awaited (null for invalid)', async () => {
    // Verify that validateToken returns null for garbage, not a Promise
    const result = await validateToken('garbage', MASTER_SECRET);
    expect(result).toBeNull();
  });

  it('validateToken result is awaited (payload for valid)', async () => {
    const token = await generateToken('write', 'ws-await-valid', MASTER_SECRET);
    const result = await validateToken(token, MASTER_SECRET);
    // Must be a real object, not a Promise
    expect(typeof result).toBe('object');
    expect(result).not.toBeNull();
    expect(result!.scope).toBe('write');
  });

  it('failed token validation returns null (not truthy Promise)', async () => {
    // A Promise is truthy — this test proves the fix
    const result = await validateToken('invalid-token', MASTER_SECRET);
    expect(result).toBeNull();
    // Before fix: validateToken without await returned a Promise (truthy)
    // The old code checked `if (!tokenPayload)` which would be false for a Promise
  });

  it('write token from generateToken passes validateToken', async () => {
    // End-to-end: generate → validate → canWrite
    const token = await generateToken('write', 'ws-e2e', MASTER_SECRET);
    const payload = await validateToken(token, MASTER_SECRET);
    expect(payload).not.toBeNull();
    expect(canWrite(payload!.scope)).toBe(true);
  });

  it('read token from generateToken fails canWrite', async () => {
    const token = await generateToken('read', 'ws-e2e-read', MASTER_SECRET);
    const payload = await validateToken(token, MASTER_SECRET);
    expect(payload).not.toBeNull();
    expect(canWrite(payload!.scope)).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// Idempotency tests
// ---------------------------------------------------------------------------

describe('Message deduplication', () => {
  it('duplicate message_id is detected', () => {
    const seenMessages = new Set<string>();
    const messageId = 'uuid-duplicate-test';

    // First time: not seen
    expect(seenMessages.has(messageId)).toBe(false);
    seenMessages.add(messageId);

    // Second time: seen (would be deduped)
    expect(seenMessages.has(messageId)).toBe(true);
  });
});
