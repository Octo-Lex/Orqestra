import { useState, useEffect } from 'react';
import { TaskTable } from './components/TaskTable';
import { open } from '@tauri-apps/plugin-dialog';
import {
  storePat,
  hasStoredPat,
  gitPullRoadmap,
  gitPushRoadmap,
  type GitResult,
} from './lib/git';

type SyncStatus =
  | { state: 'idle' }
  | { state: 'loading'; action: string }
  | { state: 'success'; action: string; message: string }
  | { state: 'error'; action: string; message: string };

export default function App() {
  const [projectRoot, setProjectRoot] = useState<string | null>(null);
  const [syncStatus, setSyncStatus] = useState<SyncStatus>({ state: 'idle' });
  const [showSettings, setShowSettings] = useState(false);
  const [patInput, setPatInput] = useState('');
  const [patSaved, setPatSaved] = useState(false);
  const [patError, setPatError] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  // Check if PAT is already stored on mount
  useEffect(() => {
    hasStoredPat()
      .then(saved => setPatSaved(saved))
      .catch(() => setPatSaved(false));
  }, []);

  async function openProject() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string') setProjectRoot(selected);
  }

  async function handleSavePat() {
    setPatError(null);
    try {
      await storePat(patInput);
      setPatSaved(true);
      setPatInput('');
    } catch (e) {
      setPatError(e instanceof Error ? e.message : String(e));
    }
  }

  async function handlePull() {
    if (!projectRoot) return;
    setSyncStatus({ state: 'loading', action: 'Pull' });
    try {
      const result: GitResult = await gitPullRoadmap(projectRoot);
      if (result.success) {
        setSyncStatus({
          state: 'success',
          action: 'Pull',
          message: result.stdout.trim() || 'Up to date.',
        });
        setRefreshKey(k => k + 1); // trigger table re-render
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
    if (!projectRoot) return;
    setSyncStatus({ state: 'loading', action: 'Push' });
    try {
      const result: GitResult = await gitPushRoadmap(projectRoot);
      if (result.success) {
        setSyncStatus({
          state: 'success',
          action: 'Push',
          message: result.stdout.trim() || 'Pushed successfully.',
        });
      } else {
        // "nothing to commit" is okay
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
                  placeholder={patSaved ? '•••••••• (saved)' : 'ghp_xxxxxxxxxxxx'}
                  style={{ flexGrow: 1, padding: '0.25rem 0.5rem' }}
                />
                <button onClick={handleSavePat} disabled={!patInput.trim()}>
                  Save
                </button>
              </div>
              {patError && <div style={{ color: 'red', marginTop: '0.25rem' }}>{patError}</div>}
              {patSaved && !patError && (
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

          {/* Task table */}
          <TaskTable key={refreshKey} projectRoot={projectRoot} />
        </>
      )}
    </div>
  );
}
