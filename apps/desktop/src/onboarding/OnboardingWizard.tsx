import React, { useState } from 'react';
import { WelcomeStep } from './WelcomeStep';
import { ProjectStep } from './ProjectStep';
import { ReadinessStep } from './ReadinessStep';
import { invoke } from '@tauri-apps/api/core';

interface Props {
  onComplete: (projectRoot: string) => void;
}

const STEPS = ['welcome', 'project', 'readiness', 'complete'] as const;
type Step = typeof STEPS[number];

export const OnboardingWizard: React.FC<Props> = ({ onComplete }) => {
  const [step, setStep] = useState<Step>('welcome');
  const [projectRoot, setProjectRoot] = useState<string | null>(null);

  const handleOpenProject = (path: string) => {
    setProjectRoot(path);
    setStep('readiness');
  };

  const handleSampleProject = (path: string) => {
    setProjectRoot(path);
    setStep('readiness');
  };

  const handleComplete = async () => {
    await invoke('set_onboarding_state_cmd', {
      update: {
        onboarding_completed: true,
        last_project_root: projectRoot,
        chosen_path: projectRoot,
      },
    });
    if (projectRoot) {
      onComplete(projectRoot);
    }
  };

  return (
    <div style={styles.container}>
      <div style={styles.wizard}>
        <div style={styles.header}>
          <h1 style={styles.title}>Orqestra</h1>
          <p style={styles.subtitle}>
            Local-first, AI-native project management
          </p>
          {/* Progress bar */}
          <div style={styles.progressBar}>
            {STEPS.map((s, i) => (
              <div
                key={s}
                style={{
                  ...styles.progressDot,
                  backgroundColor: STEPS.indexOf(step) >= i ? '#6366f1' : '#374151',
                }}
              />
            ))}
          </div>
        </div>

        <div style={styles.content}>
          {step === 'welcome' && (
            <WelcomeStep
              onOpenProject={() => setStep('project')}
              onTrySample={() => setStep('project')}
              onViewSetup={() => setStep('readiness')}
            />
          )}
          {step === 'project' && (
            <ProjectStep
              onProjectSelected={handleOpenProject}
              onSampleProject={handleSampleProject}
              onBack={() => setStep('welcome')}
            />
          )}
          {step === 'readiness' && (
            <ReadinessStep
              projectRoot={projectRoot}
              onComplete={handleComplete}
              onBack={() => setStep('project')}
            />
          )}
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    minHeight: '100vh',
    backgroundColor: '#0f172a',
    color: '#e2e8f0',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  },
  wizard: {
    width: '100%',
    maxWidth: '640px',
    padding: '32px',
  },
  header: {
    textAlign: 'center' as const,
    marginBottom: '32px',
  },
  title: {
    fontSize: '28px',
    fontWeight: 700,
    margin: 0,
    color: '#f1f5f9',
  },
  subtitle: {
    fontSize: '14px',
    color: '#94a3b8',
    marginTop: '4px',
  },
  progressBar: {
    display: 'flex',
    justifyContent: 'center',
    gap: '8px',
    marginTop: '16px',
  },
  progressDot: {
    width: '32px',
    height: '4px',
    borderRadius: '2px',
    transition: 'background-color 0.2s',
  },
  content: {
    backgroundColor: '#1e293b',
    borderRadius: '12px',
    padding: '24px',
  },
};
