import React from 'react';

interface Props {
  onOpenProject: () => void;
  onTrySample: () => void;
  onViewSetup: () => void;
}

export const WelcomeStep: React.FC<Props> = ({ onOpenProject, onTrySample, onViewSetup }) => {
  return (
    <div>
      <h2 style={styles.heading}>Welcome to Orqestra</h2>
      <p style={styles.description}>
        Orqestra turns a Git repository into a local-first project-management,
        semantic-history, and agent-assisted development workspace.
      </p>
      <p style={styles.note}>
        Works locally with a Git repository. AI and cloud features are optional.
      </p>

      <div style={styles.actions}>
        <button style={{ ...styles.button, ...styles.primary }} onClick={onOpenProject}>
          Open existing project
        </button>
        <button style={{ ...styles.button, ...styles.secondary }} onClick={onTrySample}>
          Try sample project
        </button>
        <button style={{ ...styles.button, ...styles.tertiary }} onClick={onViewSetup}>
          View setup checklist
        </button>
      </div>

      <div style={styles.features}>
        <div style={styles.feature}>
          <span style={styles.featureIcon}>&#9783;</span>
          <div>
            <strong>Local PM</strong>
            <p style={styles.featureText}>Roadmap, table, Gantt, Kanban views</p>
          </div>
        </div>
        <div style={styles.feature}>
          <span style={styles.featureIcon}>&#9881;</span>
          <div>
            <strong>AI Agents</strong>
            <p style={styles.featureText}>Docs &amp; bugfix review (optional)</p>
          </div>
        </div>
        <div style={styles.feature}>
          <span style={styles.featureIcon}>&#9733;</span>
          <div>
            <strong>Dashboard</strong>
            <p style={styles.featureText}>Public roadmap dashboard (optional)</p>
          </div>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  heading: {
    fontSize: '20px',
    fontWeight: 600,
    margin: '0 0 8px 0',
    color: '#f1f5f9',
  },
  description: {
    fontSize: '14px',
    color: '#cbd5e1',
    lineHeight: 1.5,
    margin: '0 0 12px 0',
  },
  note: {
    fontSize: '13px',
    color: '#94a3b8',
    margin: '0 0 24px 0',
  },
  actions: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
    marginBottom: '24px',
  },
  button: {
    padding: '10px 16px',
    borderRadius: '8px',
    border: 'none',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer',
    textAlign: 'left' as const,
  },
  primary: {
    backgroundColor: '#6366f1',
    color: '#fff',
  },
  secondary: {
    backgroundColor: '#334155',
    color: '#e2e8f0',
  },
  tertiary: {
    backgroundColor: 'transparent',
    color: '#94a3b8',
    border: '1px solid #334155',
  },
  features: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
    paddingTop: '16px',
    borderTop: '1px solid #334155',
  },
  feature: {
    display: 'flex',
    gap: '12px',
    alignItems: 'flex-start',
  },
  featureIcon: {
    fontSize: '18px',
    color: '#6366f1',
    width: '24px',
    textAlign: 'center' as const,
  },
  featureText: {
    fontSize: '12px',
    color: '#94a3b8',
    margin: '2px 0 0 0',
  },
};
