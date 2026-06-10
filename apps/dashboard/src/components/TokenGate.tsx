/**
 * TokenGate — Private view connector for the dashboard.
 *
 * The dashboard is a read-only evidence surface.
 * Tokens unlock private metadata views only — no write access, no admin scope.
 * Accepted tokens: ork_read_* only.
 * Rejected: ork_write_*, admin tokens, all other formats.
 * Tokens are never persisted, logged, or included in errors/URLs.
 */
import React, { useState } from 'react';

export const TokenGate: React.FC<{
  onAuth: (token: string, scope: string) => void;
}> = ({ onAuth }) => {
  const [token, setToken] = useState('');
  const [error, setError] = useState('');
  const [show, setShow] = useState(false);

  const validate = () => {
    if (token.startsWith('ork_read_')) {
      onAuth(token, 'private');
      setError('');
    } else {
      setError('Invalid token. Use a read token for private dashboard view.');
    }
  };

  if (!show) {
    return (
      <div style={{
        padding: '12px 16px',
        backgroundColor: '#1e293b',
        borderRadius: 8,
        marginBottom: 16,
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
      }}>
        <span style={{ fontSize: 13, color: '#94a3b8' }}>
          Read-only · Connect for private metadata
        </span>
        <button
          onClick={() => setShow(true)}
          style={{
            padding: '4px 12px',
            backgroundColor: '#3b82f6',
            color: '#fff',
            border: 'none',
            borderRadius: 6,
            cursor: 'pointer',
            fontSize: 13,
          }}
        >
          Connect Private View
        </button>
      </div>
    );
  }

  return (
    <div style={{
      padding: 16,
      backgroundColor: '#1e293b',
      borderRadius: 8,
      marginBottom: 16,
    }}>
      <div style={{ fontWeight: 600, marginBottom: 8 }}>Connect Private View</div>
      <div style={{ display: 'flex', gap: 8 }}>
        <input
          type="password"
          placeholder="Paste your read token..."
          value={token}
          onChange={e => { setToken(e.target.value); setError(''); }}
          onKeyDown={e => e.key === 'Enter' && validate()}
          style={{
            flex: 1, padding: '8px 12px', borderRadius: 6,
            border: '1px solid #334155', backgroundColor: '#0f172a', color: '#fff',
            fontSize: 13,
          }}
        />
        <button
          onClick={validate}
          style={{
            padding: '8px 16px', backgroundColor: '#3b82f6', color: '#fff',
            border: 'none', borderRadius: 6, cursor: 'pointer', fontWeight: 600,
          }}
        >
          Connect
        </button>
        <button
          onClick={() => { setShow(false); setError(''); setToken(''); }}
          style={{
            padding: '8px 12px', backgroundColor: '#334155', color: '#94a3b8',
            border: 'none', borderRadius: 6, cursor: 'pointer',
          }}
        >
          Cancel
        </button>
      </div>
      {error && <div style={{ marginTop: 4, color: '#ef4444', fontSize: 13 }}>{error}</div>}
    </div>
  );
};
