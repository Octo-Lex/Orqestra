import { useState } from 'react';

interface GitOperationProvider {
  operation: string;
  provider: string;
  native: boolean;
  fallback_available: boolean;
  read_only: boolean;
  mutates_repository: boolean;
  executed_in_diagnostics: boolean;
  latency_ms: number | null;
}

interface GitProviderReport {
  operations: GitOperationProvider[];
  snapshot_time: string;
  repository_valid: boolean;
}

interface GitProviderDiagnosticsPanelProps {
  projectRoot: string | null;
}

export function GitProviderDiagnosticsPanel({ projectRoot }: GitProviderDiagnosticsPanelProps) {
  const [report, setReport] = useState<GitProviderReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchReport = async () => {
    if (!projectRoot) return;
    setLoading(true);
    setError(null);
    try {
      const result = await (window as any).__TAURI_INTERNALS__?.invoke?.('git_provider_diagnostics_cmd', { projectRoot });
      if (typeof result === 'string') {
        setReport(JSON.parse(result));
      } else {
        setReport(result);
      }
    } catch (e: any) {
      setError(e?.toString() || 'Failed to fetch provider report');
    } finally {
      setLoading(false);
    }
  };

  const providerBadge = (provider: string) => {
    const colors: Record<string, { bg: string; text: string }> = {
      'gix': { bg: '#dcfce7', text: '#166534' },
      'gix-hybrid': { bg: '#dbeafe', text: '#1e40af' },
      'git-cli-fallback': { bg: '#fef3c7', text: '#92400e' },
      'deterministic-heuristic': { bg: '#f3e8ff', text: '#6b21a8' },
      'not-implemented': { bg: '#f3f4f6', text: '#6b7280' },
    };
    const style = colors[provider] || { bg: '#f3f4f6', text: '#6b7280' };
    return (
      <span style={{
        backgroundColor: style.bg,
        color: style.text,
        padding: '2px 8px',
        borderRadius: '10px',
        fontSize: '11px',
        fontWeight: 600,
        whiteSpace: 'nowrap',
      }}>
        {provider}
      </span>
    );
  };

  const readOnlyLabel = (readOnly: boolean) => (
    <span style={{ color: readOnly ? '#22c55e' : '#ef4444', fontSize: '11px' }}>
      {readOnly ? 'read-only' : 'mutating'}
    </span>
  );

  const executedLabel = (executed: boolean) => (
    <span style={{ color: executed ? '#3b82f6' : '#9ca3af', fontSize: '11px' }}>
      {executed ? 'measured' : 'registered'}
    </span>
  );

  return (
    <div style={{ fontSize: '12px', padding: '12px', backgroundColor: '#f8fafc', borderRadius: '6px', border: '1px solid #e2e8f0' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
        <h4 style={{ margin: 0 }}>Git Provider Diagnostics</h4>
        <button
          onClick={fetchReport}
          disabled={loading || !projectRoot}
          style={{
            padding: '4px 12px',
            fontSize: '11px',
            backgroundColor: loading ? '#94a3b8' : '#3b82f6',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: loading ? 'not-allowed' : 'pointer',
          }}
        >
          {loading ? 'Running...' : 'Run Diagnostics'}
        </button>
      </div>

      {error && (
        <div style={{ padding: '6px', backgroundColor: '#fef2f2', borderRadius: '4px', color: '#dc2626', marginBottom: '8px' }}>
          {error}
        </div>
      )}

      {report && (
        <>
          <div style={{ display: 'flex', gap: '16px', marginBottom: '8px', fontSize: '10px', color: '#6b7280' }}>
            <span>Valid repo: {report.repository_valid ? '✓' : '✗'}</span>
            <span>{report.snapshot_time}</span>
          </div>

          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #e2e8f0' }}>
                <th style={{ textAlign: 'left', padding: '4px 6px', fontSize: '10px', color: '#6b7280' }}>Operation</th>
                <th style={{ textAlign: 'left', padding: '4px 6px', fontSize: '10px', color: '#6b7280' }}>Provider</th>
                <th style={{ textAlign: 'center', padding: '4px 6px', fontSize: '10px', color: '#6b7280' }}>Mode</th>
                <th style={{ textAlign: 'center', padding: '4px 6px', fontSize: '10px', color: '#6b7280' }}>Native</th>
                <th style={{ textAlign: 'center', padding: '4px 6px', fontSize: '10px', color: '#6b7280' }}>Diag</th>
                <th style={{ textAlign: 'right', padding: '4px 6px', fontSize: '10px', color: '#6b7280' }}>Latency</th>
              </tr>
            </thead>
            <tbody>
              {report.operations.map((op) => (
                <tr key={op.operation} style={{ borderBottom: '1px solid #f1f5f9' }}>
                  <td style={{ padding: '4px 6px', fontFamily: 'monospace', fontSize: '11px' }}>{op.operation}</td>
                  <td style={{ padding: '4px 6px' }}>{providerBadge(op.provider)}</td>
                  <td style={{ padding: '4px 6px', textAlign: 'center' }}>{readOnlyLabel(op.read_only)}</td>
                  <td style={{ padding: '4px 6px', textAlign: 'center', color: op.native ? '#22c55e' : '#6b7280', fontSize: '11px' }}>
                    {op.native ? '✓' : '—'}
                  </td>
                  <td style={{ padding: '4px 6px', textAlign: 'center' }}>{executedLabel(op.executed_in_diagnostics)}</td>
                  <td style={{ padding: '4px 6px', textAlign: 'right', fontFamily: 'monospace', fontSize: '11px', color: '#6b7280' }}>
                    {op.latency_ms !== null ? `${op.latency_ms}ms` : '—'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          <div style={{ marginTop: '8px', fontSize: '10px', color: '#6b7280' }}>
            {(() => {
              const native = report.operations.filter(o => o.native).length;
              const hybrid = report.operations.filter(o => o.provider === 'gix-hybrid').length;
              const cli = report.operations.filter(o => o.provider === 'git-cli-fallback').length;
              const notImpl = report.operations.filter(o => o.provider === 'not-implemented').length;
              return `Summary: ${native} native | ${hybrid} hybrid | ${cli} CLI-only | ${notImpl} not implemented`;
            })()}
          </div>
        </>
      )}

      {!report && !loading && !error && (
        <div style={{ color: '#9ca3af', fontSize: '11px', textAlign: 'center', padding: '12px' }}>
          Click "Run Diagnostics" to inspect Git provider selection per operation.
        </div>
      )}
    </div>
  );
}
