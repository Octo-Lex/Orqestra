import { useState, useEffect } from 'react';

interface ChangedFile {
  path: string;
  status: string;
  staged: boolean;
  file_kind: string;
  risk: string;
  risk_reason?: string;
}

interface GitStatusPanelProps {
  projectRoot: string;
}

export function GitStatusPanel({ projectRoot }: GitStatusPanelProps) {
  const [snapshot, setSnapshot] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await (window as any).__TAURI_INTERNALS__.invoke(
        'git_repository_snapshot_cmd',
        { projectRoot }
      );
      setSnapshot(JSON.parse(result));
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { refresh(); }, [projectRoot]);

  if (loading) return <div className="git-status-loading">Loading git status…</div>;
  if (error) return <div className="git-status-error">Error: {error}</div>;
  if (!snapshot) return null;

  const riskBadge = (risk: string) => {
    const colors: Record<string, string> = {
      secret: '#dc2626',
      workflow: '#d97706',
      binary: '#6b7280',
      large: '#6b7280',
      normal: '#22c55e',
      unknown: '#9ca3af',
    };
    return (
      <span
        style={{
          backgroundColor: colors[risk] || '#9ca3af',
          color: 'white',
          padding: '1px 6px',
          borderRadius: '4px',
          fontSize: '11px',
          marginLeft: '6px',
        }}
      >
        {risk}
      </span>
    );
  };

  return (
    <div className="git-status-panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
        <h3 style={{ margin: 0 }}>Git Status</h3>
        <button onClick={refresh} style={{ fontSize: '12px', padding: '4px 10px' }}>
          Refresh
        </button>
      </div>

      {/* Status summary */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px', marginBottom: '12px' }}>
        <div>
          <strong>Branch:</strong> {snapshot.branch}
        </div>
        <div>
          <strong>Dirty:</strong>{' '}
          <span style={{ color: snapshot.dirty ? '#dc2626' : '#22c55e' }}>
            {snapshot.dirty ? 'Yes' : 'Clean'}
          </span>
        </div>
        <div>
          <strong>Provider:</strong> {snapshot.provider}
        </div>
        <div>
          <strong>Latency:</strong> {snapshot.latency_ms}ms
        </div>
      </div>

      {/* HEAD metadata */}
      {snapshot.head && (
        <div style={{ marginBottom: '12px', padding: '8px', backgroundColor: '#f8fafc', borderRadius: '4px' }}>
          <strong>HEAD:</strong>{' '}
          <code>{snapshot.head.short_sha}</code>{' '}
          <span style={{ color: '#6b7280' }}>{snapshot.head.message}</span>
          {snapshot.head.detached && (
            <span style={{ color: '#d97706', marginLeft: '8px', fontSize: '12px' }}>
              ⚠ Detached HEAD
            </span>
          )}
        </div>
      )}

      {/* Counts */}
      <div style={{ display: 'flex', gap: '16px', marginBottom: '12px', fontSize: '13px' }}>
        <span>Staged: <strong>{snapshot.staged_count}</strong></span>
        <span>Unstaged: <strong>{snapshot.unstaged_count}</strong></span>
        <span>Untracked: <strong>{snapshot.untracked_count}</strong></span>
      </div>

      {/* Changed files */}
      {snapshot.changed_files.length > 0 && (
        <div>
          <h4 style={{ margin: '8px 0 4px' }}>Changed Files</h4>
          <table style={{ width: '100%', fontSize: '12px', borderCollapse: 'collapse' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #e5e7eb' }}>
                <th style={{ textAlign: 'left', padding: '4px' }}>Path</th>
                <th style={{ textAlign: 'left', padding: '4px' }}>Status</th>
                <th style={{ textAlign: 'left', padding: '4px' }}>Risk</th>
              </tr>
            </thead>
            <tbody>
              {snapshot.changed_files.map((f: ChangedFile, i: number) => (
                <tr key={i} style={{ borderBottom: '1px solid #f1f5f9' }}>
                  <td style={{ padding: '3px 4px' }}>
                    <code>{f.path}</code>
                  </td>
                  <td style={{ padding: '3px 4px' }}>
                    {f.status}{f.staged ? ' (staged)' : ''}
                  </td>
                  <td style={{ padding: '3px 4px' }}>
                    {riskBadge(f.risk)}
                    {f.risk_reason && (
                      <span style={{ color: '#9ca3af', fontSize: '10px', marginLeft: '4px' }}>
                        {f.risk_reason}
                      </span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Diagnostics */}
      {snapshot.fallback_used && (
        <div style={{ marginTop: '8px', color: '#d97706', fontSize: '12px' }}>
          ⚠ Using CLI fallback for some operations
        </div>
      )}
      {snapshot.diagnostics.length > 0 && (
        <div style={{ marginTop: '4px', color: '#6b7280', fontSize: '11px' }}>
          {snapshot.diagnostics.map((d: string, i: number) => (
            <div key={i}>{d}</div>
          ))}
        </div>
      )}
    </div>
  );
}
