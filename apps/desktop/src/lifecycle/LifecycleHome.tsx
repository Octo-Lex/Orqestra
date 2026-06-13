import React, { useEffect, useState, useCallback } from 'react';
import type { LifecycleState, StageInfo, LifecycleStageName } from '../lifecycle/types';
import { IMPLEMENTED_STAGES } from '../lifecycle/types';
import { lifecycleGetState, lifecycleInit, lifecycleApproveGate, lifecycleRejectGate } from '../lifecycle/api';

interface Props {
  projectRoot: string;
}

/**
 * Lifecycle Home — the workflow operating surface.
 *
 * This is NOT a chat. It is a structured work surface that always shows:
 * - Where am I in the lifecycle?
 * - What artifact am I producing?
 * - Which role is helping?
 * - Which skill is active?
 * - What is required before I can proceed?
 * - What evidence exists?
 * - What is the next safe action?
 */
export const LifecycleHome: React.FC<Props> = ({ projectRoot }) => {
  const [state, setState] = useState<LifecycleState | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!projectRoot) return;
    try {
      const s = await lifecycleGetState(projectRoot);
      setState(s);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [projectRoot]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleStart = useCallback(async () => {
    if (!projectRoot) return;
    try {
      await lifecycleInit(projectRoot);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }, [projectRoot, refresh]);

  const handleApprove = useCallback(async () => {
    if (!projectRoot) return;
    try {
      await lifecycleApproveGate(projectRoot);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }, [projectRoot, refresh]);

  const handleReject = useCallback(async (reason: string) => {
    if (!projectRoot) return;
    try {
      await lifecycleRejectGate(projectRoot, reason);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }, [projectRoot, refresh]);

  if (loading) {
    return <div style={styles.loading}>Loading lifecycle state...</div>;
  }

  // Not initialized — show start button
  if (!state?.initialized) {
    return (
      <div style={styles.container}>
        <div style={styles.welcome}>
          <h2 style={styles.welcomeTitle}>Development Lifecycle</h2>
          <p style={styles.welcomeText}>
            Orqestra guides a product from idea to release and learning using
            bounded AI agents, typed artifacts, human approval gates, and
            evidence-backed state.
          </p>
          <p style={styles.welcomeText}>
            Start the lifecycle to analyze your repository, define what you're
            building, and track it through 13 governed stages.
          </p>
          <button style={styles.startBtn} onClick={handleStart}>
            Start Lifecycle Mode
          </button>
          {error && <p style={styles.error}>{error}</p>}
        </div>
      </div>
    );
  }

  const currentStage = state.current_stage;
  const isImplemented = currentStage ? IMPLEMENTED_STAGES.includes(currentStage) : false;
  const currentGate = state.gates.find(g => {
    const gateStage = g.gate.split('_')[0];
    return currentStage?.startsWith(gateStage) || g.gate.startsWith(currentStage || '');
  });
  const gateApproved = currentGate?.status === 'approved';
  const artifactsForStage = state.artifacts.filter(() => {
    // Simple heuristic: artifacts without feature_id are project-level
    return true; // Show all artifacts in the list
  });

  return (
    <div style={styles.container}>
      {/* Stage indicator bar */}
      <StageIndicator stages={state.stages} current={currentStage} />

      {/* Current stage panel */}
      {state.current_stage_name && (
        <div style={styles.stagePanel}>
          <div style={styles.stageHeader}>
            <h3 style={styles.stageTitle}>
              {state.current_stage_name}
            </h3>
            <span style={{
              ...styles.stageBadge,
              backgroundColor: isImplemented ? '#3730a3' : '#374151',
              color: isImplemented ? '#c7d2fe' : '#94a3b8',
            }}>
              {isImplemented ? 'Active' : 'Placeholder'}
            </span>
          </div>

          <p style={styles.stagePurpose}>
            {state.current_stage_purpose}
          </p>

          {/* Active role */}
          <div style={styles.infoRow}>
            <span style={styles.infoLabel}>Active role</span>
            <span style={styles.infoValue}>
              {getRoleForStage(currentStage)}
            </span>
          </div>

          {/* Active skill */}
          <div style={styles.infoRow}>
            <span style={styles.infoLabel}>Active skill</span>
            <span style={styles.infoValue}>
              {getSkillForStage(currentStage)}
            </span>
          </div>

          {/* Gate status */}
          <div style={styles.infoRow}>
            <span style={styles.infoLabel}>Gate status</span>
            <span style={{
              ...styles.infoValue,
              color: gateApproved ? '#22c55e' : '#f59e0b',
            }}>
              {gateApproved ? 'Approved — ready to advance' :
               currentGate?.status === 'rejected' ? 'Rejected' :
               currentGate?.status === 'requested' ? 'Requested — awaiting decision' :
               'Pending — review required before advancing'}
            </span>
          </div>

          {/* Evidence count */}
          <div style={styles.infoRow}>
            <span style={styles.infoLabel}>Evidence</span>
            <span style={styles.infoValue}>
              {state.artifacts.length} artifact{state.artifacts.length !== 1 ? 's' : ''}
              {' · '}
              {state.events_count} event{state.events_count !== 1 ? 's' : ''}
            </span>
          </div>

          {/* Next actions */}
          <div style={styles.actions}>
            {isImplemented && !gateApproved && (
              <>
                <button style={{ ...styles.btn, ...styles.btnPrimary }} onClick={handleApprove}>
                  Approve &amp; Advance →
                </button>
                <button
                  style={{ ...styles.btn, ...styles.btnSecondary }}
                  onClick={() => {
                    const reason = prompt('Reason for rejection:');
                    if (reason) handleReject(reason);
                  }}
                >
                  Reject
                </button>
              </>
            )}
            {isImplemented && gateApproved && (
              <p style={styles.gateApproved}>Gate approved — you can advance.</p>
            )}
            {!isImplemented && (
              <p style={styles.placeholderNote}>
                This stage is part of the lifecycle but not yet implemented.
                It will be available in a future release.
              </p>
            )}
          </div>

          {error && <p style={styles.error}>{error}</p>}
        </div>
      )}

      {/* Artifacts list */}
      {artifactsForStage.length > 0 && (
        <div style={styles.artifactsPanel}>
          <h4 style={styles.sectionTitle}>Artifacts</h4>
          {artifactsForStage.map((a, i) => (
            <div key={i} style={styles.artifactRow}>
              <span style={styles.artifactType}>{a.artifact_type}</span>
              <span style={styles.artifactPath}>{a.path}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

// ---------------------------------------------------------------------------
// Stage indicator bar
// ---------------------------------------------------------------------------

const StageIndicator: React.FC<{ stages: StageInfo[]; current: LifecycleStageName | null }> = ({
  stages,
  current,
}) => {
  return (
    <div style={styles.stageBar}>
      {stages.map((stage) => {
        const isCurrent = stage.name === current;
        const isPast = current ? stage.index < (stages.find(s => s.name === current)?.index ?? 0) : false;
        return (
          <React.Fragment key={stage.name}>
            <div style={{
              ...styles.stageDot,
              backgroundColor: isCurrent ? '#6366f1' : isPast ? '#312e81' : '#1e293b',
              color: isCurrent ? '#fff' : isPast ? '#a5b4fc' : '#475569',
              borderColor: isCurrent ? '#818cf8' : 'transparent',
            }}>
              {stage.index}
            </div>
            <span style={{
              ...styles.stageLabel,
              color: isCurrent ? '#e2e8f0' : isPast ? '#64748b' : '#475569',
              fontWeight: isCurrent ? 600 : 400,
            }}>
              {stage.display_name}
            </span>
            {stage.index < stages.length - 1 && (
              <div style={{
                ...styles.stageConnector,
                backgroundColor: isPast ? '#312e81' : '#1e293b',
              }} />
            )}
          </React.Fragment>
        );
      })}
    </div>
  );
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getRoleForStage(stage: LifecycleStageName | null): string {
  if (!stage) return '—';
  switch (stage) {
    case 'orient': return 'Architect';
    case 'discover': return 'Product Manager';
    case 'define': return 'Product Manager';
    case 'design': return 'UX Designer + Architect';
    case 'plan': return 'Tech Lead + QA Agent';
    case 'prepare': return 'Implementation Agent';
    case 'build': return 'Implementation Agent';
    case 'verify': return 'QA Agent';
    case 'review': return 'Tech Lead + Security Reviewer';
    case 'ship': return 'Release/Evidence Agent';
    case 'observe': return 'QA Agent + Product Manager';
    case 'learn': return 'Product Manager + Architect';
    case 'evolve': return 'Product Manager + Tech Lead';
    default: return '—';
  }
}

function getSkillForStage(stage: LifecycleStageName | null): string {
  if (!stage) return '—';
  switch (stage) {
    case 'orient': return 'Source-driven analysis';
    case 'discover': return 'Interview/clarification';
    case 'define': return 'Spec-driven development';
    case 'design': return 'API/interface design';
    case 'plan': return 'Planning and task breakdown';
    case 'prepare': return 'Git workflow and versioning';
    case 'build': return 'Incremental implementation';
    case 'verify': return 'Test-driven development';
    case 'review': return 'Code review and quality';
    case 'ship': return 'Shipping and launch';
    case 'observe': return 'Observability and instrumentation';
    case 'learn': return 'Doubt-driven development';
    case 'evolve': return 'Planning and task breakdown';
    default: return '—';
  }
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
    padding: '16px',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    color: '#e2e8f0',
  },
  loading: {
    padding: '2rem',
    color: '#94a3b8',
    textAlign: 'center',
    fontFamily: 'system-ui',
  },
  welcome: {
    textAlign: 'center',
    padding: '2rem',
    maxWidth: '600px',
    margin: '0 auto',
  },
  welcomeTitle: {
    fontSize: '22px',
    fontWeight: 700,
    color: '#f1f5f9',
    marginBottom: '12px',
  },
  welcomeText: {
    fontSize: '14px',
    color: '#94a3b8',
    lineHeight: 1.6,
    marginBottom: '12px',
  },
  startBtn: {
    padding: '12px 32px',
    borderRadius: '8px',
    border: 'none',
    backgroundColor: '#6366f1',
    color: '#fff',
    fontSize: '15px',
    fontWeight: 600,
    cursor: 'pointer',
    marginTop: '16px',
  },
  // Stage bar
  stageBar: {
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
    flexWrap: 'wrap',
    padding: '12px 16px',
    backgroundColor: '#0f172a',
    borderRadius: '8px',
    overflowX: 'auto',
  },
  stageDot: {
    width: '24px',
    height: '24px',
    borderRadius: '50%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '10px',
    fontWeight: 700,
    border: '2px solid transparent',
    flexShrink: 0,
  },
  stageLabel: {
    fontSize: '11px',
    whiteSpace: 'nowrap',
  },
  stageConnector: {
    width: '12px',
    height: '2px',
    flexShrink: 0,
  },
  // Stage panel
  stagePanel: {
    backgroundColor: '#1e293b',
    borderRadius: '12px',
    padding: '20px',
    border: '1px solid #334155',
  },
  stageHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    marginBottom: '8px',
  },
  stageTitle: {
    fontSize: '20px',
    fontWeight: 700,
    color: '#f1f5f9',
    margin: 0,
  },
  stageBadge: {
    fontSize: '10px',
    padding: '2px 8px',
    borderRadius: '4px',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
  },
  stagePurpose: {
    fontSize: '14px',
    color: '#cbd5e1',
    margin: '0 0 16px 0',
    fontStyle: 'italic',
  },
  // Info rows
  infoRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '6px 0',
    borderBottom: '1px solid #334155',
  },
  infoLabel: {
    fontSize: '12px',
    color: '#64748b',
    fontWeight: 600,
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
    minWidth: '120px',
  },
  infoValue: {
    fontSize: '14px',
    color: '#e2e8f0',
  },
  // Actions
  actions: {
    display: 'flex',
    gap: '8px',
    marginTop: '16px',
  },
  btn: {
    padding: '10px 20px',
    borderRadius: '8px',
    border: 'none',
    fontSize: '14px',
    fontWeight: 600,
    cursor: 'pointer',
  },
  btnPrimary: {
    backgroundColor: '#6366f1',
    color: '#fff',
  },
  btnSecondary: {
    backgroundColor: 'transparent',
    color: '#ef4444',
    border: '1px solid #ef4444',
  },
  gateApproved: {
    fontSize: '13px',
    color: '#22c55e',
    margin: 0,
  },
  placeholderNote: {
    fontSize: '13px',
    color: '#64748b',
    margin: 0,
    fontStyle: 'italic',
  },
  // Artifacts
  artifactsPanel: {
    backgroundColor: '#111827',
    borderRadius: '8px',
    padding: '12px 16px',
  },
  sectionTitle: {
    fontSize: '12px',
    fontWeight: 600,
    color: '#64748b',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
    margin: '0 0 8px 0',
  },
  artifactRow: {
    display: 'flex',
    gap: '12px',
    padding: '4px 0',
    fontSize: '12px',
  },
  artifactType: {
    color: '#818cf8',
    fontWeight: 600,
    minWidth: '160px',
  },
  artifactPath: {
    color: '#94a3b8',
    fontFamily: 'monospace',
  },
  // Error
  error: {
    fontSize: '13px',
    color: '#ef4444',
    marginTop: '8px',
  },
};
