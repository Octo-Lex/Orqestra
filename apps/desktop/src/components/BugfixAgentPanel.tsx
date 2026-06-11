/**
 * BugfixAgentPanel — v2.14.4 bugfix-agent review-only execution panel.
 *
 * Flow:
 * 1. User selects bug-labeled task
 * 2. User enters relative file paths to analyze (at least one required)
 * 3. run_bugfix_agent_cmd invokes AI service with selected files
 * 4. AI returns diagnosis + proposed edits
 * 5. DiffReviewPanel shows reviewable diff
 * 6. Accept/reject are review-only labels (no file mutation)
 *
 * Key constraint: auto_commit is ALWAYS disabled for bugfix-agent.
 */

import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DiffReviewPanel, AgentEditResponse } from './DiffReviewPanel';

interface Props {
  projectRoot: string;
  task: {
    id: string;
    title: string;
    labels: string[];
    source_path?: string;
  } | null;
}

export const BugfixAgentPanel: React.FC<Props> = ({ projectRoot, task }) => {
  const [result, setResult] = useState<AgentEditResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filePathInput, setFilePathInput] = useState('');
  const [status, setStatus] = useState<'idle' | 'running' | 'review' | 'accepted' | 'rejected'>('idle');

  if (!task) {
    return (
      <div style={{ fontSize: '0.875rem', color: '#6b7280', padding: '0.75rem' }}>
        No bug-labeled task found. Add a <code>bugfix</code> or <code>bug</code> label to a task to enable the bugfix agent.
      </div>
    );
  }

  // Parse newline-separated relative paths, trimmed, non-empty
  const parsedFiles = filePathInput
    .split('\n')
    .map(p => p.trim())
    .filter(p => p.length > 0);

  const hasFiles = parsedFiles.length > 0;

  const runAgent = async () => {
    if (!hasFiles) {
      setError('Enter at least one file path to analyze.');
      return;
    }

    setLoading(true);
    setError(null);
    setStatus('running');

    try {
      const taskJson = JSON.stringify({
        id: task.id,
        title: task.title,
        labels: task.labels,
        source_path: task.source_path,
      });

      // Validate paths: reject absolute, traversal, and escape patterns
      const invalidPaths = parsedFiles.filter(p => {
        // Reject absolute paths (Windows or Unix)
        if (/^[A-Za-z]:/.test(p) || p.startsWith('/') || p.startsWith('\\')) return true;
        // Reject home directory
        if (p.startsWith('~')) return true;
        // Reject path traversal
        if (p.includes('..')) return true;
        // Reject backslashes
        if (p.includes('\\')) return true;
        // Reject bare dot
        if (p === '.' || p === './') return true;
        return false;
      });

      if (invalidPaths.length > 0) {
        setError(`Invalid path(s): ${invalidPaths.join(', ')}. Use relative paths without .. or absolute prefixes.`);
        return;
      }

      // Read file contents — fail if any file cannot be read
      const filesWithContext: { path: string; content: string }[] = [];
      for (const path of parsedFiles) {
        try {
          const content = await invoke<string>('read_file_cmd', {
            projectRoot,
            path: `${projectRoot}/${path}`,
          });
          filesWithContext.push({ path, content });
        } catch {
          setError(`Cannot read file: ${path}. Ensure the path is correct and the file exists.`);
          setLoading(false);
          setStatus('idle');
          return;
        }
      }

      const filesJson = JSON.stringify(filesWithContext);

      const response = await invoke<string>('run_bugfix_agent_cmd', {
        projectRoot,
        task: taskJson,
        allowedFiles: filesJson,
      });

      const parsed = JSON.parse(response) as AgentEditResponse;
      setResult(parsed);
      setStatus('review');
    } catch (e: any) {
      setError(String(e));
      setStatus('idle');
    } finally {
      setLoading(false);
    }
  };

  const handleAccept = () => {
    setStatus('accepted');
  };

  const handleReject = () => {
    setResult(null);
    setStatus('rejected');
  };

  return (
    <div style={{ border: '1px solid #e5e7eb', borderRadius: '0.5rem', padding: '1rem', marginBottom: '1rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.75rem' }}>
        <span style={{ fontWeight: 600 }}>Task: {task.title} ({task.id})</span>
        <span style={{ fontSize: '0.75rem', backgroundColor: '#fef3c7', color: '#92400e', padding: '0.125rem 0.5rem', borderRadius: '9999px' }}>
          review-only
        </span>
      </div>

      {/* File path input */}
      {status === 'idle' && (
        <div style={{ marginBottom: '0.75rem' }}>
          <label style={{ display: 'block', fontSize: '0.875rem', fontWeight: 500, marginBottom: '0.25rem', color: '#374151' }}>
            File paths to analyze (one per line, relative to project root)
          </label>
          <textarea
            value={filePathInput}
            onChange={e => setFilePathInput(e.target.value)}
            placeholder={"src/lib/handler.rs\nsrc/lib/error.rs"}
            rows={3}
            style={{
              width: '100%',
              padding: '0.5rem',
              border: '1px solid #d1d5db',
              borderRadius: '0.375rem',
              fontFamily: 'monospace',
              fontSize: '0.875rem',
              resize: 'vertical',
            }}
          />
          {!hasFiles && filePathInput.length > 0 && (
            <div style={{ fontSize: '0.75rem', color: '#ef4444', marginTop: '0.25rem' }}>
              Enter at least one valid file path.
            </div>
          )}
          <button
            onClick={runAgent}
            disabled={!hasFiles || loading}
            style={{
              marginTop: '0.5rem',
              padding: '0.5rem 1rem',
              backgroundColor: hasFiles ? '#3b82f6' : '#9ca3af',
              color: 'white',
              border: 'none',
              borderRadius: '0.375rem',
              cursor: hasFiles ? 'pointer' : 'not-allowed',
              fontWeight: 600,
              fontSize: '0.875rem',
            }}
          >
            {loading ? 'Running...' : 'Run Bugfix Agent'}
          </button>
        </div>
      )}

      {status === 'running' && (
        <div style={{ fontSize: '0.875rem', color: '#6b7280' }}>
          Agent is analyzing selected files...
        </div>
      )}

      {status === 'review' && (
        <div>
          <DiffReviewPanel
            result={result}
            loading={loading}
            error={error}
            onAccept={handleAccept}
            onReject={handleReject}
          />
        </div>
      )}

      {status === 'accepted' && (
        <div style={{ fontSize: '0.875rem', color: '#16a34a' }}>
          Proposal accepted (review-only). No files were changed.
        </div>
      )}

      {status === 'rejected' && (
        <div style={{ fontSize: '0.875rem', color: '#6b7280' }}>
          Proposal rejected.
          <button
            onClick={() => setStatus('idle')}
            style={{ marginLeft: '0.5rem', color: '#3b82f6', background: 'none', border: 'none', cursor: 'pointer', textDecoration: 'underline' }}
          >
            Retry
          </button>
        </div>
      )}

      {error && (
        <div style={{ fontSize: '0.875rem', color: '#ef4444', marginTop: '0.5rem' }}>
          Error: {error}
        </div>
      )}

      <div style={{ fontSize: '0.75rem', color: '#9ca3af', marginTop: '0.5rem' }}>
        Bugfix agent is review-only. No autonomous commits. Accept/reject are labels only.
      </div>
    </div>
  );
};
