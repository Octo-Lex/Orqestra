import { useState, useEffect } from 'react';

interface AgentContextV2 {
  schema_version: string;
  branch: string;
  head_short_sha: string;
  dirty: boolean;
  changed_files: Array<{
    path: string;
    status: string;
    staged: boolean;
    file_kind: string;
    risk: string;
    risk_reason: string | null;
    original_path: string | null;
  }>;
  risk_summary: {
    normal_count: number;
    secret_count: number;
    workflow_count: number;
    binary_count: number;
    large_count: number;
    unknown_count: number;
  };
  diff_stat: {
    files_changed: number;
    insertions: number;
    deletions: number;
  };
  commit_groups: Array<{
    scope: string;
    change_type: string;
    file_count: number;
    risk: string;
    suggested_title: string;
  }>;
  semantic_proposal: {
    title: string;
    scope: string;
    change_type: string;
    risk_level: string;
    confidence: number;
  };
  recent_commit_subjects: string[];
  provider: string;
  content_policy: {
    git_context_file_contents: boolean;
    diff_body_included: boolean;
    secret_contents_excluded: boolean;
    binary_contents_excluded: boolean;
    large_contents_excluded: boolean;
    symlink_contents_excluded: boolean;
  };
}

export function AgentContextPanel({ projectRoot }: { projectRoot: string }) {
  const [context, setContext] = useState<AgentContextV2 | null>(null);
  const [status, setStatus] = useState<'loading' | 'available' | 'unavailable'>('loading');
  const [buildTime, setBuildTime] = useState<string>('');

  const refresh = async () => {
    setStatus('loading');
    const start = performance.now();
    try {
      const result = await (window as any).__TAURI__.invoke('build_agent_context_preview', { projectRoot });
      setContext(result);
      setStatus('available');
      const elapsed = ((performance.now() - start) / 1000).toFixed(2);
      setBuildTime(elapsed + 's');
    } catch {
      setContext(null);
      setStatus('unavailable');
      setBuildTime('');
    }
  };

  useEffect(() => { refresh(); }, [projectRoot]);

  if (status === 'loading') {
    return <div style={{ padding: '12px', color: '#6b7280', fontSize: '12px' }}>Loading context...</div>;
  }

  if (status === 'unavailable' || !context) {
    return (
      <div style={{ padding: '12px', fontSize: '12px' }}>
        <div style={{ color: '#ef4444', fontWeight: 600, marginBottom: '4px' }}>Context Unavailable</div>
        <div style={{ color: '#6b7280' }}>Git context could not be built. Agents will run without repository metadata.</div>
        <button onClick={refresh} style={{ marginTop: '8px', fontSize: '11px', padding: '4px 8px', cursor: 'pointer' }}>Retry</button>
      </div>
    );
  }

  const riskColors: Record<string, string> = {
    normal: '#10b981',
    secret: '#ef4444',
    workflow: '#f59e0b',
    binary: '#6366f1',
    large: '#8b5cf6',
    unknown: '#6b7280',
  };

  return (
    <div style={{ padding: '12px', fontSize: '12px' }}>
      {/* Header */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
        <span style={{ fontWeight: 600 }}>Agent Context</span>
        <span style={{ fontSize: '10px', color: '#6b7280' }}>{buildTime}</span>
      </div>

      {/* Schema + Provider */}
      <div style={{ display: 'flex', gap: '8px', marginBottom: '8px' }}>
        <span style={{ padding: '2px 6px', backgroundColor: '#dbeafe', borderRadius: '4px', fontSize: '10px', color: '#1e40af' }}>
          {context.schema_version}
        </span>
        <span style={{ padding: '2px 6px', backgroundColor: '#f3f4f6', borderRadius: '4px', fontSize: '10px', color: '#374151' }}>
          {context.provider}
        </span>
      </div>

      {/* Branch + HEAD */}
      <div style={{ marginBottom: '8px', color: '#374151' }}>
        <strong>{context.branch}</strong> @ <code style={{ fontSize: '11px' }}>{context.head_short_sha}</code>
        {context.dirty && <span style={{ color: '#f59e0b', marginLeft: '4px' }}>(dirty)</span>}
      </div>

      {/* Changed files */}
      <div style={{ marginBottom: '8px' }}>
        <div style={{ fontWeight: 500, marginBottom: '4px' }}>Changed Files ({context.changed_files.length})</div>
        {context.changed_files.slice(0, 10).map((f: any, i: number) => (
          <div key={i} style={{ display: 'flex', alignItems: 'center', gap: '6px', padding: '2px 0', fontFamily: 'monospace', fontSize: '11px' }}>
            <span style={{ color: riskColors[f.risk] || '#6b7280', fontSize: '8px' }}>●</span>
            <span>{f.path}</span>
            <span style={{ color: '#9ca3af', fontSize: '10px' }}>{f.status}</span>
            {f.risk !== 'normal' && <span style={{ fontSize: '9px', color: riskColors[f.risk], backgroundColor: '#fef2f2', padding: '1px 4px', borderRadius: '2px' }}>{f.risk}</span>}
          </div>
        ))}
        {context.changed_files.length > 10 && (
          <div style={{ color: '#9ca3af', fontSize: '10px', marginTop: '2px' }}>
            +{context.changed_files.length - 10} more files
          </div>
        )}
      </div>

      {/* Risk summary */}
      <div style={{ marginBottom: '8px' }}>
        <div style={{ fontWeight: 500, marginBottom: '4px' }}>Risk Summary</div>
        <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
          {Object.entries(context.risk_summary).map(([key, count]) => {
            const label = key.replace('_count', '');
            if (Number(count) === 0) return null;
            return (
              <span key={key} style={{ fontSize: '11px' }}>
                <span style={{ color: riskColors[label] || '#6b7280' }}>●</span> {label}: {Number(count)}
              </span>
            );
          })}
        </div>
      </div>

      {/* Diff stat */}
      <div style={{ marginBottom: '8px', color: '#374151' }}>
        <span style={{ fontWeight: 500 }}>Diff:</span>{' '}
        {context.diff_stat.files_changed} files, +{context.diff_stat.insertions}/-{context.diff_stat.deletions}
      </div>

      {/* Commit groups */}
      {context.commit_groups.length > 0 && (
        <div style={{ marginBottom: '8px' }}>
          <div style={{ fontWeight: 500, marginBottom: '4px' }}>Commit Groups ({context.commit_groups.length})</div>
          {context.commit_groups.map((g: any, i: number) => (
            <div key={i} style={{ padding: '4px 0', borderBottom: '1px solid #f3f4f6' }}>
              <div style={{ fontSize: '11px', fontFamily: 'monospace' }}>{g.suggested_title}</div>
              <div style={{ fontSize: '10px', color: '#6b7280' }}>{g.scope} · {g.change_type} · {g.risk}</div>
            </div>
          ))}
        </div>
      )}

      {/* Semantic proposal */}
      <div style={{ marginBottom: '8px' }}>
        <div style={{ fontWeight: 500, marginBottom: '4px' }}>Proposal</div>
        <div style={{ fontSize: '11px', fontFamily: 'monospace' }}>{context.semantic_proposal.title}</div>
        <div style={{ fontSize: '10px', color: '#6b7280' }}>
          {context.semantic_proposal.scope} · {context.semantic_proposal.change_type} · confidence {(context.semantic_proposal.confidence * 100).toFixed(0)}%
        </div>
      </div>

      {/* Content policy */}
      <div style={{ marginBottom: '8px' }}>
        <div style={{ fontWeight: 500, marginBottom: '4px' }}>Content Policy</div>
        <div style={{ fontSize: '10px', color: '#6b7280', lineHeight: '1.6' }}>
          {!context.content_policy.git_context_file_contents && '✓'} No file contents
          {!context.content_policy.diff_body_included && ' · ✓'} No diffs
          {context.content_policy.secret_contents_excluded && ' · ✓'} Secrets excluded
          {context.content_policy.binary_contents_excluded && ' · ✓'} Binary excluded
          {context.content_policy.large_contents_excluded && ' · ✓'} Large excluded
          {context.content_policy.symlink_contents_excluded && ' · ✓'} Symlinks excluded
        </div>
      </div>

      <button onClick={refresh} style={{ fontSize: '11px', padding: '4px 8px', cursor: 'pointer' }}>Refresh</button>
    </div>
  );
}
