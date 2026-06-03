import { useState, useEffect } from 'react';

interface CommitSummary {
  sha: string;
  short_sha: string;
  message: string;
  author_name: string;
  author_email: string;
  timestamp: string;
  parents: string[];
  provider: string;
}

interface CommitSummaryPanelProps {
  projectRoot: string;
  limit?: number;
}

export function CommitSummaryPanel({ projectRoot, limit = 10 }: CommitSummaryPanelProps) {
  const [commits, setCommits] = useState<CommitSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await (window as any).__TAURI_INTERNALS__.invoke(
        'git_recent_commits_cmd',
        { projectRoot, limit }
      );
      setCommits(JSON.parse(result));
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { refresh(); }, [projectRoot, limit]);

  if (loading) return <div style={{ fontSize: '12px', color: '#6b7280' }}>Loading commits…</div>;
  if (error) return <div style={{ fontSize: '12px', color: '#dc2626' }}>Error: {error}</div>;
  if (commits.length === 0) return <div style={{ fontSize: '12px', color: '#6b7280' }}>No commits</div>;

  return (
    <div className="commit-summary-panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
        <h4 style={{ margin: 0 }}>Recent Commits</h4>
        <button onClick={refresh} style={{ fontSize: '11px', padding: '2px 8px' }}>
          Refresh
        </button>
      </div>

      <div style={{ fontSize: '12px' }}>
        {commits.map((commit, i) => (
          <div
            key={commit.sha}
            style={{
              padding: '6px 8px',
              borderBottom: i < commits.length - 1 ? '1px solid #f1f5f9' : 'none',
              display: 'flex',
              gap: '8px',
              alignItems: 'baseline',
            }}
          >
            <code style={{ color: '#6b7280', flexShrink: 0 }}>{commit.short_sha}</code>
            <span style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {commit.message}
            </span>
            <span style={{ color: '#9ca3af', fontSize: '10px', flexShrink: 0 }}>
              {commit.author_name}
            </span>
          </div>
        ))}
      </div>

      <div style={{ marginTop: '6px', color: '#9ca3af', fontSize: '10px' }}>
        Provider: {commits[0]?.provider || 'unknown'}
      </div>
    </div>
  );
}
