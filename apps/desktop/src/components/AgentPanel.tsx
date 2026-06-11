import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ROUTING_RULES } from '../agent/AgentRouter';
import type { AgentResult } from '../agent/AgentWorkspace';
import type { Task } from '../lib/orqestra';
import { DiffReviewPanel, type AgentEditResponse } from './DiffReviewPanel';
import { BugfixAgentPanel } from './BugfixAgentPanel';
import { ArchitectAgentPanel } from './ArchitectAgentPanel';

interface AgentPanelProps {
  projectRoot: string;
  tasks: Task[];
}

interface WorkspaceCard {
  dir: string;
  id: string;
  status: 'idle' | 'running' | 'done' | 'error' | 'unavailable';
  result: AgentResult | null;
  log: string[];
}

export function AgentPanel({ projectRoot, tasks }: AgentPanelProps) {
  const [workspaces, setWorkspaces] = useState<WorkspaceCard[]>([]);
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

  const statusColors: Record<string, string> = {
    idle: '#9ca3af',
    running: '#f59e0b',
    done: '#10b981',
    error: '#ef4444',
    unavailable: '#6b7280',
  };

  const totalCommits = workspaces.filter(ws => ws.result?.commitHash).length;

  return (
    <div style={{ marginTop: '1.5rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '0.75rem' }}>
        <h3 style={{ margin: 0 }}>Agent Workspaces</h3>
        <span style={{ fontSize: '0.85em', color: '#6b7280' }}>
          Individual agents call real AI service. No fabricated results.
        </span>
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

      {/* Bugfix agent (real execution) */}
      <div style={{ marginTop: '1rem', marginBottom: '1rem' }}>
        <h4 style={{ margin: 0, marginBottom: '0.5rem' }}>Bugfix Agent</h4>
        <BugfixAgentPanel projectRoot={projectRoot} task={((): { id: string; title: string; labels: string[]; source_path?: string } | null => {
          const t = tasks.find(t => t.frontmatter.labels?.some(l => l === 'bugfix' || l === 'bug'));
          return t ? { id: t.frontmatter.id, title: t.frontmatter.title, labels: t.frontmatter.labels ?? [], source_path: t.source_path } : null;
        })()} />
      </div>

      {/* Architect agent (real execution) */}
      <div style={{ marginTop: '1rem', marginBottom: '1rem' }}>
        <h4 style={{ margin: 0, marginBottom: '0.5rem' }}>Architect Agent</h4>
        <ArchitectAgentPanel projectRoot={projectRoot} task={((): { id: string; title: string; labels: string[]; source_path?: string } | null => {
          const t = tasks.find(t => t.frontmatter.labels?.some(l => l === 'architect' || l === 'architecture'));
          return t ? { id: t.frontmatter.id, title: t.frontmatter.title, labels: t.frontmatter.labels ?? [], source_path: t.source_path } : null;
        })()} />
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
