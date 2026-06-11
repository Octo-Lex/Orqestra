import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface BetaEvidenceExportPanelProps {
  projectRoot: string | null;
  betaReadiness?: {
    readiness: string;
    blocking: boolean;
    checks: Record<string, boolean | null>;
    blocked_features: string[];
  } | null;
}

export const BetaEvidenceExportPanel: React.FC<BetaEvidenceExportPanelProps> = ({ projectRoot, betaReadiness }) => {
  const [state, setState] = useState<'idle' | 'reviewing' | 'exporting' | 'done' | 'error'>('idle');
  const [ackRedaction, setAckRedaction] = useState(false);
  const [ackLocalOnly, setAckLocalOnly] = useState(false);
  const [exportPath, setExportPath] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const canExport = ackRedaction && ackLocalOnly;

  const handleExport = async () => {
    setState('exporting');
    try {
      const checks = betaReadiness?.checks ?? {};
      const aiReachable = checks.ai_service_reachable === true;

      const steps = {
        app_launched: true,
        repo_opened: !!projectRoot,
        roadmap_detected: checks.roadmap_found === true,
        pm_views_rendered: checks.roadmap_found === true, // if roadmap found, views likely rendered
        readiness_reviewed: true, // user is on readiness panel exporting
        ai_service_available: aiReachable,
        agent_flow_completed: false, // not tracked in this flow
        ai_degraded_mode_understood: !aiReachable,
        dashboard_evidence_viewed: false, // not tracked in this flow
        diagnostics_exported: true, // this export counts as diagnostics
      };

      // Build observed failures from blocked features
      const failures = (betaReadiness?.blocked_features ?? []).map((f: string) => ({
        code: f === 'agent_execution' ? 'AI_SERVICE_UNAVAILABLE' : 'UNKNOWN_FAILURE',
        severity: 'warning',
        category: f === 'agent_execution' ? 'ai_service' : 'unknown',
        user_recoverable: true,
        blocked_steps: [f],
      }));

      const result = await invoke('export_beta_evidence_cmd', {
        projectRoot: projectRoot || null,
        consent: {
          explicit: true,
          timestamp: new Date().toISOString(),
          user_acknowledged_redaction: ackRedaction,
          user_acknowledged_local_only: ackLocalOnly,
        },
        steps,
        failures: failures.length > 0 ? failures : null,
      }) as { ok: boolean; path?: string; files?: string[]; code?: string };

      if (result.ok) {
        setExportPath(result.path || null);
        setState('done');
      } else {
        setErrorMsg(result.code || 'Unknown error');
        setState('error');
      }
    } catch (e) {
      setErrorMsg(String(e));
      setState('error');
    }
  };

  if (state === 'done') {
    return (
      <div style={styles.panel}>
        <h3 style={styles.title}>Beta Evidence Exported</h3>
        <p style={styles.successText}>✓ Bundle created successfully.</p>
        {exportPath && (
          <p style={styles.pathText}>Saved to: {exportPath}</p>
        )}
        <p style={styles.hint}>
          The bundle is local-only. No automatic upload occurred.
          Review the files before sharing.
        </p>
        <button style={styles.resetBtn} onClick={() => setState('idle')}>
          Export Another
        </button>
      </div>
    );
  }

  if (state === 'error') {
    return (
      <div style={styles.panel}>
        <h3 style={styles.title}>Export Failed</h3>
        <p style={styles.errorText}>{errorMsg}</p>
        <button style={styles.resetBtn} onClick={() => setState('idle')}>
          Try Again
        </button>
      </div>
    );
  }

  if (state === 'reviewing' || state === 'exporting') {
    return (
      <div style={styles.panel}>
        <h3 style={styles.title}>Export Beta Evidence</h3>
        <p style={styles.description}>
          This export creates a local, redacted evidence bundle for beta feedback.
          It does not upload anything automatically.
          You can review the files before sharing.
        </p>

        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>Included Categories</h4>
          <ul style={styles.list}>
            <li>Session outcome (steps completed, warnings)</li>
            <li>Repo metadata (hashed path, branch, dirty state)</li>
            <li>Platform info (OS, architecture)</li>
            <li>Failure taxonomy (structured failure codes)</li>
            <li>Git status (branch, clean/dirty, remote)</li>
          </ul>
        </div>

        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>Excluded Categories</h4>
          <ul style={styles.list}>
            <li>Tokens, PATs, API keys, bearer tokens</li>
            <li>Raw repo paths (hashed instead)</li>
            <li>Raw user home paths</li>
            <li>Full file contents</li>
            <li>Remote URLs containing credentials</li>
          </ul>
        </div>

        <div style={styles.consentSection}>
          <label style={styles.checkLabel}>
            <input
              type="checkbox"
              checked={ackRedaction}
              onChange={(e) => setAckRedaction(e.target.checked)}
            />
            <span style={styles.checkText}>
              I understand that secrets, tokens, raw paths, and file contents are excluded or redacted.
            </span>
          </label>
          <label style={styles.checkLabel}>
            <input
              type="checkbox"
              checked={ackLocalOnly}
              onChange={(e) => setAckLocalOnly(e.target.checked)}
            />
            <span style={styles.checkText}>
              I understand that the bundle is saved locally only. No automatic upload will occur.
            </span>
          </label>
        </div>

        <div style={styles.buttonRow}>
          <button
            style={{
              ...styles.exportBtn,
              opacity: canExport && state === 'reviewing' ? 1 : 0.5,
            }}
            disabled={!canExport || state === 'exporting'}
            onClick={handleExport}
          >
            {state === 'exporting' ? 'Exporting...' : 'Export Locally'}
          </button>
          <button style={styles.cancelBtn} onClick={() => setState('idle')}>
            Cancel
          </button>
        </div>
      </div>
    );
  }

  // Idle state
  return (
    <div style={styles.panel}>
      <h3 style={styles.title}>Beta Evidence</h3>
      <p style={styles.description}>
        Export a consented, redacted evidence bundle for beta feedback.
        The bundle is local-only and contains no secrets or raw paths.
      </p>
      <button style={styles.startBtn} onClick={() => setState('reviewing')}>
        Export Beta Evidence
      </button>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  panel: {
    backgroundColor: '#1e293b',
    borderRadius: '8px',
    padding: '16px',
    marginBottom: '12px',
  },
  title: {
    fontSize: '14px',
    fontWeight: 600,
    color: '#f1f5f9',
    margin: '0 0 8px 0',
  },
  description: {
    fontSize: '13px',
    color: '#94a3b8',
    margin: '0 0 12px 0',
    lineHeight: '1.4',
  },
  section: {
    marginBottom: '12px',
  },
  sectionTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#64748b',
    textTransform: 'uppercase' as const,
    letterSpacing: '0.5px',
    margin: '0 0 6px 0',
  },
  list: {
    fontSize: '12px',
    color: '#cbd5e1',
    margin: 0,
    paddingLeft: '20px',
    lineHeight: '1.6',
  },
  consentSection: {
    marginBottom: '12px',
  },
  checkLabel: {
    display: 'flex',
    alignItems: 'flex-start',
    gap: '8px',
    marginBottom: '8px',
    cursor: 'pointer',
  },
  checkText: {
    fontSize: '12px',
    color: '#e2e8f0',
    lineHeight: '1.4',
  },
  buttonRow: {
    display: 'flex',
    gap: '8px',
  },
  exportBtn: {
    padding: '8px 16px',
    borderRadius: '6px',
    border: 'none',
    backgroundColor: '#22c55e',
    color: '#fff',
    fontSize: '13px',
    fontWeight: 600,
    cursor: 'pointer',
  },
  cancelBtn: {
    padding: '8px 16px',
    borderRadius: '6px',
    border: '1px solid #475569',
    backgroundColor: 'transparent',
    color: '#94a3b8',
    fontSize: '13px',
    cursor: 'pointer',
  },
  startBtn: {
    padding: '8px 16px',
    borderRadius: '6px',
    border: 'none',
    backgroundColor: '#6366f1',
    color: '#fff',
    fontSize: '13px',
    fontWeight: 600,
    cursor: 'pointer',
  },
  resetBtn: {
    padding: '6px 12px',
    borderRadius: '6px',
    border: '1px solid #475569',
    backgroundColor: 'transparent',
    color: '#94a3b8',
    fontSize: '12px',
    cursor: 'pointer',
    marginTop: '8px',
  },
  successText: {
    fontSize: '13px',
    color: '#22c55e',
    margin: '0 0 4px 0',
  },
  errorText: {
    fontSize: '13px',
    color: '#ef4444',
    margin: '0 0 4px 0',
  },
  pathText: {
    fontSize: '12px',
    color: '#94a3b8',
    margin: '0 0 8px 0',
    wordBreak: 'break-all' as const,
  },
  hint: {
    fontSize: '12px',
    color: '#64748b',
    margin: '4px 0 0 0',
  },
};
