import React, { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { validateProject, createSampleProject } from '../lib/projectValidation';
import type { ProjectValidationResult } from '../lib/projectValidation';

interface Props {
  onProjectSelected: (path: string) => void;
  onSampleProject: (path: string) => void;
  onBack: () => void;
}

export const ProjectStep: React.FC<Props> = ({ onProjectSelected, onSampleProject, onBack }) => {
  const [validation, setValidation] = useState<ProjectValidationResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleOpenFolder = async () => {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected && typeof selected === 'string') {
        setLoading(true);
        setError(null);
        const result = await validateProject(selected);
        setValidation(result);
        setLoading(false);

        if (result.status === 'valid') {
          onProjectSelected(selected);
        }
      }
    } catch (e) {
      setError(String(e));
      setLoading(false);
    }
  };

  const handleSampleProject = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await createSampleProject();
      setLoading(false);
      onSampleProject(result.path);
    } catch (e) {
      setError(String(e));
      setLoading(false);
    }
  };

  return (
    <div>
      <div style={styles.nav}>
        <button style={styles.backBtn} onClick={onBack}>&larr; Back</button>
        <h2 style={styles.heading}>Choose Project</h2>
      </div>

      <div style={styles.options}>
        <button
          style={{ ...styles.optionCard, ...styles.optionPrimary }}
          onClick={handleOpenFolder}
          disabled={loading}
        >
          <strong>Open existing project</strong>
          <p style={styles.optionDesc}>Select a folder containing an Orqestra repository</p>
        </button>

        <button
          style={{ ...styles.optionCard, ...styles.optionSecondary }}
          onClick={handleSampleProject}
          disabled={loading}
        >
          <strong>Try sample project</strong>
          <p style={styles.optionDesc}>Generate a demo project with sample tasks</p>
        </button>
      </div>

      {loading && <p style={styles.loading}>Loading...</p>}
      {error && <p style={styles.error}>{error}</p>}

      {validation && validation.status !== 'valid' && (
        <div style={styles.validationResult}>
          <h3 style={styles.validationTitle}>
            Status: {validation.status}
          </h3>
          {validation.errors.map((e, i) => (
            <p key={i} style={styles.validationError}>{e.message}</p>
          ))}
          {validation.suggested_actions.map((a, i) => (
            <p key={i} style={styles.validationAction}>
              &rarr; {a.label}: {a.description}
            </p>
          ))}
        </div>
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
  options: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
    marginBottom: '16px',
  },
  optionCard: {
    padding: '16px',
    borderRadius: '8px',
    border: 'none',
    cursor: 'pointer',
    textAlign: 'left' as const,
  },
  optionPrimary: {
    backgroundColor: '#1e3a5f',
    color: '#e2e8f0',
  },
  optionSecondary: {
    backgroundColor: '#1a2744',
    color: '#e2e8f0',
  },
  optionDesc: {
    fontSize: '12px',
    color: '#94a3b8',
    margin: '4px 0 0 0',
  },
  loading: {
    color: '#94a3b8',
    fontSize: '13px',
    textAlign: 'center' as const,
  },
  error: {
    color: '#f87171',
    fontSize: '13px',
  },
  validationResult: {
    marginTop: '16px',
    padding: '12px',
    backgroundColor: '#1e1e2e',
    borderRadius: '8px',
  },
  validationTitle: {
    fontSize: '14px',
    fontWeight: 600,
    color: '#fbbf24',
    margin: '0 0 8px 0',
  },
  validationError: {
    fontSize: '13px',
    color: '#f87171',
    margin: '4px 0',
  },
  validationAction: {
    fontSize: '13px',
    color: '#94a3b8',
    margin: '4px 0',
  },
};
