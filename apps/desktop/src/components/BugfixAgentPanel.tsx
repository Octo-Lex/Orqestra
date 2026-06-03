/**
 * BugfixAgentPanel — v1.1.0 bugfix-agent review-only execution panel.
 *
 * Flow:
 * 1. User selects bug-labeled task
 * 2. Agent router identifies bugfix workspace
 * 3. Desktop gathers task context and selected files
 * 4. run_bugfix_agent_cmd invokes AI service
 * 5. AI returns diagnosis + proposed edits
 * 6. DiffReviewPanel shows reviewable diff
 * 7. On accept: user commits through normal commit flow (NOT auto-commit)
 * 8. On reject: edits discarded
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
  const selectedFiles: string[] = [];
  const [status, setStatus] = useState<'idle' | 'running' | 'review' | 'accepted' | 'rejected'>('idle');

  if (!task) {
    return (
      <div className="text-sm text-gray-500 p-4">
        Select a bug-labeled task to run the bugfix agent.
      </div>
    );
  }

  const runAgent = async () => {
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

      const filesJson = JSON.stringify(
        selectedFiles.map(path => ({ path, content: '' }))
      );

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
    // v1.1.0: User must commit through normal commit flow.
    // This does NOT auto-commit. It records acceptance.
    setStatus('accepted');
  };

  const handleReject = () => {
    // Discard proposed edits
    setResult(null);
    setStatus('rejected');
  };

  return (
    <div className="border rounded-lg p-4 mb-4">
      <div className="flex items-center gap-2 mb-3">
        <h3 className="font-semibold">Bugfix Agent</h3>
        <span className="text-xs bg-yellow-100 text-yellow-700 px-2 py-0.5 rounded-full">
          review-only
        </span>
      </div>

      <div className="text-sm text-gray-600 mb-3">
        Task: {task.title} ({task.id})
      </div>

      {status === 'idle' && (
        <button
          onClick={runAgent}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
          disabled={loading}
        >
          Run Bugfix Agent
        </button>
      )}

      {status === 'running' && (
        <div className="text-sm text-gray-500">
          Agent is analyzing the task...
        </div>
      )}

      {status === 'review' && (
        <div>
          <div className="text-sm text-gray-500 mb-2">
            Review the proposed changes below. Accept to apply, reject to discard.
            Commits use the normal Git flow.
          </div>
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
        <div className="text-sm text-green-600">
          Changes accepted. Use the commit panel to commit.
        </div>
      )}

      {status === 'rejected' && (
        <div className="text-sm text-gray-500">
          Changes rejected.
          <button
            onClick={() => setStatus('idle')}
            className="ml-2 text-blue-500 hover:underline"
          >
            Retry
          </button>
        </div>
      )}

      {error && (
        <div className="text-sm text-red-500 mt-2">
          Error: {error}
        </div>
      )}

      <div className="text-xs text-gray-400 mt-2">
        Bugfix agent is review-only. No autonomous commits.
      </div>
    </div>
  );
};
