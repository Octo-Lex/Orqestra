/**
 * ReleaseHistoryPanel — Public evidence of release timeline.
 * Data sourced from docs/evidence/release-history.json at build time.
 */
import React from 'react';

const TYPE_COLORS: Record<string, { bg: string; text: string }> = {
  'feature': { bg: '#3b82f622', text: '#3b82f6' },
  'security': { bg: '#ef444422', text: '#ef4444' },
  'security-patch': { bg: '#ef444422', text: '#ef4444' },
  'observability': { bg: '#8b5cf622', text: '#8b5cf6' },
  'ergonomics': { bg: '#22c55e22', text: '#22c55e' },
  'structural-evidence': { bg: '#f59e0b22', text: '#f59e0b' },
};

import type { ReleaseHistoryEvidence } from '../lib/data';

type Props = {
  releaseHistory: ReleaseHistoryEvidence;
};

export const ReleaseHistoryPanel: React.FC<Props> = ({ releaseHistory }) => {
  if (!releaseHistory?.releases) {
    return <div style={styles.empty}>Release history unavailable in this export.</div>;
  }

  const releases = Object.entries(releaseHistory.releases)
    .reverse(); // newest first

  return (
    <div style={styles.panel}>
      <div style={styles.header}>
        <h3 style={styles.title}>Release History</h3>
        <span style={styles.provenance}>Source: docs/evidence/release-history.json</span>
      </div>
      <div style={styles.list}>
        {releases.map(([version, data]: [string, any]) => {
          const colors = TYPE_COLORS[data.type] || { bg: '#6b728022', text: '#6b7280' };
          return (
            <div key={version} style={styles.row}>
              <span style={styles.version}>v{version}</span>
              <span style={styles.label}>{data.label}</span>
              <span style={{ ...styles.badge, backgroundColor: colors.bg, color: colors.text }}>
                {data.type}
              </span>
              <span style={styles.date}>{data.date}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: { backgroundColor: '#0f172a', borderRadius: 8, padding: 16, marginBottom: 16 },
  header: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 },
  title: { margin: 0, fontSize: 16, fontWeight: 600, color: '#e2e8f0' },
  provenance: { fontSize: 11, color: '#64748b' },
  list: { display: 'flex', flexDirection: 'column', gap: 6 },
  row: { display: 'flex', alignItems: 'center', gap: 12, padding: '6px 8px', backgroundColor: '#1e293b', borderRadius: 6 },
  version: { fontFamily: 'monospace', fontSize: 13, fontWeight: 600, color: '#e2e8f0', minWidth: 60 },
  label: { fontSize: 13, color: '#94a3b8', flex: 1 },
  badge: { padding: '2px 8px', borderRadius: 4, fontSize: 11, fontWeight: 600 },
  date: { fontSize: 12, color: '#64748b', minWidth: 80, textAlign: 'right' },
  empty: { padding: 16, color: '#64748b', fontSize: 13, backgroundColor: '#0f172a', borderRadius: 8, marginBottom: 16 },
};
