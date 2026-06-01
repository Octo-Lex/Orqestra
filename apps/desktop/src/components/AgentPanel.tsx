import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AgentRouter, ROUTING_RULES } from '../agent/AgentRouter';
import type { AgentResult } from '../agent/AgentWorkspace';
import type { Task } from '../lib/orqestra';

interface AgentPanelProps {
  projectRoot: string;
  tasks: Task[];
}

interface WorkspaceCard {
  dir: string;
  id: string;
  status: 'idle' | 'running' | 'done' | 'error';
  result: AgentResult | null;
  log: string[];
}

export function AgentPanel({ projectRoot, tasks }: AgentPanelProps) {
  const [workspaces, setWorkspaces] = useState<WorkspaceCard[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [globalLog, setGlobalLog] = useState<string[]>([]);

  // Load workspace list on mount
  useEffect(() => {
    loadWorkspaces();
  }, [projectRoot]);

  async function loadWorkspaces() {
    try {
      const entries = await invoke<{ dir: string; id: string }[]>('list_workspaces_cmd', {
        projectRoot,
      });
      setWorkspaces(
        entries.map(e => ({
          dir: e.dir,
          id: e.id,
          status: 'idle',
          result: null,
          log: [],
        })),
      );
    } catch (e) {
      console.error('Failed to load workspaces:', e);
    }
  }

  const addLog = useCallback((wsId: string, message: string) => {
    const ts = new Date().toLocaleTimeString();
    setGlobalLog(prev => [...prev, `[${ts}] [${wsId}] ${message}`]);
    setWorkspaces(prev =>
      prev.map(ws =>
        ws.id === wsId
          ? { ...ws, log: [...ws.log, `[${ts}] ${message}`].slice(-20) }
          : ws,
      ),
    );
  }, []);

  async function runAllAgents() {
    setIsRunning(true);
    setGlobalLog([]);

    try {
      const router = new AgentRouter(projectRoot);

      // Route each task to its workspace
      const routedTasks = new Map<string, { task: Task; dir: string }>();
      for (const task of tasks) {
        try {
          const { workspaceDir } = await router.route(task);
          if (!routedTasks.has(workspaceDir)) {
            routedTasks.set(workspaceDir, { task, dir: workspaceDir });
          }
        } catch (e) {
          console.warn(`Failed to route task ${task.frontmatter.id}:`, e);
        }
      }

      if (routedTasks.size === 0) {
        addLog('system', 'No tasks could be routed to workspaces');
        setIsRunning(false);
        return;
      }

    // Mark all matched workspaces as running
    setWorkspaces(prev =>
      prev.map(ws => {
        const match = routedTasks.get(ws.dir);
        return match ? { ...ws, status: 'running' as const, log: [], result: null } : ws;
      }),
    );

    // Run all agents in parallel
    const promises = Array.from(routedTasks.entries()).map(
      async ([dir, { task }]) => {
        try {
          addLog(dir, `Routing task ${task.frontmatter.id} → ${dir}`);
          const result = await router.runTask(task);
          addLog(dir, `Done: confidence=${result.confidence}, gate=${result.gateAction}`);

          setWorkspaces(prev =>
            prev.map(ws => (ws.dir === dir ? { ...ws, status: 'done' as const, result } : ws)),
          );
          return result;
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          addLog(dir, `Error: ${msg}`);
          setWorkspaces(prev =>
            prev.map(ws => (ws.dir === dir ? { ...ws, status: 'error' as const } : ws)),
          );
          return null;
        }
      },
    );

    await Promise.all(promises);
    } catch (e) {
      console.error('runAllAgents failed:', e);
      addLog('system', `Fatal: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setIsRunning(false);
    }
  }

  const statusColors: Record<string, string> = {
    idle: '#9ca3af',
    running: '#f59e0b',
    done: '#10b981',
    error: '#ef4444',
  };

  const totalCommits = workspaces.filter(ws => ws.result?.commitHash).length;

  return (
    <div style={{ marginTop: '1.5rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '0.75rem' }}>
        <h3 style={{ margin: 0 }}>Agent Workspaces</h3>
        <button
          onClick={runAllAgents}
          disabled={isRunning || workspaces.length === 0}
          style={{
            padding: '0.4rem 1rem',
            background: isRunning ? '#fbbf24' : '#3b82f6',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: isRunning ? 'wait' : 'pointer',
            fontWeight: 600,
          }}
        >
          {isRunning ? 'Running...' : 'Run All Agents'}
        </button>
        {totalCommits > 0 && (
          <span style={{ color: '#10b981', fontSize: '0.85em', fontWeight: 600 }}>
            {totalCommits} commit{totalCommits !== 1 ? 's' : ''} produced
          </span>
        )}
      </div>

      {/* Routing rules legend */}
      <div style={{ fontSize: '0.75em', color: '#6b7280', marginBottom: '0.75rem' }}>
        Routing: {Object.entries(ROUTING_RULES).map(([label, ws]) => `${label}→${ws}`).join(' · ')}
      </div>

      {/* Workspace cards */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '0.75rem' }}>
        {workspaces.map(ws => (
          <div
            key={ws.id}
            style={{
              border: '1px solid #e5e7eb',
              borderRadius: '6px',
              padding: '0.75rem',
              background: '#fafafa',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' }}>
              <strong style={{ fontSize: '0.95em' }}>{ws.id}</strong>
              <span
                style={{
                  fontSize: '0.8em',
                  color: statusColors[ws.status],
                  fontWeight: 600,
                  textTransform: 'uppercase',
                }}
              >
                {ws.status}
              </span>
            </div>

            {ws.result && (
              <div style={{ fontSize: '0.85em', marginBottom: '0.5rem' }}>
                <div>
                  <span style={{ color: '#6b7280' }}>Task:</span>{' '}
                  <code>{ws.result.taskId}</code>
                </div>
                <div>
                  <span style={{ color: '#6b7280' }}>Summary:</span>{' '}
                  {ws.result.summary}
                </div>
                <div>
                  <span style={{ color: '#6b7280' }}>Confidence:</span>{' '}
                  <span style={{ fontWeight: 600, color: ws.result.confidence >= 0.9 ? '#10b981' : '#f59e0b' }}>
                    {(ws.result.confidence * 100).toFixed(1)}%
                  </span>
                </div>
                <div>
                  <span style={{ color: '#6b7280' }}>Gate:</span>{' '}
                  <span
                    style={{
                      fontWeight: 600,
                      color:
                        ws.result.gateAction === 'auto_commit'
                          ? '#10b981'
                          : ws.result.gateAction === 'propose'
                            ? '#f59e0b'
                            : '#ef4444',
                    }}
                  >
                    {ws.result.gateAction.replace('_', ' ')}
                  </span>
                </div>
                {ws.result.commitHash && (
                  <div>
                    <span style={{ color: '#6b7280' }}>Commit:</span>{' '}
                    <code style={{ fontSize: '0.85em' }}>{ws.result.commitHash}</code>
                  </div>
                )}
                {ws.result.changes.length > 0 && (
                  <div style={{ marginTop: '0.25rem' }}>
                    <span style={{ color: '#6b7280' }}>Files:</span>{' '}
                    {ws.result.changes.map(c => (
                      <code key={c.path} style={{ fontSize: '0.8em', marginRight: '0.5rem' }}>
                        {c.path}
                      </code>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Log output */}
            {ws.log.length > 0 && (
              <div
                style={{
                  fontSize: '0.75em',
                  fontFamily: 'monospace',
                  background: '#1f2937',
                  color: '#d1d5db',
                  padding: '0.5rem',
                  borderRadius: '3px',
                  maxHeight: '120px',
                  overflowY: 'auto',
                }}
              >
                {ws.log.map((line, i) => (
                  <div key={i}>{line}</div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Global log */}
      {globalLog.length > 0 && (
        <details style={{ marginTop: '1rem' }}>
          <summary style={{ cursor: 'pointer', fontSize: '0.85em', fontWeight: 600 }}>
            Execution Log ({globalLog.length} events)
          </summary>
          <div
            style={{
              fontSize: '0.75em',
              fontFamily: 'monospace',
              background: '#1f2937',
              color: '#d1d5db',
              padding: '0.75rem',
              borderRadius: '4px',
              maxHeight: '200px',
              overflowY: 'auto',
              marginTop: '0.5rem',
            }}
          >
            {globalLog.map((line, i) => (
              <div key={i}>{line}</div>
            ))}
          </div>
        </details>
      )}
    </div>
  );
}
