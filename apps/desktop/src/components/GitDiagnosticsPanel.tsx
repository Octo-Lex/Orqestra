interface GitDiagnosticsPanelProps {
  snapshot: {
    provider: string;
    fallback_used: boolean;
    parity_status: string;
    latency_ms: number;
    diagnostics: string[];
  } | null;
}

export function GitDiagnosticsPanel({ snapshot }: GitDiagnosticsPanelProps) {
  if (!snapshot) return null;

  const providerLabel = (provider: string) => {
    const colors: Record<string, string> = {
      'gix': '#22c55e',
      'gix-hybrid': '#3b82f6',
      'git-cli-fallback': '#d97706',
      'git-cli': '#d97706',
    };
    return (
      <span style={{ color: colors[provider] || '#6b7280', fontWeight: 600 }}>
        {provider}
      </span>
    );
  };

  const parityLabel = (parity: string) => {
    const colors: Record<string, string> = {
      'match': '#22c55e',
      'verified-core-cases': '#3b82f6',
      'mismatch': '#dc2626',
      'fallback': '#d97706',
      'not-tested': '#9ca3af',
    };
    return (
      <span style={{ color: colors[parity] || '#9ca3af' }}>
        {parity}
      </span>
    );
  };

  return (
    <div className="git-diagnostics-panel" style={{ fontSize: '12px', padding: '8px', backgroundColor: '#f8fafc', borderRadius: '4px' }}>
      <h4 style={{ margin: '0 0 8px' }}>Git Diagnostics</h4>
      <div style={{ display: 'grid', gridTemplateColumns: '140px auto', gap: '4px 12px' }}>
        <span style={{ color: '#6b7280' }}>Provider:</span>
        <span>{providerLabel(snapshot.provider)}</span>

        <span style={{ color: '#6b7280' }}>Fallback used:</span>
        <span>{snapshot.fallback_used ? 'Yes' : 'No'}</span>

        <span style={{ color: '#6b7280' }}>Parity status:</span>
        <span>{parityLabel(snapshot.parity_status)}</span>

        <span style={{ color: '#6b7280' }}>Latency:</span>
        <span>{snapshot.latency_ms}ms</span>
      </div>

      {snapshot.diagnostics.length > 0 && (
        <div style={{ marginTop: '8px', padding: '4px', backgroundColor: '#fffbeb', borderRadius: '4px' }}>
          {snapshot.diagnostics.map((d: string, i: number) => (
            <div key={i} style={{ color: '#92400e' }}>⚠ {d}</div>
          ))}
        </div>
      )}
    </div>
  );
}
