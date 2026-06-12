/**
 * Token authentication for sync relay (v2.5.1).
 *
 * Trust boundary:
 *   - Master secret lives ONLY in Worker environment (ORQESTRA_SYNC_MASTER).
 *   - Desktop stores only workspace-scoped tokens.
 *   - Desktop never stores or derives the master secret.
 *   - Tokens generated server-side via POST /token/generate.
 *
 * Token format v2:
 *   ork_v2_{scope}_{workspace_id}_{timestamp_hex}_{hmac_sha256_hex}
 *
 * Legacy v1 tokens (djb2 hash) are rejected with UNSUPPORTED_TOKEN_VERSION.
 */

import { TokenScope } from './protocol';

/** Current token version */
const TOKEN_VERSION = 'v2';

/** Token version prefix */
const TOKEN_PREFIX = `ork_${TOKEN_VERSION}_`;

/** Legacy v1 prefix (no version) */
const LEGACY_PREFIX = 'ork_';

export interface TokenPayload {
  scope: TokenScope;
  workspace_id: string;
  timestamp: number;
  hmac: string;
  version: string;
}

/**
 * Constant-time hex string comparison.
 * XORs all byte pairs, accumulates differences, returns true only if zero diff.
 */
function timingSafeEqualHex(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let result = 0;
  for (let i = 0; i < a.length; i++) {
    result |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return result === 0;
}

/**
 * Compute HMAC-SHA256 using Web Crypto API (available in Cloudflare Workers).
 * Returns hex-encoded digest.
 */
async function computeHmac(secret: string, payload: string): Promise<string> {
  const encoder = new TextEncoder();
  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign'],
  );
  const signature = await crypto.subtle.sign('HMAC', key, encoder.encode(payload));
  return Array.from(new Uint8Array(signature))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * Validate a token against the master secret.
 * Token format v2: ork_v2_{scope}_{workspace_id}_{timestamp_hex}_{hmac_sha256_hex}
 *
 * Returns null for:
 *   - Legacy v1 tokens → UNSUPPORTED_TOKEN_VERSION error thrown
 *   - Invalid format
 *   - Wrong HMAC
 *   - Expired tokens (older than 24h)
 */
export async function validateToken(
  token: string,
  masterSecret: string,
): Promise<TokenPayload | null> {
  // Fail closed: reject empty tokens or empty master secret
  if (!token || !masterSecret) {
    return null;
  }

  // Master secret must have minimum length (16 chars)
  if (masterSecret.length < 16) {
    return null;
  }

  // Master token is admin (checked first, before prefix check)
  // Both token and masterSecret are guaranteed non-empty at this point
  if (timingSafeEqualHex(token, masterSecret) && token.length === masterSecret.length) {
    return { scope: 'admin', workspace_id: '*', timestamp: 0, hmac: '', version: 'master' };
  }

  // Reject legacy v1 tokens (ork_ without v2)
  if (token.startsWith(LEGACY_PREFIX) && !token.startsWith(TOKEN_PREFIX)) {
    throw new Error('UNSUPPORTED_TOKEN_VERSION: Legacy v1 tokens are no longer accepted');
  }

  if (!token.startsWith(TOKEN_PREFIX)) return null;

  // Strip prefix: ork_v2_
  const body = token.slice(TOKEN_PREFIX.length);
  const parts = body.split('_');

  // {scope}_{workspace_id_parts...}_{timestamp_hex}_{hmac_hex}
  // hmac is 64 chars (SHA-256), timestamp is hex
  // Need at least: scope, one workspace part, timestamp, hmac = 4 parts
  if (parts.length < 4) return null;

  const hmacPart = parts[parts.length - 1];
  const timestampStr = parts[parts.length - 2];

  // HMAC must be 64 hex chars (SHA-256)
  if (hmacPart.length !== 64 || !/^[0-9a-f]+$/.test(hmacPart)) return null;

  const scope = parts[0] as TokenScope;
  if (scope !== 'write' && scope !== 'read') return null;

  const workspaceParts = parts.slice(1, parts.length - 2);
  const workspace_id = workspaceParts.join('_');
  if (!workspace_id) return null;

  const timestamp = parseInt(timestampStr, 16);
  if (isNaN(timestamp)) return null;

  // Token expiry: 24 hours
  const age = Date.now() - timestamp;
  if (age < 0 || age > 24 * 60 * 60 * 1000) {
    return null; // Expired or future-dated
  }

  // Verify HMAC with constant-time comparison
  const payload = `${TOKEN_VERSION}_${scope}_${workspace_id}_${timestamp}`;
  const expectedHmac = await computeHmac(masterSecret, payload);
  if (!timingSafeEqualHex(expectedHmac, hmacPart)) return null;

  return { scope, workspace_id, timestamp, hmac: hmacPart, version: TOKEN_VERSION };
}

/**
 * Generate a workspace-scoped token (v2).
 * Called server-side only (POST /token/generate).
 */
export async function generateToken(
  scope: TokenScope,
  workspaceId: string,
  masterSecret: string,
): Promise<string> {
  const timestamp = Date.now();
  const payload = `${TOKEN_VERSION}_${scope}_${workspaceId}_${timestamp}`;
  const hmac = await computeHmac(masterSecret, payload);
  return `ork_${TOKEN_VERSION}_${scope}_${workspaceId}_${timestamp.toString(16)}_${hmac}`;
}

/**
 * Check if a scope allows write operations.
 */
export function canWrite(scope: TokenScope): boolean {
  return scope === 'write' || scope === 'admin';
}
