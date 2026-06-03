/**
 * AiModeStatus — v1.1.0 product-readiness AI mode indicator.
 *
 * Shows credential state, agent paths, and review-only badge.
 * Clearly communicates that agents are review-only, not autonomous.
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface AiModeState {
  credentialProvider: string;
  hasToken: boolean;
  agentsAvailable: string[];
  mode: 'review-only' | 'unavailable';
}

interface Props {
  projectRoot: string;
}

export const AiModeStatus: React.FC<Props> = ({ projectRoot }) => {
  const [state, setState] = useState<AiModeState>({
    credentialProvider: 'checking',
    hasToken: false,
    agentsAvailable: [],
    mode: 'unavailable',
  });

  useEffect(() => {
    checkAiMode();
  }, [projectRoot]);

  const checkAiMode = async () => {
    try {
      const result = await invoke<string>('get_github_token_status_cmd', { projectRoot });
      const parsed = JSON.parse(result);
      setState({
        credentialProvider: parsed.provider || 'unknown',
        hasToken: parsed.exists || false,
        agentsAvailable: parsed.exists ? ['docs-agent', 'bugfix-agent'] : [],
        mode: parsed.exists ? 'review-only' : 'unavailable',
      });
    } catch {
      setState(prev => ({ ...prev, mode: 'unavailable' }));
    }
  };

  return (
    <div className="border rounded-lg p-3 mb-4 bg-gray-50">
      <div className="flex items-center gap-2 mb-1">
        <span className="font-semibold text-sm">AI Mode</span>
        {state.mode === 'review-only' ? (
          <span className="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded-full">
            review-only
          </span>
        ) : (
          <span className="text-xs bg-gray-200 text-gray-500 px-2 py-0.5 rounded-full">
            no-key mode
          </span>
        )}
      </div>
      <div className="text-xs text-gray-500 space-y-0.5">
        <div>Provider: {state.credentialProvider}</div>
        <div>Agents: {state.agentsAvailable.length > 0 ? state.agentsAvailable.join(', ') : 'none configured'}</div>
        {state.hasToken && (
          <div className="text-green-600">GitHub token stored</div>
        )}
        {!state.hasToken && (
          <div className="text-gray-400">No GitHub token — basic mode only</div>
        )}
      </div>
      <div className="text-xs text-gray-400 mt-1">
        Agents propose changes for human review. No autonomous commits.
      </div>
    </div>
  );
};
