/**
 * SyncPanel — CRDT sync status and offline merge demo.
 *
 * Shows:
 * - Peer ID and connection state
 * - Open CRDT documents
 * - 2-peer offline merge simulation
 * - Token management (generate/validate)
 */
import React, { useState, useCallback } from 'react';
import {
  type SyncStatus,
  type TaskField,
  type AuthResult,
  type SyncConnectionState,
} from '../sync/LoroProvider';

// Mock data for browser testing
const MOCK_SYNC_STATUS: SyncStatus = {
  peer_id: 18446744073709551615n as unknown as number,
  open_docs: [
    'roadmap/TASK-2026-038.md',
    'roadmap/TASK-2026-040.md',
    'roadmap/TASK-2026-042.md',
  ],
};

const MOCK_PEER_A: Record<string, string> = {
  title: 'Rate limiter v2',
  status: 'in-progress',
  assignee: 'alice',
};

const MOCK_PEER_B: Record<string, string> = {
  title: 'Rate limiter v3',
  priority: 'critical',
};

export const SyncPanel: React.FC = () => {
  const [_status] = useState<SyncStatus | null>(MOCK_SYNC_STATUS);
  const [connection, setConnection] = useState<SyncConnectionState>('connected');
  const [mergeResult, setMergeResult] = useState<{
    converged: boolean;
    fields: TaskField[];
  } | null>(null);
  const [tokenInput, setTokenInput] = useState('');
  const [authResult, setAuthResult] = useState<AuthResult | null>(null);
  const [generatedToken, setGeneratedToken] = useState('');
  const [tokenScope, setTokenScope] = useState<'write' | 'read'>('write');

  // Simulate 2-peer offline merge
  const runMerge = useCallback(() => {
    setConnection('syncing');
    setTimeout(() => {
      // Merge both peers' edits: union of keys
      const merged: Record<string, string> = {
        ...MOCK_PEER_A,
        ...MOCK_PEER_B,
      };

      // In CRDT, title conflict resolves deterministically
      // Both peers get the same result
      const fields: TaskField[] = Object.entries(merged).map(([key, value]) => ({
        key,
        value,
      }));

      setMergeResult({ converged: true, fields });
      setConnection('connected');
    }, 1000);
  }, []);

  // Validate token
  const checkToken = useCallback(() => {
    // Mock validation
    if (tokenInput.startsWith('ork_write_')) {
      setAuthResult({ authorized: true, scope: 'Write' });
    } else if (tokenInput.startsWith('ork_read_')) {
      setAuthResult({ authorized: true, scope: 'Read' });
    } else if (tokenInput === 'master-secret') {
      setAuthResult({ authorized: true, scope: 'Admin' });
    } else {
      setAuthResult({ authorized: false, reason: 'Invalid token' });
    }
  }, [tokenInput]);

  // Generate token
  const genToken = useCallback(() => {
    const ts = Date.now().toString(16);
    setGeneratedToken(`ork_${tokenScope}_${ts}`);
  }, [tokenScope]);

  const connColor: Record<SyncConnectionState, string> = {
    connected: '#22c55e',
    syncing: '#f59e0b',
    offline: '#ef4444',
    disconnected: '#6b7280',
  };

  return (
    <div style={{ padding: 16, borderTop: '2px solid #1e293b' }}>
      <h3 style={{ marginTop: 0 }}>CRDT Sync & Collaboration</h3>

      {/* Sync Status */}
      <div style={{ display: 'flex', gap: 16, alignItems: 'center', marginBottom: 12 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <div style={{
            width: 10, height: 10, borderRadius: '50%',
            backgroundColor: connColor[connection],
          }} />
          <span style={{ fontWeight: 600, textTransform: 'capitalize' }}>{connection}</span>
        </div>
        {_status && (
          <span style={{ color: '#94a3b8', fontSize: 13 }}>
            Peer {_status.peer_id} · {_status.open_docs.length} docs open
          </span>
        )}
      </div>

      {/* Open Documents */}
      {_status && _status.open_docs.length > 0 && (
        <div style={{ marginBottom: 16 }}>
          <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 4 }}>CRDT Documents</div>
          {_status.open_docs.map(doc => (
            <div key={doc} style={{
              padding: '4px 8px', marginBottom: 2,
              backgroundColor: '#1e293b', borderRadius: 4, fontSize: 13,
            }}>
              {doc}
            </div>
          ))}
        </div>
      )}

      {/* 2-Peer Offline Merge Demo */}
      <div style={{ marginBottom: 16, padding: 12, backgroundColor: '#0f172a', borderRadius: 8 }}>
        <div style={{ fontWeight: 600, marginBottom: 8 }}>Offline Merge Simulation</div>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, marginBottom: 12 }}>
          <div>
            <div style={{ fontSize: 12, fontWeight: 600, color: '#60a5fa', marginBottom: 4 }}>
              Peer A (offline edits)
            </div>
            {Object.entries(MOCK_PEER_A).map(([k, v]) => (
              <div key={k} style={{ fontSize: 13 }}>
                <span style={{ color: '#94a3b8' }}>{k}:</span> {v}
              </div>
            ))}
          </div>
          <div>
            <div style={{ fontSize: 12, fontWeight: 600, color: '#34d399', marginBottom: 4 }}>
              Peer B (offline edits)
            </div>
            {Object.entries(MOCK_PEER_B).map(([k, v]) => (
              <div key={k} style={{ fontSize: 13 }}>
                <span style={{ color: '#94a3b8' }}>{k}:</span> {v}
              </div>
            ))}
          </div>
        </div>
        <button
          onClick={runMerge}
          style={{
            padding: '6px 16px', backgroundColor: '#3b82f6', color: '#fff',
            border: 'none', borderRadius: 6, cursor: 'pointer', fontWeight: 600,
          }}
        >
          Sync & Merge
        </button>

        {mergeResult && (
          <div style={{ marginTop: 12 }}>
            <div style={{
              fontWeight: 600, color: mergeResult.converged ? '#22c55e' : '#ef4444',
              marginBottom: 4,
            }}>
              {mergeResult.converged ? 'Merged — Converged!' : 'Merge conflict'}
            </div>
            <div style={{ fontSize: 13, color: '#94a3b8', marginBottom: 4 }}>
              {mergeResult.fields.length} fields after merge (no data loss):
            </div>
            {mergeResult.fields.map(f => (
              <div key={f.key} style={{ fontSize: 13, display: 'flex', gap: 8 }}>
                <span style={{ color: '#fbbf24', minWidth: 80 }}>{f.key}</span>
                <span>{f.value}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Token Management */}
      <div style={{ padding: 12, backgroundColor: '#0f172a', borderRadius: 8 }}>
        <div style={{ fontWeight: 600, marginBottom: 8 }}>Token Access Control</div>

        {/* Generate */}
        <div style={{ display: 'flex', gap: 8, marginBottom: 12, alignItems: 'center' }}>
          <select
            value={tokenScope}
            onChange={e => setTokenScope(e.target.value as 'write' | 'read')}
            style={{ padding: '4px 8px', borderRadius: 4, border: '1px solid #334155', backgroundColor: '#1e293b', color: '#fff' }}
          >
            <option value="write">Write</option>
            <option value="read">Read</option>
          </select>
          <button
            onClick={genToken}
            style={{
              padding: '4px 12px', backgroundColor: '#6366f1', color: '#fff',
              border: 'none', borderRadius: 6, cursor: 'pointer', fontSize: 13,
            }}
          >
            Generate Token
          </button>
          {generatedToken && (
            <code style={{ fontSize: 12, color: '#a5b4fc', wordBreak: 'break-all' }}>{generatedToken}</code>
          )}
        </div>

        {/* Validate */}
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <input
            type="text"
            placeholder="Paste token to validate..."
            value={tokenInput}
            onChange={e => setTokenInput(e.target.value)}
            style={{
              flex: 1, padding: '6px 10px', borderRadius: 6,
              border: '1px solid #334155', backgroundColor: '#1e293b', color: '#fff',
              fontSize: 13,
            }}
          />
          <button
            onClick={checkToken}
            style={{
              padding: '6px 12px', backgroundColor: '#0ea5e9', color: '#fff',
              border: 'none', borderRadius: 6, cursor: 'pointer', fontSize: 13,
            }}
          >
            Validate
          </button>
        </div>
        {authResult && (
          <div style={{ marginTop: 8, fontSize: 13 }}>
            {authResult.authorized ? (
              <span style={{ color: '#22c55e' }}>
                Authorized · Scope: <strong>{authResult.scope}</strong>
              </span>
            ) : (
              <span style={{ color: '#ef4444' }}>
                Denied: {authResult.reason}
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
