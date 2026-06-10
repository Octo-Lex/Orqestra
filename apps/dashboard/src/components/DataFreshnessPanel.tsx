/**
 * DataFreshnessPanel — Shows when data was exported and from what source.
 * Distinguishes static export from live data.
 */
import React from 'react';

type Props = {
  generatedAt: string;
  source: { repo: string; branch: string; commit: string };
  evidenceGeneratedFrom?: { source: string; commit: string; generated_at: string };
};

export const DataFreshnessPanel: React.FC<Props> = ({ generatedAt, source, evidenceGeneratedFrom }) => {
  return (
    <div style={styles.panel}>
      <div style={styles.header}>
        <h3 style={styles.title}>Data Freshness</h3>
        <span style={styles.staticBadge}>Static Export</span>
      </div>
      <div style={styles.rows}>
        <div style={styles.row}>
          <span style={styles.label}>Roadmap generated</span>
          <span style={styles.value}>{new Date(generatedAt).toLocaleString()}</span>
        </div>
        <div style={styles.row}>
          <span style={styles.label}>Source commit</span>
          <span style={styles.mono}>{source.commit}</span>
        </div>
        <div style={styles.row}>
          <span style={styles.label}>Branch</span>
          <span style={styles.mono}>{source.branch}</span>
        </div>
        {evidenceGeneratedFrom && (
          <>
            <div style={styles.row}>
              <span style={styles.label}>Evidence commit</span>
              <span style={styles.mono}>{evidenceGeneratedFrom.commit.slice(0, 12)}</span>
            </div>
            <div style={styles.row}>
              <span style={styles.label}>Evidence source</span>
              <span style={styles.value}>{evidenceGeneratedFrom.source}</span>
            </div>
          </>
        )}
      </div>
      <div style={styles.notice}>
        This data reflects the repository state at export time, not real-time status.
        Evidence is derived from repo artifacts and curated release metadata.
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: { backgroundColor: '#0f172a', borderRadius: 8, padding: 16, marginBottom: 16 },
  header: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 },
  title: { margin: 0, fontSize: 16, fontWeight: 600, color: '#e2e8f0' },
  staticBadge: { padding: '2px 10px', borderRadius: 4, backgroundColor: '#3b82f622', color: '#3b82f6', fontSize: 11, fontWeight: 600 },
  rows: { display: 'flex', flexDirection: 'column', gap: 4 },
  row: { display: 'flex', justifyContent: 'space-between', padding: '4px 8px', backgroundColor: '#1e293b', borderRadius: 4 },
  label: { fontSize: 13, color: '#94a3b8' },
  value: { fontSize: 13, color: '#e2e8f0' },
  mono: { fontSize: 13, color: '#e2e8f0', fontFamily: 'monospace' },
  notice: { marginTop: 12, fontSize: 12, color: '#64748b', fontStyle: 'italic', padding: '8px 12px', backgroundColor: '#1e293b', borderRadius: 6 },
};
