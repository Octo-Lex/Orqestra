/**
 * TestCountTrendPanel — Public evidence of test count growth across releases.
 * Data sourced from docs/evidence/test-count-history.json at build time.
 */
import React from 'react';

import type { TestCountEvidence } from '../lib/data';

type Props = {
  testCounts: TestCountEvidence;
};

export const TestCountTrendPanel: React.FC<Props> = ({ testCounts }) => {
  if (!testCounts?.history) {
    return <div style={styles.empty}>Test count history unavailable in this export.</div>;
  }

  const history: TestCountEvidence['history'] = testCounts.history;
  const maxTotal = Math.max(...history.map((h) => h.total));

  return (
    <div style={styles.panel}>
      <div style={styles.header}>
        <h3 style={styles.title}>Test Count Trend</h3>
        <span style={styles.provenance}>Source: docs/evidence/test-count-history.json</span>
      </div>
      <div style={styles.chart}>
        {history.map((h) => {
          const barWidth = maxTotal > 0 ? (h.total / maxTotal) * 100 : 0;
          return (
            <div key={h.version} style={styles.barRow}>
              <span style={styles.barVersion}>v{h.version}</span>
              <div style={styles.barTrack}>
                <div style={{ ...styles.barFill, width: `${barWidth}%` }} />
                <span style={styles.barLabel}>{h.total} total ({h.rust} Rust + {h.worker} Worker{h.dashboard ? ` + ${h.dashboard} Dashboard` : ''})</span>
              </div>
            </div>
          );
        })}
      </div>
      <div style={styles.footer}>
        Curated release evidence — not auto-extracted from CI.
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: { backgroundColor: '#0f172a', borderRadius: 8, padding: 16, marginBottom: 16 },
  header: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 },
  title: { margin: 0, fontSize: 16, fontWeight: 600, color: '#e2e8f0' },
  provenance: { fontSize: 11, color: '#64748b' },
  chart: { display: 'flex', flexDirection: 'column', gap: 6 },
  barRow: { display: 'flex', alignItems: 'center', gap: 8 },
  barVersion: { fontFamily: 'monospace', fontSize: 12, color: '#94a3b8', minWidth: 55 },
  barTrack: { flex: 1, backgroundColor: '#1e293b', borderRadius: 4, height: 24, position: 'relative', overflow: 'hidden' },
  barFill: { position: 'absolute', top: 0, left: 0, bottom: 0, backgroundColor: '#3b82f644', borderRadius: 4 },
  barLabel: { position: 'relative', fontSize: 11, color: '#94a3b8', lineHeight: '24px', paddingLeft: 8, whiteSpace: 'nowrap' as const },
  footer: { marginTop: 8, fontSize: 11, color: '#64748b', fontStyle: 'italic' },
  empty: { padding: 16, color: '#64748b', fontSize: 13, backgroundColor: '#0f172a', borderRadius: 8, marginBottom: 16 },
};
