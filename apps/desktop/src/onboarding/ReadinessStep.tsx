import React, { useEffect, useState } from 'react';
import { getReadiness } from '../lib/readiness';
import type { ReadinessReport, ToolReadiness } from '../lib/readiness';

interface Props {
  projectRoot: string | null;
  onComplete: () => void;
  onBack: () => void;
}

export const ReadinessStep: React.FC<Props> = ({ projectRoot, onComplete, onBack }) => {
  const [report, setReport] = useState<ReadinessReport | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getReadiness(projectRoot || undefined).then(r => {
      setReport(r);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, [projectRoot]);

  const statusColor = (status: string) => {
    switch (status) {
      case 'ok': case 'found': case 'configured': case 'available': case 'real': return '#22c55e';
      case 'warning': case 'degraded_mock': case 'missing': return '#f59e0b';
      case 'error': case 'unreachable': case 'unavailable': return '#ef4444';
      default: return '#94a3b8';
    }
  };

  return (
    <div>
      <div style={styles.nav}>
        <button style={styles.backBtn} onClick={onBack}>&larr; Back</button>
        <h2 style={styles.heading}>Environment Readiness</h2>
      </div>

      {loading && <p style={styles.loading}>Checking environment...</p>}

      {report && (
        <>
          {/* App info */}
          <div style={styles.section}>
            <h3 style={styles.sectionTitle}>Application</h3>
            <div style={styles.row}>
              <span>Version</span>
              <span>{report.app.version}</span>
            </div>
            <div style={styles.row}>
              <span>Platform</span>
              <span>{report.app.platform}</span>
            </div>
          </div>

          {/* Tools */}
          <div style={styles.section}>
            <h3 style={styles.sectionTitle}>Local Tools</h3>
            {report.local_tools.map((t: ToolReadiness) => (
              <div key={t.tool} style={styles.row}>
                <span>{t.tool}</span>
                <span style={{ color: statusColor(t.status) }}>
                  {t.status} {t.version ? `(${t.version})` : ''}
                </span>
              </div>
            ))}
          </div>

          {/* AI */}
          <div style={styles.section}>
            <h3 style={styles.sectionTitle}>AI Service</h3>
            <div style={styles.row}>
              <span>Service</span>
              <span style={{ color: statusColor(report.ai.service_status) }}>
                {report.ai.service_status}
              </span>
            </div>
            <div style={styles.row}>
              <span>API Key</span>
              <span style={{ color: statusColor(report.ai.api_key_status) }}>
                {report.ai.api_key_status}
              </span>
            </div>
            <div style={styles.row}>
              <span>Mode</span>
              <span style={{ color: statusColor(report.ai.mode) }}>
                {report.ai.mode === 'real' ? 'Real AI enabled' :
                 report.ai.mode === 'degraded_mock' ? 'Degraded (mock)' : 'Unavailable'}
              </span>
            </div>
            {report.ai.last_error && (
              <p style={styles.errorNote}>{report.ai.last_error}</p>
            )}
          </div>

          {/* Credentials */}
          <div style={styles.section}>
            <h3 style={styles.sectionTitle}>Credentials</h3>
            <div style={styles.row}>
              <span>GitHub Token</span>
              <span style={{ color: statusColor(report.credentials.github_token) }}>
                {report.credentials.github_token}
              </span>
            </div>
            <div style={styles.row}>
              <span>Provider</span>
              <span>{report.credentials.provider}</span>
            </div>
          </div>

          {/* Dashboard */}
          <div style={styles.section}>
            <h3 style={styles.sectionTitle}>Dashboard</h3>
            <div style={styles.row}>
              <span>Local JSON</span>
              <span style={{ color: statusColor(report.dashboard.local_json) }}>
                {report.dashboard.local_json}
              </span>
            </div>
            <div style={styles.row}>
              <span>Live URL</span>
              <span style={{ color: statusColor(report.dashboard.live_url_status) }}>
                {report.dashboard.live_url_status}
              </span>
            </div>
          </div>

          {/* Warnings */}
          {report.warnings.length > 0 && (
            <div style={styles.section}>
              <h3 style={styles.sectionTitle}>Warnings</h3>
              {report.warnings.map((w, i) => (
                <div key={i} style={styles.warningCard}>
                  <span style={{ color: statusColor(w.severity) }}>{w.severity}</span>: {w.message}
                  <p style={styles.recovery}>{w.recovery}</p>
                </div>
              ))}
            </div>
          )}

          <button style={styles.completeBtn} onClick={onComplete}>
            Open Workspace
          </button>
        </>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  nav: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    marginBottom: '16px',
  },
  backBtn: {
    background: 'none',
    border: 'none',
    color: '#94a3b8',
    cursor: 'pointer',
    fontSize: '14px',
    padding: '4px 8px',
  },
  heading: {
    fontSize: '18px',
    fontWeight: 600,
    margin: 0,
    color: '#f1f5f9',
  },
  loading: {
    color: '#94a3b8',
    fontSize: '13px',
    textAlign: 'center' as const,
  },
  section: {
    marginBottom: '16px',
  },
  sectionTitle: {
    fontSize: '13px',
    fontWeight: 600,
    color: '#94a3b8',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    margin: '0 0 8px 0',
  },
  row: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '4px 0',
    fontSize: '13px',
    color: '#e2e8f0',
  },
  errorNote: {
    fontSize: '12px',
    color: '#f59e0b',
    margin: '4px 0 0 0',
  },
  warningCard: {
    fontSize: '13px',
    color: '#e2e8f0',
    padding: '8px',
    backgroundColor: '#1e1e2e',
    borderRadius: '6px',
    marginBottom: '6px',
  },
  recovery: {
    fontSize: '12px',
    color: '#94a3b8',
    margin: '4px 0 0 0',
  },
  completeBtn: {
    width: '100%',
    padding: '12px',
    borderRadius: '8px',
    border: 'none',
    backgroundColor: '#6366f1',
    color: '#fff',
    fontSize: '14px',
    fontWeight: 600,
    cursor: 'pointer',
    marginTop: '16px',
  },
};
