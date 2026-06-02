import React from 'react';
import { createSampleProject } from '../lib/projectValidation';

interface Props {
  onCreated: (path: string) => void;
  onBack: () => void;
}

export const SampleProjectStep: React.FC<Props> = ({ onCreated, onBack }) => {
  const handleCreate = async () => {
    const result = await createSampleProject();
    onCreated(result.path);
  };

  return (
    <div>
      <h2 style={styles.heading}>Sample Project</h2>
      <p style={styles.description}>
        Generate a demo project with sample tasks to explore Orqestra features
        including Table, Gantt, and Kanban views.
      </p>
      <p style={styles.note}>
        The sample project includes 4 tasks in different states (backlog, in-progress, done)
        with dependencies, labels, and time estimates.
      </p>
      <div style={styles.actions}>
        <button style={{ ...styles.button, ...styles.primary }} onClick={handleCreate}>
          Create sample project
        </button>
        <button style={{ ...styles.button, ...styles.secondary }} onClick={onBack}>
          Back
        </button>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  heading: {
    fontSize: '18px',
    fontWeight: 600,
    margin: '0 0 8px 0',
    color: '#f1f5f9',
  },
  description: {
    fontSize: '14px',
    color: '#cbd5e1',
    lineHeight: 1.5,
    margin: '0 0 8px 0',
  },
  note: {
    fontSize: '13px',
    color: '#94a3b8',
    margin: '0 0 16px 0',
  },
  actions: {
    display: 'flex',
    gap: '8px',
  },
  button: {
    padding: '10px 16px',
    borderRadius: '8px',
    border: 'none',
    fontSize: '14px',
    fontWeight: 500,
    cursor: 'pointer',
  },
  primary: {
    backgroundColor: '#6366f1',
    color: '#fff',
  },
  secondary: {
    backgroundColor: '#334155',
    color: '#e2e8f0',
  },
};
