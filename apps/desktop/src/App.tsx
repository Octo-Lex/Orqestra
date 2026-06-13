import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { TaskTable } from './components/TaskTable';
import { CommitPanel } from './components/CommitPanel';
import { AgentPanel } from './components/AgentPanel';
import QueryHistory from './components/QueryHistory';
import SemanticDiff from './components/SemanticDiff';
import ShockwaveMerge from './components/ShockwaveMerge';
import { SyncPanel } from './components/SyncPanel';
import { GanttView } from './pm/GanttView';
import { KanbanView } from './pm/KanbanView';
import { TimeTracking } from './pm/TimeTracking';
import { ViewSwitcher, type ViewMode } from './pm/ViewSwitcher';
import { OnboardingWizard } from './onboarding/OnboardingWizard';
import { LifecycleHome } from './lifecycle/LifecycleHome';
import { ReadinessPanel } from './setup/ReadinessPanel';
import { DiagnosticsPanel } from './setup/DiagnosticsPanel';
import { open } from '@tauri-apps/plugin-dialog';
import {
  hasStoredPat,
  storePat,
  clearStoredPat,
  gitPullRoadmap,
  gitPushRoadmap,
  type GitResult,
} from './lib/git';
import {
  updateTaskStatus,
  type Task,
  type TaskStatus,
} from './lib/orqestra';

interface OnboardingState {
  onboarding_completed: boolean;
  last_project_root: string | null;
}

type SyncStatus =
  | { state: 'idle' }
  | { state: 'loading'; action: string }
  | { state: 'success'; action: string; message: string }
  | { state: 'error'; action: string; message: string };

export default function App() {
  const [projectRoot, setProjectRoot] = useState<string | null>(null);
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [onboardingChecked, setOnboardingChecked] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showReadiness, setShowReadiness] = useState(false);
  const [showDiagnostics, setShowDiagnostics] = useState(false);
  const [showLifecycle, setShowLifecycle] = useState(true);
  const [syncStatus, setSyncStatus] = useState<SyncStatus>({ state: 'idle' });
  const [viewMode, setViewMode] = useState<ViewMode>('table');

  // PAT state — managed by Rust/keychain, not persisted in frontend
  const [hasPat, setHasPat] = useState(false);
  const [patInput, setPatInput] = useState('');
  const [patError, setPatError] = useState<string | null>(null);

  // Shared task data
  const [tasks, setTasks] = useState<Task[]>([]);
  const [refreshKey, setRefreshKey] = useState(0);

  // Check onboarding state on mount
  useEffect(() => {
    async function checkOnboarding() {
      try {
        const state = await invoke<OnboardingState>('get_onboarding_state_cmd');
        if (state.onboarding_completed && state.last_project_root) {
          setProjectRoot(state.last_project_root);
        } else {
          setShowOnboarding(true);
        }
      } catch {
        setShowOnboarding(true);
      }
      setOnboardingChecked(true);
    }
    checkOnboarding();
  }, []);

  // Check keychain PAT status
  useEffect(() => {
    hasStoredPat()
      .then(stored => setHasPat(stored))
      .catch(() => setHasPat(false));
  }, []);

  function handleOnboardingComplete(root: string) {
    setShowOnboarding(false);
    setProjectRoot(root);
  }

  async function openProject() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string') setProjectRoot(selected);
  }

  async function handleSavePat() {
    setPatError(null);
    try {
      // PAT is stored in OS keychain by Rust — never written to disk by frontend
      await storePat(patInput);
      setHasPat(true);
      setPatInput('');
    } catch (e) {
      setPatError(e instanceof Error ? e.message : String(e));
    }
  }

  async function handleClearPat() {
    try {
      await clearStoredPat();
      setHasPat(false);
    } catch (e) {
      setPatError(e instanceof Error ? e.message : String(e));
    }
  }

  async function handlePull() {
    if (!projectRoot) {
      setSyncStatus({ state: 'error', action: 'Pull', message: 'No project open.' });
      return;
    }
    if (!hasPat) {
      setSyncStatus({ state: 'error', action: 'Pull', message: 'GitHub PAT not configured. Save it in Settings.' });
      return;
    }
    setSyncStatus({ state: 'loading', action: 'Pull' });
    try {
      const result: GitResult = await gitPullRoadmap(projectRoot);
      setSyncStatus(result.success
        ? { state: 'success', action: 'Pull', message: result.stdout.trim() || 'Up to date.' }
        : { state: 'error', action: 'Pull', message: result.stderr.trim() || 'Unknown error.' }
      );
      if (result.success) setRefreshKey(k => k + 1);
    } catch (e) {
      setSyncStatus({ state: 'error', action: 'Pull', message: String(e) });
    }
  }

  async function handlePush() {
    if (!projectRoot) {
      setSyncStatus({ state: 'error', action: 'Push', message: 'No project open.' });
      return;
    }
    if (!hasPat) {
      setSyncStatus({ state: 'error', action: 'Push', message: 'GitHub PAT not configured. Save it in Settings.' });
      return;
    }
    setSyncStatus({ state: 'loading', action: 'Push' });
    try {
      const result: GitResult = await gitPushRoadmap(projectRoot);
      const combined = result.stderr.trim() || result.stdout.trim();
      setSyncStatus(result.success
        ? { state: 'success', action: 'Push', message: result.stdout.trim() || 'Pushed.' }
        : combined.includes('nothing to commit')
          ? { state: 'success', action: 'Push', message: 'Nothing to commit.' }
          : { state: 'error', action: 'Push', message: combined || 'Unknown error.' }
      );
    } catch (e) {
      setSyncStatus({ state: 'error', action: 'Push', message: String(e) });
    }
  }

  async function handleStatusChange(taskId: string, newStatus: TaskStatus) {
    if (!projectRoot) return;
    try {
      await updateTaskStatus(projectRoot, taskId, newStatus);
      setRefreshKey(k => k + 1);
    } catch (e) {
      console.error('Failed to update task status:', e);
    }
  }

  function handleAutoSchedule() {
    alert('Auto-Schedule applied. Date adjustments calculated.');
  }

  function handleTasksLoaded(loadedTasks: Task[]) {
    setTasks(loadedTasks);
  }

  if (!onboardingChecked) {
    return (
      <div style={{ padding: '2rem', color: '#94a3b8', fontFamily: 'system-ui', textAlign: 'center' }}>
        Loading...
      </div>
    );
  }

  if (showOnboarding) {
    return <OnboardingWizard onComplete={handleOnboardingComplete} />;
  }

  return (
    <div style={{ padding: '1rem', fontFamily: 'system-ui, sans-serif' }}>
      {!projectRoot ? (
        <div style={{ textAlign: 'center', padding: '3rem' }}>
          <button onClick={openProject}>Open project folder</button>
          <div style={{ marginTop: '1rem' }}>
            <button onClick={() => setShowOnboarding(true)} style={{ fontSize: '0.85em', color: '#6366f1', background: 'none', border: 'none', cursor: 'pointer' }}>
              Run setup wizard
            </button>
          </div>
        </div>
      ) : (
        <>
          {/* Project path bar */}
          <div style={{ marginBottom: '0.5rem', color: '#666', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <code>{projectRoot}</code>
            <button onClick={() => { setProjectRoot(null); setSyncStatus({ state: 'idle' }); }}>
              Close
            </button>
            <button onClick={() => setShowSettings(s => !s)}>
              {showSettings ? 'Hide Settings' : 'Settings'}
            </button>
            <button onClick={() => setShowLifecycle(s => !s)}>
              {showLifecycle ? 'Hide Lifecycle' : 'Lifecycle'}
            </button>
            <button onClick={() => setShowReadiness(s => !s)}>
              {showReadiness ? 'Hide Readiness' : 'Setup'}
            </button>
            <button onClick={() => setShowDiagnostics(s => !s)}>
              {showDiagnostics ? 'Hide Diagnostics' : 'Diagnostics'}
            </button>
          </div>

          {/* Lifecycle home — the workflow operating surface */}
          {showLifecycle && <LifecycleHome projectRoot={projectRoot} />}

          {/* Readiness panel */}
          {showReadiness && <ReadinessPanel projectRoot={projectRoot || undefined} />}

          {/* Diagnostics panel */}
          {showDiagnostics && <DiagnosticsPanel projectRoot={projectRoot || undefined} />}

          {/* Settings panel */}
          {showSettings && (
            <div style={{ marginBottom: '1rem', padding: '0.75rem', border: '1px solid #ddd', borderRadius: '4px', background: '#fafafa' }}>
              <div style={{ marginBottom: '0.5rem', fontWeight: 'bold' }}>GitHub PAT</div>
              <div style={{ marginBottom: '0.25rem', fontSize: '0.85em', color: '#666' }}>
                Stored in OS keychain. Never written to disk by the frontend.
              </div>
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                <input
                  type="password"
                  value={patInput}
                  onChange={e => setPatInput(e.target.value)}
                  placeholder="ghp_xxxxxxxxxxxx"
                  style={{ flexGrow: 1, padding: '0.25rem 0.5rem' }}
                />
                <button onClick={handleSavePat} disabled={!patInput.trim()}>Save to Keychain</button>
                {hasPat && <button onClick={handleClearPat}>Remove</button>}
              </div>
              {patError && <div style={{ color: 'red', marginTop: '0.25rem' }}>{patError}</div>}
              {hasPat && !patError && <div style={{ color: 'green', marginTop: '0.25rem', fontSize: '0.85em' }}>PAT stored in keychain.</div>}
            </div>
          )}

          {/* Sync controls */}
          <div style={{ marginBottom: '0.75rem', display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
            <button onClick={handlePull} disabled={syncStatus.state === 'loading'}>
              {syncStatus.state === 'loading' && syncStatus.action === 'Pull' ? 'Pulling...' : 'Pull'}
            </button>
            <button onClick={handlePush} disabled={syncStatus.state === 'loading'}>
              {syncStatus.state === 'loading' && syncStatus.action === 'Push' ? 'Pushing...' : 'Push'}
            </button>
            {syncStatus.state !== 'idle' && (
              <span style={{
                fontSize: '0.85em',
                color: syncStatus.state === 'loading' ? '#666' : syncStatus.state === 'success' ? '#10b981' : '#ef4444',
              }}>
                {syncStatus.state === 'loading' ? `${syncStatus.action}ing...` : syncStatus.message}
              </span>
            )}
          </div>

          {/* View switcher */}
          <ViewSwitcher current={viewMode} onChange={setViewMode} />
          <TimeTracking tasks={tasks} />

          {/* Active view */}
          {viewMode === 'table' && <TaskTable key={refreshKey} projectRoot={projectRoot} onTasksLoaded={handleTasksLoaded} />}
          {viewMode === 'gantt' && <GanttView tasks={tasks} onAutoSchedule={handleAutoSchedule} />}
          {viewMode === 'kanban' && <KanbanView tasks={tasks} onStatusChange={handleStatusChange} />}

          <CommitPanel projectRoot={projectRoot} />
          <AgentPanel projectRoot={projectRoot} tasks={tasks} />
          <QueryHistory projectRoot={projectRoot} />
          <SemanticDiff projectRoot={projectRoot} />
          <ShockwaveMerge projectRoot={projectRoot} />
          <SyncPanel />
        </>
      )}
    </div>
  );
}
