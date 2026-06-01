import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AgentRouter, ROUTING_RULES } from '../agent/AgentRouter';
import type { AgentResult } from '../agent/AgentWorkspace';
import type { Task } from '../lib/orqestra';
import { DiffReviewPanel, type AgentEditResponse } from './DiffReviewPanel';

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

  // Docs agent (real execution path)
  const [docsResult, setDocsResult] = useState<AgentEditResponse | null>(null);
  const [docsLoading, setDocsLoading] = useState(false);
  const [docsError, setDocsError] = useState<string | null>(null);

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

  async function runDocsAgent() {
    // Find the first docs-labeled task
    const docsTask = tasks.find(t =>
      t.frontmatter.labels.some(l => l === 'docs' || l === 'documentation' || l === 'readme')
    );
    if (!docsTask) {
      setDocsError('No docs-labeled task found');
      return;
    }

    setDocsLoading(true);
    setDocsError(null);
    setDocsResult(null);

    try {
      // Gather context files (README.md from project root)
      const contextFiles: { path: string; content: string }[] = [];
      try {
        const readme = await invoke<string>('read_file_cmd', { path: `${projectRoot}/README.md` });
        contextFiles.push({ path: 'README.md', content: readme });
      } catch { /* README not available */ }

      const resultStr = await invoke<string>('run_docs_agent_cmd', {
        projectRoot,
        task: JSON.stringify({
          id: docsTask.frontmatter.id,
          title: docsTask.frontmatter.title,
          body: docsTask.body?.raw || '',
          labels: docsTask.frontmatter.labels,
        }),
        contextFiles: JSON.stringify(contextFiles),
      });

      const result: AgentEditResponse = JSON.parse(resultStr);
      setDocsResult(result);
    } catch (e) {
      setDocsError(e instanceof Error ? e.message : String(e));
    } finally {
      setDocsLoading(false);
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

      {/* Docs agent (real execution) */}
      <div style={{ marginTop: '1rem', marginBottom: '1rem' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '0.5rem' }}>
          <h4 style={{ margin: 0 }}>Docs Agent (Real AI Execution)</h4>
          <button
            onClick={runDocsAgent}
            disabled={docsLoading}
            style={{
              padding: '0.3rem 0.75rem',
              background: '#8b5cf6',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: docsLoading ? 'wait' : 'pointer',
              fontWeight: 600,
              fontSize: '0.85em',
            }}
          >
            {docsLoading ? 'Running...' : 'Run Docs Agent'}
          </button>
          <span style={{ fontSize: '0.75em', color: '#6b7280' }}>
            Calls real AI service, no auto-commit
          </span>
        </div>
        <DiffReviewPanel
          result={docsResult}
          loading={docsLoading}
          error={docsError}
          onAccept={() => addLog('docs-agent', 'Change accepted by human')}
          onReject={() => addLog('docs-agent', 'Change rejected by human')}
        />
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
