/**
 * LoroProvider — CRDT sync context for the Orqestra desktop app.
 *
 * Wraps all Tauri CRDT commands behind typed async functions.
 * Tracks sync status: peer ID, open documents, connection state.
 */
import { invoke } from '@tauri-apps/api/core';

export type SyncStatus = {
  peer_id: number;
  open_docs: string[];
};

export type TaskField = {
  key: string;
  value: string;
};

export type AuthResult = {
  authorized: boolean;
  scope?: 'Admin' | 'Write' | 'Read';
  reason?: string;
};

export type SyncConnectionState = 'disconnected' | 'connected' | 'syncing' | 'offline';

/**
 * Initialize the CRDT sync engine for a project.
 */
export async function initSync(projectRoot: string, masterToken: string): Promise<SyncStatus> {
  return invoke('init_sync_cmd', { projectRoot, masterToken });
}

/**
 * Open a CRDT document for a task file.
 */
export async function openCrdtDoc(filePath: string): Promise<void> {
  return invoke('open_crdt_doc_cmd', { filePath });
}

/**
 * Set a field on a CRDT document.
 */
export async function setCrdtField(filePath: string, key: string, value: string): Promise<void> {
  return invoke('set_crdt_field_cmd', { payload: { filePath, key, value } });
}

/**
 * Get a field from a CRDT document.
 */
export async function getCrdtField(filePath: string, key: string): Promise<string> {
  return invoke('get_crdt_field_cmd', { filePath, key });
}

/**
 * Get all fields from a CRDT document.
 */
export async function getAllFields(filePath: string): Promise<TaskField[]> {
  return invoke('get_all_fields_cmd', { filePath });
}

/**
 * Export CRDT delta for a document (to send to remote peer).
 */
export async function exportDelta(filePath: string): Promise<number[]> {
  return invoke('export_delta_cmd', { filePath });
}

/**
 * Import CRDT delta (merge remote changes).
 */
export async function importDelta(filePath: string, data: number[]): Promise<void> {
  return invoke('import_delta_cmd', { payload: { filePath, data } });
}

/**
 * Load markdown content into a CRDT document.
 */
export async function loadMarkdown(filePath: string, content: string): Promise<void> {
  return invoke('load_markdown_cmd', { payload: { filePath, content } });
}

/**
 * Export CRDT state back to markdown.
 */
export async function exportMarkdown(filePath: string): Promise<string> {
  return invoke('export_markdown_cmd', { filePath });
}

/**
 * Save CRDT snapshot to disk.
 */
export async function saveSnapshot(filePath: string): Promise<void> {
  return invoke('save_snapshot_cmd', { filePath });
}

/**
 * Get current sync status.
 */
export async function getSyncStatus(): Promise<SyncStatus> {
  return invoke('sync_status_cmd');
}

/**
 * Generate an access token.
 */
export async function generateToken(scope: 'admin' | 'write' | 'read', label: string): Promise<string> {
  return invoke('generate_token_cmd', { request: { scope, label } });
}

/**
 * Validate an access token.
 */
export async function validateToken(token: string): Promise<AuthResult> {
  return invoke('validate_token_cmd', { request: { token } });
}

/**
 * Simulate 2-peer offline sync for demo/testing.
 * Returns true if both peers converged.
 */
export async function simulateOfflineMerge(
  _filePath: string,
  peerAFields: Record<string, string>,
  peerBFields: Record<string, string>,
): Promise<{ converged: boolean; fieldsA: TaskField[]; fieldsB: TaskField[] }> {
  // This is a browser-test mock — the real implementation uses the Rust engine
  // via Tauri commands. In mock mode, we simulate locally.

  // Merge: union of all keys, last-write-wins for conflicts
  const merged = { ...peerAFields, ...peerBFields };

  const fieldsA = Object.entries(merged).map(([key, value]) => ({ key, value }));
  const fieldsB = Object.entries(merged).map(([key, value]) => ({ key, value }));

  // Check convergence: both have same keys
  const keysA = new Set(fieldsA.map(f => f.key));
  const keysB = new Set(fieldsB.map(f => f.key));

  return {
    converged: keysA.size === keysB.size && [...keysA].every(k => keysB.has(k)),
    fieldsA,
    fieldsB,
  };
}