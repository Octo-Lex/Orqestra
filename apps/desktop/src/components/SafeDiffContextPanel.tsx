import { useState, useEffect } from 'react';

interface SafeDiffContextPanelProps {
  projectRoot: string;
}

export function SafeDiffContextPanel({ projectRoot }: SafeDiffContextPanelProps) {
  const [context, setContext] = useState<any>(null);
  const [status, setStatus] = useState<'loading' | 'available' | 'unavailable'>('loading');

  const refresh = async () => {
    setStatus('loading');
    try {
      const result = await (window as any).__TAURI__.invoke('build_agent_context_preview', { projectRoot });
      setContext(result?.safe_diff_context || null);
      setStatus('available');
    } catch {
      setStatus('unavailable');
    }
  };

  useEffect(() => { refresh(); }, [projectRoot]);

  if (status === 'loading') return <div style={{ padding: '12px', color: '#6b7280', fontSize: '12px' }}>Loading safe diff context...</div>;
  if (status === 'unavailable' || !context) {
    return (
      <div style={{ padding: '12px', fontSize: '12px' }}>
        <div style={{ color: '#ef4444', fontWeight: 600, marginBottom: '4px' }}>Context Unavailable</div>
        <div style={{ color: '#6b7280' }}>Safe diff context could not be loaded.</div>
      </div>
    );
  }

  return (
    <div style={{ padding: '12px', fontSize: '12px' }}>
      <div style={{ fontWeight: 600, marginBottom: '12px' }}>Safe Diff Context</div>

      {/* Status */}
      <div style={{ display: 'flex', gap: '8px', marginBottom: '12px', alignItems: 'center' }}>
        <span style={{
          padding: '2px 6px',
          borderRadius: '4px',
          fontSize: '10px',
          backgroundColor: context.enabled ? '#dcfce7' : '#f3f4f6',
          color: context.enabled ? '#166534' : '#6b7280',
        }}>
          {context.enabled ? 'Enabled' : 'Disabled'}
        </span>
        <span style={{ fontSize: '10px', color: '#9ca3af' }}>
          Source: {context.enabled_source}
        </span>
      </div>

      {/* Toggle info */}
      <div style={{ marginBottom: '12px', fontSize: '10px', color: '#6b7280' }}>
        Runtime toggle: <code>{context.policy?.runtime_toggle || 'ORQESTRA_SAFE_DIFF_CONTEXT'}</code>
      </div>

      {/* Summary */}
      {context.summary && (
        <div style={{ marginBottom: '12px' }}>
          <div style={{ fontWeight: 500, marginBottom: '4px' }}>Summary</div>
          <div style={{ display: 'flex', gap: '12px', fontSize: '11px', color: '#374151' }}>
            <span>Included: {context.summary.included_files}</span>
            <span>Excluded: {context.summary.excluded_files}</span>
            <span>Lines: {context.summary.total_lines}</span>
            {context.summary.truncated && <span style={{ color: '#f59e0b' }}>Truncated</span>}
          </div>
        </div>
      )}

      {/* Policy caps */}
      {context.policy && (
        <div style={{ marginBottom: '12px' }}>
          <div style={{ fontWeight: 500, marginBottom: '4px' }}>Policy Caps</div>
          <div style={{ fontSize: '10px', color: '#6b7280', lineHeight: '1.6' }}>
            Max files: {context.policy.max_files}<br />
            Max file size: {(context.policy.max_file_size_bytes / 1024).toFixed(0)} KiB<br />
            Max lines/hunk: {context.policy.max_lines_per_hunk}<br />
            Max lines/file: {context.policy.max_lines_per_file}<br />
            Max total lines: {context.policy.max_total_lines}
          </div>
        </div>
      )}

      {/* Files */}
      {context.files && context.files.length > 0 && (
        <div style={{ marginBottom: '12px' }}>
          <div style={{ fontWeight: 500, marginBottom: '4px' }}>Files</div>
          {context.files.map((f: any, i: number) => (
            <div key={i} style={{
              padding: '4px 6px',
              marginBottom: '2px',
              backgroundColor: f.included ? '#f0fdf4' : '#fef2f2',
              borderRadius: '3px',
              fontSize: '11px',
              fontFamily: 'monospace',
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span>{f.path}</span>
                <span style={{ color: f.included ? '#15803d' : '#dc2626' }}>
                  {f.included ? 'included' : f.exclusion_reason || 'excluded'}
                </span>
              </div>
              {f.included && f.hunks && f.hunks.length > 0 && (
                <div style={{ marginTop: '2px', fontSize: '10px', color: '#6b7280' }}>
                  {f.line_count} lines in {f.hunks.length} hunk(s)
                  {f.original_path && <span> (renamed from {f.original_path})</span>}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Safety */}
      <div style={{
        padding: '6px 8px',
        backgroundColor: '#f0fdf4',
        border: '1px solid #86efac',
        borderRadius: '4px',
        fontSize: '10px',
        color: '#15803d',
      }}>
        Agent mode: review-only · Auto-commit: false · Auto-apply: false
      </div>

      <button onClick={refresh} style={{ marginTop: '8px', fontSize: '11px', padding: '4px 8px', cursor: 'pointer' }}>Refresh</button>
    </div>
  );
}
