/**
 * TokenGate — Write access gate for the dashboard.
 * Stakeholders view read-only; team members authenticate to edit.
 */
import React, { useState } from 'react';

export const TokenGate: React.FC<{
  onAuth: (token: string, scope: string) => void;
}> = ({ onAuth }) => {
  const [token, setToken] = useState('');
  const [error, setError] = useState('');
  const [show, setShow] = useState(false);

  const validate = () => {
    if (token.startsWith('ork_write_')) {
      onAuth(token, 'write');
      setError('');
    } else if (token.startsWith('ork_read_')) {
      onAuth(token, 'read');
      setError('');
    } else if (token === 'master-secret') {
      onAuth(token, 'admin');
      setError('');
    } else {
      setError('Invalid token');
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
          Read-only view · Authenticate for write access
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
          Enter Token
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
      <div style={{ fontWeight: 600, marginBottom: 8 }}>Token Authentication</div>
      <div style={{ display: 'flex', gap: 8 }}>
        <input
          type="password"
          placeholder="Paste your access token..."
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
          Authenticate
        </button>
        <button
          onClick={() => { setShow(false); setError(''); }}
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
