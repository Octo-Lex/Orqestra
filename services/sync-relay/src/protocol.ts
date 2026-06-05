/**
 * Protocol types shared between Worker and test suite.
 * Must match crates/loro-engine/src/protocol.rs exactly.
 */

export const PROTOCOL_VERSION = 1;
export const MAX_DELTA_SIZE = 1024 * 1024; // 1 MiB
export const MAX_SNAPSHOT_SIZE = 10 * 1024 * 1024; // 10 MiB
export const MAX_FILE_PATH_LEN = 512;
export const MAX_QUEUED_DELTAS = 100;
export const MAX_PEERS_PER_ROOM = 20;
export const MAX_SEEN_MESSAGES = 10_000;

// Error codes
export const ERR_UNSUPPORTED_PROTOCOL_VERSION = 'UNSUPPORTED_PROTOCOL_VERSION';
export const ERR_PAYLOAD_TOO_LARGE = 'PAYLOAD_TOO_LARGE';
export const ERR_UNAUTHORIZED = 'UNAUTHORIZED';
export const ERR_READ_ONLY_SCOPE = 'READ_ONLY_SCOPE';
export const ERR_ROOM_FULL = 'ROOM_FULL';
export const ERR_INVALID_MESSAGE = 'INVALID_MESSAGE';
export const ERR_INTERNAL = 'INTERNAL_ERROR';

export type TokenScope = 'admin' | 'write' | 'read';

export interface ClientJoin {
  type: 'join';
  protocol_version: number;
  message_id: string;
  workspace_id: string;
  peer_id: number;
  token: string;
}

export interface ClientDelta {
  type: 'delta';
  protocol_version: number;
  message_id: string;
  workspace_id: string;
  file_path: string;
  peer_id: number;
  sequence: number;
  data: string; // base64
}

export interface ClientSnapshot {
  type: 'snapshot';
  protocol_version: number;
  message_id: string;
  workspace_id: string;
  file_path: string;
  peer_id: number;
  data: string; // base64
}

export interface ClientLeave {
  type: 'leave';
  protocol_version: number;
  message_id: string;
  workspace_id: string;
}

export type ClientMessage = ClientJoin | ClientDelta | ClientSnapshot | ClientLeave;

export interface ServerWelcome {
  type: 'welcome';
  protocol_version: number;
  message_id: string;
  peers: number[];
  latest_snapshot: SnapshotInfo | null;
}

export interface ServerDelta {
  type: 'delta';
  protocol_version: number;
  message_id: string;
  from_peer: number;
  file_path: string;
  sequence: number;
  data: string; // base64
}

export interface ServerPeerJoin {
  type: 'peer_join';
  protocol_version: number;
  message_id: string;
  peer_id: number;
}

export interface ServerPeerLeave {
  type: 'peer_leave';
  protocol_version: number;
  message_id: string;
  peer_id: number;
}

export interface ServerAck {
  type: 'ack';
  protocol_version: number;
  message_id: string;
}

export interface ServerError {
  type: 'error';
  protocol_version: number;
  message_id: string;
  code: string;
  message: string;
}

export type ServerMessage =
  | ServerWelcome
  | ServerDelta
  | ServerPeerJoin
  | ServerPeerLeave
  | ServerAck
  | ServerError;

export interface SnapshotInfo {
  file_path: string;
  sequence: number;
  size_bytes: number;
}

export function validateClientMessage(msg: ClientMessage): string | null {
  // Check protocol version
  if (msg.protocol_version !== PROTOCOL_VERSION) {
    return ERR_UNSUPPORTED_PROTOCOL_VERSION;
  }

  // Check file_path length
  if ('file_path' in msg && msg.file_path && msg.file_path.length > MAX_FILE_PATH_LEN) {
    return ERR_PAYLOAD_TOO_LARGE;
  }

  // Check payload size
  if (msg.type === 'delta' && msg.data) {
    const estimated = Math.floor(msg.data.length * 0.75);
    if (estimated > MAX_DELTA_SIZE) {
      return ERR_PAYLOAD_TOO_LARGE;
    }
  }

  if (msg.type === 'snapshot' && msg.data) {
    const estimated = Math.floor(msg.data.length * 0.75);
    if (estimated > MAX_SNAPSHOT_SIZE) {
      return ERR_PAYLOAD_TOO_LARGE;
    }
  }

  return null;
}
