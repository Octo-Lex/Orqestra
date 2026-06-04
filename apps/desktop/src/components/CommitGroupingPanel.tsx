interface CommitGroup {
  scope: string;
  change_type: string;
  files: string[];
  risk: string;
  suggested_title: string;
  suggested_body: string;
  requires_manual_review: boolean;
}

interface CommitGroupingPanelProps {
  groups: CommitGroup[];
}

export function CommitGroupingPanel({ groups }: CommitGroupingPanelProps) {
  if (!groups || groups.length === 0) {
    return <div style={{ fontSize: '12px', color: '#6b7280' }}>No commit groups</div>;
  }

  const riskBadge = (risk: string) => {
    const colors: Record<string, string> = {
      normal: '#22c55e',
      secret: '#dc2626',
      workflow: '#d97706',
      unknown: '#9ca3af',
    };
    return (
      <span
        style={{
          backgroundColor: colors[risk] || '#9ca3af',
          color: 'white',
          padding: '1px 6px',
          borderRadius: '4px',
          fontSize: '10px',
          marginLeft: '6px',
        }}
      >
        {risk}
      </span>
    );
  };

  return (
    <div className="commit-grouping-panel">
      <h4 style={{ margin: '0 0 8px' }}>Suggested Commit Groups</h4>
      <div style={{ fontSize: '11px', color: '#6b7280', marginBottom: '8px' }}>
        These are suggestions only. Review each group before committing.
      </div>
      {groups.map((group, i) => (
        <div
          key={i}
          style={{
            padding: '8px',
            marginBottom: '8px',
            border: '1px solid #e2e8f0',
            borderRadius: '4px',
            backgroundColor: group.risk !== 'normal' ? '#fef2f2' : '#ffffff',
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
            <div>
              <strong style={{ fontSize: '12px' }}>{group.suggested_title}</strong>
              {riskBadge(group.risk)}
            </div>
            <span style={{ fontSize: '10px', color: '#9ca3af' }}>
              {group.scope}({group.change_type})
            </span>
          </div>

          <div style={{ fontSize: '11px', color: '#6b7280', marginBottom: '4px' }}>
            {group.files.length} file(s):
          </div>
          <ul style={{ margin: '0', paddingLeft: '16px', fontSize: '11px' }}>
            {group.files.map((file, j) => (
              <li key={j} style={{ fontFamily: 'monospace' }}>{file}</li>
            ))}
          </ul>

          {group.requires_manual_review && (
            <div style={{ marginTop: '4px', fontSize: '10px', color: '#92400e' }}>
              ⚠ Manual review required
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
