/**
 * AutonomyPolicyPanel — Public evidence of autonomy governance status.
 * Data sourced from docs/evidence/autonomy-policy.json at build time.
 */
import React from 'react';

import type { AutonomyPolicyEvidence } from '../lib/data';

type Props = {
  autonomyPolicy: AutonomyPolicyEvidence;
};

export const AutonomyPolicyPanel: React.FC<Props> = ({ autonomyPolicy }) => {
  if (!autonomyPolicy?.status) {
    return <div style={styles.empty}>Autonomy policy data unavailable in this export.</div>;
  }

  const rows = [
    { label: 'Status', value: autonomyPolicy.status },
    { label: 'Allowed Paths', value: autonomyPolicy.allowed_paths?.join(', ') },
    { label: 'Excluded Paths', value: autonomyPolicy.excluded_paths?.join(', ') },
    { label: 'Confidence Threshold (docs/)', value: autonomyPolicy.confidence_threshold_docs },
    { label: 'Confidence Threshold (README)', value: autonomyPolicy.confidence_threshold_readme },
    { label: 'Max Session Cap', value: autonomyPolicy.max_session_cap },
    { label: 'Default Cap', value: autonomyPolicy.default_cap },
    { label: 'Auto-Commit', value: autonomyPolicy.auto_commit === false ? 'Always False' : String(autonomyPolicy.auto_commit) },
    { label: 'Audit Schema Version', value: autonomyPolicy.audit_schema_version },
    { label: 'Rejection Reasons', value: autonomyPolicy.rejection_reasons },
  ];

  return (
    <div style={styles.panel}>
      <div style={styles.header}>
        <h3 style={styles.title}>Autonomy Policy</h3>
        <span style={styles.provenance}>Source: docs/evidence/autonomy-policy.json</span>
      </div>
      <table style={styles.table}>
        <tbody>
          {rows.map(({ label, value }) => (
            <tr key={label} style={styles.row}>
              <td style={styles.labelCell}>{label}</td>
              <td style={styles.valueCell}>{String(value)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {autonomyPolicy.disallowed_operations && (
        <div style={styles.footer}>
          Disallowed: {autonomyPolicy.disallowed_operations.join(' · ')}
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: { backgroundColor: '#0f172a', borderRadius: 8, padding: 16, marginBottom: 16 },
  header: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 },
  title: { margin: 0, fontSize: 16, fontWeight: 600, color: '#e2e8f0' },
  provenance: { fontSize: 11, color: '#64748b' },
  table: { width: '100%', borderCollapse: 'collapse' as const },
  row: { borderBottom: '1px solid #1e293b' },
  labelCell: { padding: '6px 8px', fontSize: 13, color: '#94a3b8', fontWeight: 600, width: '40%' },
  valueCell: { padding: '6px 8px', fontSize: 13, color: '#e2e8f0', fontFamily: 'monospace' },
  footer: { marginTop: 8, fontSize: 11, color: '#64748b', fontStyle: 'italic' },
  empty: { padding: 16, color: '#64748b', fontSize: 13, backgroundColor: '#0f172a', borderRadius: 8, marginBottom: 16 },
};
