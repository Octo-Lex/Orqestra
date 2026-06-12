/**
 * Orqestra Sync Relay — Cloudflare Worker entry point.
 *
 * Routes:
 *   GET  /health           — Health check
 *   POST /token/generate   — Generate workspace-scoped token (requires master auth)
 *   GET  /sync             — WebSocket upgrade → SyncRoom Durable Object
 */

import { type Env } from './types';
import { SyncRoom } from './SyncRoom';
import { generateToken, validateToken, type TokenScope } from './auth';

export { SyncRoom };


export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // Health check
    if (url.pathname === '/health') {
      return new Response(JSON.stringify({ status: 'ok', version: '1.0.0' }), {
        headers: { 'Content-Type': 'application/json' },
      });
    }

    // Token generation (server-side only, requires master auth header)
    if (url.pathname === '/token/generate' && request.method === 'POST') {
      const authHeader = request.headers.get('Authorization');
      if (!authHeader || authHeader !== `Bearer ${env.ORQESTRA_SYNC_MASTER}`) {
        return new Response(JSON.stringify({ error: 'Unauthorized' }), { status: 401 });
      }

      try {
        const body = await request.json() as { scope: TokenScope; workspace_id: string; label?: string };
        if (!body.scope || !body.workspace_id) {
          return new Response(JSON.stringify({ error: 'Missing scope or workspace_id' }), { status: 400 });
        }
        if (!['read', 'write', 'admin'].includes(body.scope)) {
          return new Response(JSON.stringify({ error: 'Invalid scope' }), { status: 400 });
        }

        const token = await generateToken(body.scope, body.workspace_id, env.ORQESTRA_SYNC_MASTER);
        return new Response(JSON.stringify({ token, scope: body.scope }), {
          headers: { 'Content-Type': 'application/json' },
        });
      } catch {
        return new Response(JSON.stringify({ error: 'Invalid request body' }), { status: 400 });
      }
    }

    // WebSocket sync → route to SyncRoom Durable Object
    if (url.pathname === '/sync') {
      const workspaceId = url.searchParams.get('workspace');
      if (!workspaceId) {
        return new Response(JSON.stringify({ error: 'Missing workspace parameter' }), { status: 400 });
      }

      const id = env.SYNC_ROOM.idFromName(workspaceId);
      const stub = env.SYNC_ROOM.get(id);

      // Forward to DO with WebSocket upgrade
      return stub.fetch(request);
    }

    return new Response('Not found', { status: 404 });
  },
};
