import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getReadiness } from '../lib/readiness';
import type { ReadinessReport, ToolReadiness } from '../lib/readiness';
import { BetaEvidenceExportPanel } from '../components/BetaEvidenceExportPanel';

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
    // v2.14.11: Both calls are async on the Rust side now (spawn_blocking).
    // They won't freeze the UI even if they take several seconds.
    getReadiness(projectRoot || undefined).then(r => {
      setReport(r);
      setLoading(false);
    }).catch(() => setLoading(false));

    invoke('get_beta_readiness_cmd', { projectRoot: projectRoot || null })
      .then((data) => setBetaReadiness(data as typeof betaReadiness))
      .catch(() => { /* beta readiness is optional */ });
  }, [projectRoot]);

  // v2.14.11: Determine actionable state for the user
  const aiAvailable = betaReadiness?.checks?.ai_service_reachable === true ||
    (report?.ai.service_status === 'reachable');
  const repoSelected = !!projectRoot;

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

      {/* v2.14.11: Guided Next Steps Panel */}
      <div style={styles.guidedPanel}>
        <h3 style={styles.guidedTitle}>📋 Next Steps</h3>
        <ol style={styles.guidedList}>
          <li style={repoSelected ? styles.stepDone : styles.stepCurrent}>
            {repoSelected ? '✅' : '👉'} <strong>Open a repository</strong>
            {repoSelected
              ? ` — ${projectRoot}`
              : ' — Click "Open Workspace" below to select a Git repository'}
          </li>
          <li style={styles.stepPending}>
            ⬜ <strong>Review environment</strong> — Check the status items below
          </li>
          <li style={aiAvailable ? styles.stepDone : styles.stepOptional}>
            {aiAvailable ? '✅' : '⚙️'} <strong>AI agents</strong>
            {aiAvailable
              ? ' — AI service is available. Try the docs or bugfix agent.'
              : ' — Optional. AI service is not configured. You can still use project management, Git, and evidence export without it.'}
          </li>
          <li style={styles.stepPending}>
            ⬜ <strong>Export beta evidence</strong> — When you're done testing, export a bundle below
          </li>
        </ol>
      </div>

      {/* Actionable status banner */}
      {betaReadiness && (
        <div style={{
          ...styles.statusBanner,
          backgroundColor: betaReadiness.readiness === 'ready' ? '#14532d' :
            '#78350f',
        }}>
          {betaReadiness.readiness === 'ready'
            ? '✅ Ready — all core features available'
            : '⚠️ Ready with warnings — some features are limited (see below)'}
          {!aiAvailable && (
            <span style={styles.statusNote}>
              {' — '}Agent features need AI service. Everything else works.
            </span>
          )}
        </div>
      )}

      {loading && <p style={styles.loading}>Checking environment... (this won't freeze the app)</p>}

      {/* Collapsible details */}
      {report && (
        <details style={styles.detailsSection}>
          <summary style={styles.detailsSummary}>Technical Details</summary>

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
            <h3 style={styles.sectionTitle}>AI Service {!aiAvailable && <span style={styles.optionalBadge}>optional</span>}</h3>
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
            {report.ai.last_error && (
              <p style={styles.errorNote}>{report.ai.last_error}</p>
            )}
            {!aiAvailable && (
              <p style={styles.hintNote}>
                AI service is optional. To enable agents, start the Python AI service
                and configure an API key. Project management and evidence export work without it.
              </p>
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
        </details>
      )}

      {/* Beta readiness warnings (non-blocking) */}
      {betaReadiness && betaReadiness.warnings.length > 0 && (
        <div style={styles.warningsPanel}>
          <h3 style={styles.sectionTitle}>Notices</h3>
          {betaReadiness.warnings.map((w, i) => (
            <div key={i} style={styles.noticeRow}>• {w}</div>
          ))}
        </div>
      )}

      {/* v2.12.0: Beta Evidence Export (with feedback capture) */}
      <BetaEvidenceExportPanel projectRoot={projectRoot} betaReadiness={betaReadiness} />

      <button style={styles.completeBtn} onClick={onComplete}>
        Open Workspace
      </button>
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
  // v2.14.11: Guided panel
  guidedPanel: {
    backgroundColor: '#1e293b',
    borderRadius: '8px',
    padding: '16px',
    marginBottom: '12px',
    border: '1px solid #334155',
  },
  guidedTitle: {
    fontSize: '14px',
    fontWeight: 600,
    color: '#f1f5f9',
    margin: '0 0 12px 0',
  },
  guidedList: {
    fontSize: '13px',
    color: '#cbd5e1',
    lineHeight: '2',
    paddingLeft: '20px',
    margin: 0,
  },
  stepDone: {
    color: '#22c55e',
  },
  stepCurrent: {
    color: '#6366f1',
    fontWeight: 600,
  },
  stepPending: {
    color: '#64748b',
  },
  stepOptional: {
    color: '#f59e0b',
  },
  // v2.14.11: Status banner
  statusBanner: {
    padding: '10px 12px',
    borderRadius: '6px',
    fontSize: '13px',
    fontWeight: 600,
    color: '#fff',
    marginBottom: '12px',
  },
  statusNote: {
    fontWeight: 400,
    fontSize: '12px',
    opacity: 0.9,
  },
  loading: {
    color: '#94a3b8',
    fontSize: '13px',
    textAlign: 'center' as const,
    padding: '8px',
  },
  // Collapsible technical details
  detailsSection: {
    marginBottom: '12px',
    backgroundColor: '#111827',
    borderRadius: '8px',
    padding: '8px 12px',
  },
  detailsSummary: {
    cursor: 'pointer',
    fontSize: '13px',
    color: '#64748b',
    fontWeight: 600,
    padding: '4px 0',
  },
  section: {
    marginBottom: '12px',
    marginTop: '8px',
  },
  sectionTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#64748b',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    margin: '0 0 6px 0',
  },
  row: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '3px 0',
    fontSize: '13px',
    color: '#e2e8f0',
  },
  errorNote: {
    fontSize: '12px',
    color: '#f59e0b',
    margin: '4px 0 0 0',
  },
  hintNote: {
    fontSize: '12px',
    color: '#94a3b8',
    margin: '6px 0 0 0',
    lineHeight: '1.4',
  },
  optionalBadge: {
    fontSize: '10px',
    backgroundColor: '#3730a3',
    color: '#c7d2fe',
    padding: '1px 6px',
    borderRadius: '4px',
    marginLeft: '6px',
    textTransform: 'none' as const,
    letterSpacing: '0',
  },
  warningCard: {
    fontSize: '12px',
    color: '#e2e8f0',
    padding: '8px',
    backgroundColor: '#1e1e2e',
    borderRadius: '6px',
    marginBottom: '6px',
  },
  recovery: {
    fontSize: '11px',
    color: '#94a3b8',
    margin: '4px 0 0 0',
  },
  // Warnings panel
  warningsPanel: {
    backgroundColor: '#1e1e2e',
    borderRadius: '8px',
    padding: '12px',
    marginBottom: '12px',
  },
  noticeRow: {
    fontSize: '12px',
    color: '#94a3b8',
    lineHeight: '1.6',
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
