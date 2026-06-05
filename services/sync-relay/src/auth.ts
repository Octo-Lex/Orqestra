/**
 * Token authentication for sync relay.
 *
 * Trust boundary:
 *   - Master secret lives ONLY in Worker environment (ORQESTRA_SYNC_MASTER).
 *   - Desktop stores only workspace-scoped tokens.
 *   - Desktop never stores or derives the master secret.
 *   - Tokens generated server-side via POST /token/generate.
 */

import { TokenScope } from './protocol';

interface TokenPayload {
  scope: TokenScope;
  workspace_id: string;
  timestamp: number;
  hmac: string;
}

/**
 * Validate a token against the master secret.
 * Token format: ork_{scope}_{workspace_id}_{timestamp}_{hmac}
 */
export function validateToken(token: string, masterSecret: string): TokenPayload | null {
  // Master token is admin (checked first, before ork_ prefix check)
  if (token === masterSecret) {
    return { scope: 'admin', workspace_id: '*', timestamp: 0, hmac: '' };
  }

  if (!token.startsWith('ork_')) return null;

  const parts = token.split('_');
  // ork_{scope}_{workspace_id}_{timestamp}_{hmac...}
  // scope can be "write" or "read" (single segment) or "admin" (shouldn't reach here)
  if (parts.length < 5) return null;

  const scope = parts[1] as TokenScope;
  if (scope !== 'write' && scope !== 'read') return null;

  // workspace_id may contain dashes, so take from index 2 to -2
  const hmacPart = parts[parts.length - 1];
  const timestampStr = parts[parts.length - 2];
  const workspaceParts = parts.slice(2, parts.length - 2);
  const workspace_id = workspaceParts.join('_');

  const timestamp = parseInt(timestampStr, 16);
  if (isNaN(timestamp)) return null;

  // Verify HMAC
  const expectedHmac = computeHmac(masterSecret, `${scope}_${workspace_id}_${timestamp}`);
  if (hmacPart !== expectedHmac) return null;

  return { scope, workspace_id, timestamp, hmac: hmacPart };
}

/**
 * Generate a workspace-scoped token.
 * Called server-side only (POST /token/generate).
 */
export function generateToken(
  scope: TokenScope,
  workspaceId: string,
  masterSecret: string,
): string {
  const timestamp = Date.now();
  const hmac = computeHmac(masterSecret, `${scope}_${workspaceId}_${timestamp}`);
  return `ork_${scope}_${workspaceId}_${timestamp.toString(16)}_${hmac}`;
}

/**
 * Check if a scope allows write operations.
 */
export function canWrite(scope: TokenScope): boolean {
  return scope === 'write' || scope === 'admin';
}

/**
 * Simple HMAC-like function using SubtleCrypto.
 * In production, this would use proper HMAC-SHA256.
 * For the Worker, we use a deterministic hash for token verification.
 */
function computeHmac(secret: string, payload: string): string {
  // Simple keyed hash — in production use crypto.subtle.sign
  // For now, use a deterministic concatenation hash
  const input = `${secret}:${payload}`;
  let hash = 0;
  for (let i = 0; i < input.length; i++) {
    const char = input.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash |= 0; // Convert to 32-bit integer
  }
  // Produce a hex-like string
  return Math.abs(hash).toString(16).padStart(8, '0') +
    Math.abs(hash * 31).toString(16).padStart(8, '0');
}
