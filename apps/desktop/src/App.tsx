import { useState, useEffect } from 'react';
import { TaskTable } from './components/TaskTable';
import { CommitPanel } from './components/CommitPanel';
import { GanttView } from './pm/GanttView';
import { KanbanView } from './pm/KanbanView';
import { TimeTracking } from './pm/TimeTracking';
import { ViewSwitcher, type ViewMode } from './pm/ViewSwitcher';
import { open } from '@tauri-apps/plugin-dialog';
import {
  persistPat,
  loadPersistedPat,
  gitPullRoadmap,
  gitPushRoadmap,
  type GitResult,
} from './lib/git';
import {
  updateTaskStatus,
  type Task,
  type TaskStatus,
} from './lib/orqestra';

type SyncStatus =
  | { state: 'idle' }
  | { state: 'loading'; action: string }
  | { state: 'success'; action: string; message: string }
  | { state: 'error'; action: string; message: string };

export default function App() {
  const [projectRoot, setProjectRoot] = useState<string | null>(null);
  const [syncStatus, setSyncStatus] = useState<SyncStatus>({ state: 'idle' });
  const [showSettings, setShowSettings] = useState(false);
  const [viewMode, setViewMode] = useState<ViewMode>('table');

  // PAT lives in React state. Persisted to disk for cross-session survival,
  // but never read back from the store within the same session.
  const [pat, setPat] = useState<string | null>(null);
  const [patInput, setPatInput] = useState('');
  const [patError, setPatError] = useState<string | null>(null);

  // Shared task data for all views
  const [tasks, setTasks] = useState<Task[]>([]);
  const [refreshKey, setRefreshKey] = useState(0);

  // On mount, try to load persisted PAT from disk into state
  useEffect(() => {
    loadPersistedPat()
      .then(p => {
        if (p) setPat(p);
      })
      .catch(() => {});
  }, []);

  async function openProject() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string') setProjectRoot(selected);
  }

  async function handleSavePat() {
    setPatError(null);
    try {
      await persistPat(patInput);
      setPat(patInput);
      setPatInput('');
    } catch (e) {
      setPatError(e instanceof Error ? e.message : String(e));
    }
  }

  async function handlePull() {
    if (!projectRoot || !pat) {
      setSyncStatus({
        state: 'error',
        action: 'Pull',
        message: 'GitHub PAT not configured. Save it in Settings.',
      });
      return;
    }
    setSyncStatus({ state: 'loading', action: 'Pull' });

    try {
      const result: GitResult = await gitPullRoadmap(projectRoot, pat);
      if (result.success) {
        setSyncStatus({
          state: 'success',
          action: 'Pull',
          message: result.stdout.trim() || 'Up to date.',
        });
        setRefreshKey(k => k + 1);
      } else {
        setSyncStatus({
          state: 'error',
          action: 'Pull',
          message: result.stderr.trim() || result.stdout.trim() || 'Unknown error.',
        });
      }
    } catch (e) {
      setSyncStatus({
        state: 'error',
        action: 'Pull',
        message: e instanceof Error ? e.message : String(e),
      });
    }
  }

  async function handlePush() {
    if (!projectRoot || !pat) {
      setSyncStatus({
        state: 'error',
        action: 'Push',
        message: 'GitHub PAT not configured. Save it in Settings.',
      });
      return;
    }
    setSyncStatus({ state: 'loading', action: 'Push' });

    try {
      const result: GitResult = await gitPushRoadmap(projectRoot, pat);
      if (result.success) {
        setSyncStatus({
          state: 'success',
          action: 'Push',
          message: result.stdout.trim() || 'Pushed successfully.',
        });
      } else {
        const combined = result.stderr.trim() || result.stdout.trim();
        if (combined.includes('nothing to commit')) {
          setSyncStatus({
            state: 'success',
            action: 'Push',
            message: 'Nothing to commit — already up to date.',
          });
        } else {
          setSyncStatus({
            state: 'error',
            action: 'Push',
            message: combined || 'Unknown error.',
          });
        }
      }
    } catch (e) {
      setSyncStatus({
        state: 'error',
        action: 'Push',
        message: e instanceof Error ? e.message : String(e),
      });
    }
  }

  // Kanban drag handler: update task status via Rust backend
  async function handleStatusChange(taskId: string, newStatus: TaskStatus) {
    if (!projectRoot) return;
    try {
      await updateTaskStatus(projectRoot, taskId, newStatus);
      // Refresh to pick up the change
      setRefreshKey(k => k + 1);
    } catch (e) {
      console.error('Failed to update task status:', e);
    }
  }

  // Auto-schedule: placeholder — in Phase 3 this will write back dates
  function handleAutoSchedule() {
    alert('Auto-Schedule applied. Date adjustments calculated.\n(Write-back to .md files requires Phase 3 commit integration.)');
  }

  // Callback when TaskTable loads tasks — share with other views
  function handleTasksLoaded(loadedTasks: Task[]) {
    setTasks(loadedTasks);
  }

  return (
    <div style={{ padding: '1rem', fontFamily: 'system-ui, sans-serif' }}>
      {!projectRoot ? (
        <button onClick={openProject}>Open project folder</button>
      ) : (
        <>
          {/* Project path bar */}
          <div style={{ marginBottom: '0.5rem', color: '#666', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <code>{projectRoot}</code>
            <button onClick={() => { setProjectRoot(null); setSyncStatus({ state: 'idle' }); }}>
              Close
            </button>
            <button onClick={() => setShowSettings(s => !s)} style={{ marginLeft: 'auto' }}>
              {showSettings ? 'Hide Settings' : 'Settings'}
            </button>
          </div>

          {/* Settings panel */}
          {showSettings && (
            <div style={{ marginBottom: '1rem', padding: '0.75rem', border: '1px solid #ddd', borderRadius: '4px', background: '#fafafa' }}>
              <div style={{ marginBottom: '0.5rem', fontWeight: 'bold' }}>GitHub PAT</div>
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                <input
                  type="password"
                  value={patInput}
                  onChange={e => setPatInput(e.target.value)}
                  placeholder={pat ? '•••••••• (saved — leave blank to keep)' : 'ghp_xxxxxxxxxxxx'}
                  style={{ flexGrow: 1, padding: '0.25rem 0.5rem' }}
                />
                <button onClick={handleSavePat} disabled={!patInput.trim()}>
                  Save
                </button>
              </div>
              {patError && <div style={{ color: 'red', marginTop: '0.25rem' }}>{patError}</div>}
              {pat && !patError && (
                <div style={{ color: 'green', marginTop: '0.25rem', fontSize: '0.85em' }}>
                  PAT stored{patInput ? ' (updated)' : ''}.
                </div>
              )}
              <div style={{ marginTop: '0.25rem', fontSize: '0.75em', color: '#999' }}>
                Stored locally via tauri-plugin-store. Not hardware-encrypted.
              </div>
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
              <span
                style={{
                  fontSize: '0.85em',
                  color:
                    syncStatus.state === 'loading'
                      ? '#666'
                      : syncStatus.state === 'success'
                        ? '#10b981'
                        : '#ef4444',
                }}
              >
                {syncStatus.state === 'loading'
                  ? `${syncStatus.action}ing...`
                  : syncStatus.message}
              </span>
            )}
          </div>

          {/* View switcher */}
          <ViewSwitcher current={viewMode} onChange={setViewMode} />

          {/* Time tracking — always visible */}
          <TimeTracking tasks={tasks} />

          {/* Active view */}
          {viewMode === 'table' && (
            <TaskTable
              key={refreshKey}
              projectRoot={projectRoot}
              onTasksLoaded={handleTasksLoaded}
            />
          )}
          {viewMode === 'gantt' && (
            <GanttView tasks={tasks} onAutoSchedule={handleAutoSchedule} />
          )}
          {viewMode === 'kanban' && (
            <KanbanView tasks={tasks} onStatusChange={handleStatusChange} />
          )}

          {/* Semantic commit panel */}
          <CommitPanel projectRoot={projectRoot} />
        </>
      )}
    </div>
  );
}
