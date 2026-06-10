/**
 * RuntimeEvidencePanel — Public evidence of runtime decision matrix results.
 * Data sourced from docs/evidence/runtime-decision-matrix.json at build time.
 * This is STRUCTURAL evidence — it exercises the policy engine, not external beta data.
 */
import React from 'react';

type Props = {
  runtimeEvidence: any;
};

export const RuntimeEvidencePanel: React.FC<Props> = ({ runtimeEvidence }) => {
  if (!runtimeEvidence?.path_matrix_evaluated) {
    return <div style={styles.empty}>Runtime evidence data unavailable in this export.</div>;
  }

  return (
    <div style={styles.panel}>
      <div style={styles.header}>
        <h3 style={styles.title}>Runtime Evidence</h3>
        <span style={styles.provenance}>Source: docs/evidence/runtime-decision-matrix.json</span>
      </div>

      <div style={styles.typeBanner}>
        <span style={styles.typeBadge}>{runtimeEvidence.evidence_type || 'structural'}</span>
        {runtimeEvidence.external_beta_user_data === false && (
          <span style={styles.externalBadge}>Not external beta data</span>
        )}
      </div>

      <div style={styles.stats}>
        <div style={styles.stat}>
          <div style={styles.statValue}>{runtimeEvidence.path_matrix_evaluated}</div>
          <div style={styles.statLabel}>Paths Evaluated</div>
        </div>
        <div style={styles.stat}>
          <div style={{ ...styles.statValue, color: '#22c55e' }}>{runtimeEvidence.paths_allowed}</div>
          <div style={styles.statLabel}>Allowed</div>
        </div>
        <div style={styles.stat}>
          <div style={{ ...styles.statValue, color: '#ef4444' }}>{runtimeEvidence.paths_rejected}</div>
          <div style={styles.statLabel}>Rejected</div>
        </div>
        <div style={styles.stat}>
          <div style={styles.statValue}>{runtimeEvidence.rejection_rate}</div>
          <div style={styles.statLabel}>Rejection Rate</div>
        </div>
        <div style={styles.stat}>
          <div style={{ ...styles.statValue, color: '#22c55e' }}>
            {runtimeEvidence.safety_invariants_passing}/{runtimeEvidence.safety_invariants_total}
          </div>
          <div style={styles.statLabel}>Safety Invariants</div>
        </div>
      </div>

      {runtimeEvidence.disclaimer && (
        <div style={styles.disclaimer}>{runtimeEvidence.disclaimer}</div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: { backgroundColor: '#0f172a', borderRadius: 8, padding: 16, marginBottom: 16 },
  header: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 },
  title: { margin: 0, fontSize: 16, fontWeight: 600, color: '#e2e8f0' },
  provenance: { fontSize: 11, color: '#64748b' },
  typeBanner: { display: 'flex', gap: 8, marginBottom: 12 },
  typeBadge: { padding: '2px 10px', borderRadius: 4, backgroundColor: '#8b5cf622', color: '#8b5cf6', fontSize: 11, fontWeight: 600 },
  externalBadge: { padding: '2px 10px', borderRadius: 4, backgroundColor: '#f59e0b22', color: '#f59e0b', fontSize: 11, fontWeight: 600 },
  stats: { display: 'flex', gap: 16, flexWrap: 'wrap' as const },
  stat: { backgroundColor: '#1e293b', borderRadius: 8, padding: '10px 16px', minWidth: 100 },
  statValue: { fontSize: 24, fontWeight: 700, color: '#e2e8f0' },
  statLabel: { fontSize: 11, color: '#64748b', marginTop: 2 },
  disclaimer: { marginTop: 12, fontSize: 12, color: '#94a3b8', fontStyle: 'italic', padding: '8px 12px', backgroundColor: '#1e293b', borderRadius: 6 },
  empty: { padding: 16, color: '#64748b', fontSize: 13, backgroundColor: '#0f172a', borderRadius: 8, marginBottom: 16 },
};
