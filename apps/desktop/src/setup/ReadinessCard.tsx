import React from 'react';

interface Props {
  id: string;
  label: string;
  status: 'ok' | 'warning' | 'missing' | 'error' | 'not_applicable';
  summary: string;
  details?: string;
  action?: {
    label: string;
    kind: string;
    payload?: string;
  };
}

export const ReadinessCard: React.FC<Props> = ({ label, status, summary, details }) => {
  const statusColor = () => {
    switch (status) {
      case 'ok': return '#22c55e';
      case 'warning': return '#f59e0b';
      case 'missing': return '#94a3b8';
      case 'error': return '#ef4444';
      case 'not_applicable': return '#64748b';
      default: return '#94a3b8';
    }
  };

  const statusIcon = () => {
    switch (status) {
      case 'ok': return '[OK]';
      case 'warning': return '[!!]';
      case 'error': return '[X]';
      default: return '[-]';
    }
  };

  return (
    <div style={styles.card}>
      <div style={styles.header}>
        <span style={{ ...styles.icon, color: statusColor() }}>{statusIcon()}</span>
        <span style={styles.label}>{label}</span>
      </div>
      <div style={styles.body}>
        <span style={{ ...styles.summary, color: statusColor() }}>{summary}</span>
        {details && <span style={styles.details}>{details}</span>}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  card: {
    padding: '8px 12px',
    backgroundColor: '#0f172a',
    borderRadius: '6px',
    marginBottom: '6px',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  icon: {
    fontSize: '12px',
    fontWeight: 700,
    fontFamily: 'monospace',
  },
  label: {
    fontSize: '13px',
    fontWeight: 500,
    color: '#e2e8f0',
  },
  body: {
    marginLeft: '24px',
    marginTop: '2px',
  },
  summary: {
    fontSize: '12px',
    fontWeight: 500,
  },
  details: {
    fontSize: '11px',
    color: '#94a3b8',
    marginLeft: '8px',
  },
};
