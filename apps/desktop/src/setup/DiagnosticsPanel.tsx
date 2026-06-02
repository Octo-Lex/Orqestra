import React, { useState } from 'react';
import { exportDiagnostics } from '../lib/diagnostics';
import type { DiagnosticBundleResult } from '../lib/diagnostics';
import { RecoveryCard } from './RecoveryCard';

interface Props {
  projectRoot?: string;
}

export const DiagnosticsPanel: React.FC<Props> = ({ projectRoot }) => {
  const [bundle, setBundle] = useState<DiagnosticBundleResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleExport = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await exportDiagnostics(projectRoot);
      setBundle(result);
    } catch (e) {
      setError(String(e));
    }
    setLoading(false);
  };

  const knownErrors = [
    'ROADMAP_NOT_FOUND',
    'AI_SERVICE_UNREACHABLE',
    'AI_KEY_MISSING',
    'GITHUB_TOKEN_MISSING',
    'KEYRING_UNAVAILABLE',
    'DASHBOARD_JSON_MISSING',
  ];

  return (
    <div style={styles.panel}>
      <h2 style={styles.title}>Diagnostics</h2>
      <p style={styles.subtitle}>
        Export diagnostic information for troubleshooting. All secrets are automatically redacted.
      </p>

      <button
        style={styles.exportBtn}
        onClick={handleExport}
        disabled={loading}
      >
        {loading ? 'Exporting...' : 'Export Diagnostics'}
      </button>

      {error && <p style={styles.error}>{error}</p>}

      {bundle && (
        <div style={styles.result}>
          <h3 style={styles.resultTitle}>Export Complete</h3>
          <div style={styles.resultRow}>
            <span>Path:</span>
            <span style={styles.mono}>{bundle.path}</span>
          </div>
          <div style={styles.resultRow}>
            <span>Files:</span>
            <span>{bundle.files.length}</span>
          </div>
          <div style={styles.resultRow}>
            <span>Secrets redacted:</span>
            <span>{bundle.redaction_summary.redacted_value_count}</span>
          </div>
          <div style={styles.resultRow}>
            <span>Contains raw secrets:</span>
            <span style={{ color: bundle.redaction_summary.contains_raw_secrets ? '#ef4444' : '#22c55e' }}>
              {bundle.redaction_summary.contains_raw_secrets ? 'YES' : 'NO'}
            </span>
          </div>
        </div>
      )}

      <div style={styles.recoverySection}>
        <h3 style={styles.sectionTitle}>Common Issues</h3>
        {knownErrors.map(code => (
          <RecoveryCard key={code} code={code} />
        ))}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: {
    padding: '16px',
    backgroundColor: '#1e293b',
    borderRadius: '8px',
    color: '#e2e8f0',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  },
  title: {
    fontSize: '18px',
    fontWeight: 600,
    margin: '0 0 4px 0',
    color: '#f1f5f9',
  },
  subtitle: {
    fontSize: '13px',
    color: '#94a3b8',
    margin: '0 0 16px 0',
  },
  exportBtn: {
    padding: '10px 20px',
    borderRadius: '8px',
    border: 'none',
    backgroundColor: '#6366f1',
    color: '#fff',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer',
    marginBottom: '16px',
  },
  error: {
    color: '#ef4444',
    fontSize: '13px',
  },
  result: {
    padding: '12px',
    backgroundColor: '#0f172a',
    borderRadius: '8px',
    marginBottom: '16px',
  },
  resultTitle: {
    fontSize: '14px',
    fontWeight: 600,
    color: '#22c55e',
    margin: '0 0 8px 0',
  },
  resultRow: {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '13px',
    padding: '2px 0',
  },
  mono: {
    fontFamily: 'monospace',
    fontSize: '12px',
    color: '#94a3b8',
  },
  recoverySection: {
    marginTop: '16px',
  },
  sectionTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#94a3b8',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    margin: '0 0 8px 0',
  },
};
