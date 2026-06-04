
interface AgentDiagnosticsProps {
  agentMode: 'docs' | 'bugfix';
  allowedPaths?: string[];
  maxFilesChanged?: number;
}

export function AgentDiagnosticsPanel({ agentMode, allowedPaths, maxFilesChanged }: AgentDiagnosticsProps) {
  const label = agentMode === 'docs' ? 'Docs Agent' : 'Bugfix Agent';
  const endpoint = agentMode === 'docs' ? '/agent/docs' : '/agent/bugfix';

  return (
    <div style={{ padding: '12px', fontSize: '12px' }}>
      <div style={{ fontWeight: 600, marginBottom: '12px' }}>{label} Diagnostics</div>

      {/* Safety invariants */}
      <div style={{
        padding: '6px 8px',
        backgroundColor: '#f0fdf4',
        border: '1px solid #86efac',
        borderRadius: '4px',
        marginBottom: '12px',
        lineHeight: '1.6',
      }}>
        <div style={{ fontWeight: 500, color: '#166534', marginBottom: '4px' }}>Safety Invariants</div>
        <div style={{ fontSize: '11px', color: '#15803d' }}>
          ✓ Review-only mode<br />
          ✓ Auto-commit: false<br />
          ✓ Auto-apply: false<br />
          ✓ No staging<br />
          ✓ No repository writes<br />
          ✓ No push/pull
        </div>
      </div>

      {/* Agent config */}
      <div style={{ marginBottom: '12px' }}>
        <div style={{ fontWeight: 500, marginBottom: '4px' }}>Configuration</div>
        <table style={{ fontSize: '11px', width: '100%' }}>
          <tbody>
            <tr>
              <td style={{ color: '#6b7280', paddingRight: '12px' }}>Endpoint</td>
              <td style={{ fontFamily: 'monospace' }}>{endpoint}</td>
            </tr>
            <tr>
              <td style={{ color: '#6b7280', paddingRight: '12px' }}>Review only</td>
              <td style={{ color: '#15803d' }}>true</td>
            </tr>
            <tr>
              <td style={{ color: '#6b7280', paddingRight: '12px' }}>Auto commit</td>
              <td style={{ color: '#dc2626' }}>false</td>
            </tr>
            <tr>
              <td style={{ color: '#6b7280', paddingRight: '12px' }}>Auto apply</td>
              <td style={{ color: '#dc2626' }}>false</td>
            </tr>
            {maxFilesChanged !== undefined && (
              <tr>
                <td style={{ color: '#6b7280', paddingRight: '12px' }}>Max files changed</td>
                <td>{maxFilesChanged}</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {/* Allowed paths */}
      {allowedPaths && allowedPaths.length > 0 && (
        <div style={{ marginBottom: '12px' }}>
          <div style={{ fontWeight: 500, marginBottom: '4px' }}>Allowed Paths</div>
          {allowedPaths.map((p, i) => (
            <div key={i} style={{ fontSize: '11px', fontFamily: 'monospace', padding: '1px 0' }}>
              {p}
            </div>
          ))}
        </div>
      )}

      {/* Context schema */}
      <div style={{ marginBottom: '12px' }}>
        <div style={{ fontWeight: 500, marginBottom: '4px' }}>Context Schema</div>
        <span style={{ padding: '2px 6px', backgroundColor: '#dbeafe', borderRadius: '4px', fontSize: '10px', color: '#1e40af' }}>
          agent-context-v2
        </span>
      </div>

      {/* Content policy summary */}
      <div>
        <div style={{ fontWeight: 500, marginBottom: '4px' }}>Content Policy</div>
        <div style={{ fontSize: '10px', color: '#6b7280', lineHeight: '1.6' }}>
          File contents: excluded<br />
          Raw diffs: excluded<br />
          Secret contents: excluded<br />
          Binary contents: excluded<br />
          Large file contents: excluded<br />
          Symlink targets: excluded
        </div>
      </div>
    </div>
  );
}
