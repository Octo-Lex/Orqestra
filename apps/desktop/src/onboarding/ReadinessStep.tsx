import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getReadiness } from '../lib/readiness';
import type { ReadinessReport, ToolReadiness } from '../lib/readiness';

interface Props {
  projectRoot: string | null;
  onComplete: () => void;
  onBack: () => void;
}

export const ReadinessStep: React.FC<Props> = ({ projectRoot, onComplete, onBack }) => {
  const [report, setReport] = useState<ReadinessReport | null>(null);
  const [betaReadiness, setBetaReadiness] = useState<{
    readiness: string;
    blocking: boolean;
    checks: Record<string, boolean | null>;
    repo: { detected: boolean; branch: string; dirty: boolean; remote_configured: boolean };
    warnings: string[];
    blocked_features: string[];
  } | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getReadiness(projectRoot || undefined).then(r => {
      setReport(r);
      setLoading(false);
    }).catch(() => setLoading(false));

    // v2.11.0: Fetch beta readiness summary
    invoke('get_beta_readiness_cmd', { projectRoot: projectRoot || null })
      .then((data) => setBetaReadiness(data as typeof betaReadiness))
      .catch(() => { /* beta readiness is optional */ });
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

          {/* v2.11.0: Beta Readiness Summary */}
          {betaReadiness && (
            <div style={styles.section}>
              <h3 style={styles.sectionTitle}>Beta Readiness</h3>
              <div style={{
                ...styles.readinessBadge,
                backgroundColor: betaReadiness.readiness === 'ready' ? '#166534' :
                  betaReadiness.blocking ? '#7f1d1d' : '#92400e',
              }}>
                {betaReadiness.readiness === 'ready' ? '✓ Ready' :
                 betaReadiness.blocking ? '✗ Blocked' : '⚠ Ready with Warnings'}
              </div>

              {/* Checks */}
              <div style={{ marginTop: '8px' }}>
                {Object.entries(betaReadiness.checks).map(([key, value]) => (
                  <div key={key} style={styles.row}>
                    <span>{key.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())}</span>
                    <span style={{ color: value === true ? '#22c55e' : value === false ? '#ef4444' : '#94a3b8' }}>
                      {value === true ? '✓' : value === false ? '✗' : '? unknown'}
                    </span>
                  </div>
                ))}
              </div>

              {/* Repo details */}
              {betaReadiness.repo.detected && (
                <div style={{ marginTop: '8px' }}>
                  <div style={styles.row}>
                    <span>Branch</span>
                    <span style={{ color: '#e2e8f0' }}>{betaReadiness.repo.branch}</span>
                  </div>
                  <div style={styles.row}>
                    <span>Working Tree</span>
                    <span style={{ color: betaReadiness.repo.dirty ? '#f59e0b' : '#22c55e' }}>
                      {betaReadiness.repo.dirty ? 'Dirty (uncommitted changes)' : 'Clean'}
                    </span>
                  </div>
                  <div style={styles.row}>
                    <span>Remote</span>
                    <span style={{ color: betaReadiness.repo.remote_configured ? '#22c55e' : '#f59e0b' }}>
                      {betaReadiness.repo.remote_configured ? 'Configured' : 'Not configured'}
                    </span>
                  </div>
                </div>
              )}

              {/* Blocked features */}
              {betaReadiness.blocked_features.length > 0 && (
                <div style={{ marginTop: '8px' }}>
                  <div style={styles.sectionTitle}>Blocked Features</div>
                  {betaReadiness.blocked_features.map((f, i) => (
                    <div key={i} style={{ ...styles.row, color: '#f59e0b' }}>
                      ⚠ {f.replace(/_/g, ' ')}
                    </div>
                  ))}
                </div>
              )}

              {/* AI degraded guidance */}
              {!betaReadiness.checks.ai_service_reachable && (
                <div style={styles.degradedGuidance}>
                  AI service unavailable. Project management, roadmap views, Git history, and dashboard export remain available. Agent execution requires the local AI service (localhost:8000) to be running.
                </div>
              )}
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
  readinessBadge: {
    display: 'inline-block',
    padding: '6px 12px',
    borderRadius: '6px',
    fontSize: '13px',
    fontWeight: 600,
    color: '#fff',
    marginBottom: '8px',
  },
  degradedGuidance: {
    fontSize: '12px',
    color: '#94a3b8',
    backgroundColor: '#1e1e2e',
    padding: '8px',
    borderRadius: '6px',
    marginTop: '8px',
    lineHeight: '1.4',
  },
};
