/**
 * SecurityBoundaryPanel — Public evidence of security posture.
 * Data sourced from docs/evidence/security-boundaries.json at build time.
 */
import React from 'react';

type Props = {
  securityBoundaries: any;
};

export const SecurityBoundaryPanel: React.FC<Props> = ({ securityBoundaries }) => {
  if (!securityBoundaries?.boundaries) {
    return <div style={styles.empty}>Security boundary data unavailable in this export.</div>;
  }

  const boundaries = securityBoundaries.boundaries;
  const entries = [
    { key: 'relay_auth', label: 'Relay Authentication', detail: boundaries.relay_auth },
    { key: 'content_security_policy', label: 'Content Security Policy', detail: boundaries.content_security_policy },
    { key: 'patch_checksum', label: 'Patch Checksums', detail: boundaries.patch_checksum },
    { key: 'credential_storage', label: 'Credential Storage', detail: boundaries.credential_storage },
    { key: 'token_format', label: 'Token Format', detail: boundaries.token_format },
    { key: 'master_secret', label: 'Master Secret', detail: boundaries.master_secret },
    { key: 'dashboard_authority', label: 'Dashboard Authority', detail: boundaries.dashboard_authority },
  ];

  return (
    <div style={styles.panel}>
      <div style={styles.header}>
        <h3 style={styles.title}>Security Boundaries</h3>
        <span style={styles.provenance}>Source: docs/evidence/security-boundaries.json</span>
      </div>
      <div style={styles.grid}>
        {entries.map(({ key, label, detail }) => (
          <div key={key} style={styles.card}>
            <div style={styles.cardLabel}>{label}</div>
            <div style={styles.cardStatus}>{detail?.algorithm || detail?.status || detail?.location || detail?.version || '—'}</div>
            {detail?.description && <div style={styles.cardDesc}>{detail.description}</div>}
          </div>
        ))}
      </div>
      <div style={styles.footer}>
        {securityBoundaries.provenance || 'Security boundary evidence derived from release records.'}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: { backgroundColor: '#0f172a', borderRadius: 8, padding: 16, marginBottom: 16 },
  header: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 },
  title: { margin: 0, fontSize: 16, fontWeight: 600, color: '#e2e8f0' },
  provenance: { fontSize: 11, color: '#64748b' },
  grid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: 8 },
  card: { backgroundColor: '#1e293b', borderRadius: 6, padding: '10px 12px' },
  cardLabel: { fontSize: 12, fontWeight: 600, color: '#94a3b8', marginBottom: 4 },
  cardStatus: { fontSize: 14, fontWeight: 600, color: '#22c55e' },
  cardDesc: { fontSize: 11, color: '#64748b', marginTop: 4 },
  footer: { marginTop: 12, fontSize: 11, color: '#64748b', fontStyle: 'italic' },
  empty: { padding: 16, color: '#64748b', fontSize: 13, backgroundColor: '#0f172a', borderRadius: 8, marginBottom: 16 },
};
