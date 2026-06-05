/**
 * FirstRunGuide — v2.0.0 beta hardening checklist.
 *
 * 10 non-mutating environment checks:
 *   1. Git available
 *   2. Repository selectable
 *   3. Roadmap valid
 *   4. AI service reachable (optional/degraded)
 *   5. Credential provider available
 *   6. Dashboard export status visible
 *   7. Agent endpoints available (optional/degraded)
 *   8. Patch governance enabled
 *   9. Code intelligence enabled
 *  10. Git provider resolved
 *
 * All checks are read-only probes. No agent runs, no patch applications,
 * no audit writes, no .Orqestra mutations, no arbitrary source parsing.
 *
 * AI service and agent endpoints are optional/degraded —
 * unavailability does not fail setup.
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export type CheckStatus =
  | 'complete'      // probe succeeded
  | 'pending'       // not yet checked
  | 'failed'        // probe returned error
  | 'optional'      // feature is optional, not configured
  | 'degraded'      // feature unavailable but app still functional
  | 'configured'    // optional feature is configured
  | 'missing-key';  // credential not set

export interface ChecklistItem {
  id: string;
  label: string;
  description: string;
  status: CheckStatus;
  action?: string;
}

interface Props {
  projectRoot: string | null;
  onDismiss: () => void;
}

export const FirstRunGuide: React.FC<Props> = ({ projectRoot, onDismiss }) => {
  const [items, setItems] = useState<ChecklistItem[]>([]);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    loadChecklist();
  }, [projectRoot]);

  const loadChecklist = async () => {
    const checklist: ChecklistItem[] = [
      {
        id: 'git-available',
        label: 'Git available',
        description: 'Git CLI detected on PATH',
        status: 'pending',
      },
      {
        id: 'repo-selectable',
        label: 'Repository selectable',
        description: 'A Git repository is open',
        status: projectRoot ? 'complete' : 'pending',
      },
      {
        id: 'roadmap-valid',
        label: 'Roadmap valid',
        description: 'roadmap/_index.md is parseable',
        status: 'pending',
      },
      {
        id: 'ai-service',
        label: 'AI service reachable',
        description: 'Optional — AI features require the Python AI service at localhost:8000',
        status: 'optional',
      },
      {
        id: 'credential-provider',
        label: 'Credential provider available',
        description: 'OS keychain accessible for secure storage',
        status: 'pending',
      },
      {
        id: 'dashboard-status',
        label: 'Dashboard export visible',
        description: 'Dashboard data can be generated',
        status: 'pending',
      },
      {
        id: 'agent-endpoints',
        label: 'Agent endpoints available',
        description: 'Optional — docs, bugfix, and architect agent endpoints',
        status: 'optional',
      },
      {
        id: 'patch-governance',
        label: 'Patch governance enabled',
        description: 'Agent patch audit trail is active',
        status: 'pending',
      },
      {
        id: 'code-intel',
        label: 'Code intelligence enabled',
        description: 'Tree-sitter symbol extraction is available',
        status: 'pending',
      },
      {
        id: 'git-provider',
        label: 'Git provider resolved',
        description: 'Native Git provider (gix/gix-hybrid/CLI) determined',
        status: 'pending',
      },
    ];

    // Run all probes (non-mutating, read-only)

    // 1. Git available
    try {
      await invoke<string>('check_git_available_cmd', {});
      const gitItem = checklist.find(i => i.id === 'git-available');
      if (gitItem) gitItem.status = 'complete';
    } catch {
      const gitItem = checklist.find(i => i.id === 'git-available');
      if (gitItem) gitItem.status = 'failed';
    }

    // 2. Repo selectable — already set above from projectRoot

    // 3. Roadmap valid (bounded read)
    if (projectRoot) {
      try {
        const roadmapStatus = await invoke<string>('check_roadmap_valid_cmd', { projectRoot });
        const parsed = JSON.parse(roadmapStatus);
        const roadmapItem = checklist.find(i => i.id === 'roadmap-valid');
        if (roadmapItem) roadmapItem.status = parsed.valid ? 'complete' : 'failed';
      } catch {
        const roadmapItem = checklist.find(i => i.id === 'roadmap-valid');
        if (roadmapItem) roadmapItem.status = 'failed';
      }
    }

    // 4. AI service reachable — optional/degraded on failure
    try {
      await invoke<string>('check_ai_service_cmd', {});
      const aiItem = checklist.find(i => i.id === 'ai-service');
      if (aiItem) aiItem.status = 'complete';
    } catch {
      const aiItem = checklist.find(i => i.id === 'ai-service');
      if (aiItem) aiItem.status = 'degraded';
    }

    // 5. Credential provider available
    try {
      const credStatus = await invoke<string>('check_credential_provider_cmd', { projectRoot: projectRoot || '' });
      const parsed = JSON.parse(credStatus);
      const credItem = checklist.find(i => i.id === 'credential-provider');
      if (credItem) credItem.status = parsed.available ? 'complete' : 'failed';
    } catch {
      const credItem = checklist.find(i => i.id === 'credential-provider');
      if (credItem) credItem.status = 'failed';
    }

    // 6. Dashboard status
    try {
      const dashStatus = await invoke<string>('check_dashboard_status_cmd', { projectRoot: projectRoot || '' });
      const parsed = JSON.parse(dashStatus);
      const dashItem = checklist.find(i => i.id === 'dashboard-status');
      if (dashItem) dashItem.status = parsed.available ? 'complete' : 'optional';
    } catch {
      const dashItem = checklist.find(i => i.id === 'dashboard-status');
      if (dashItem) dashItem.status = 'optional';
    }

    // 7. Agent endpoints — optional/degraded on failure
    try {
      await invoke<string>('check_agent_endpoints_cmd', {});
      const agentItem = checklist.find(i => i.id === 'agent-endpoints');
      if (agentItem) agentItem.status = 'complete';
    } catch {
      const agentItem = checklist.find(i => i.id === 'agent-endpoints');
      if (agentItem) agentItem.status = 'degraded';
    }

    // 8. Patch governance enabled
    try {
      const pgStatus = await invoke<string>('check_patch_governance_cmd', { projectRoot: projectRoot || '' });
      const parsed = JSON.parse(pgStatus);
      const pgItem = checklist.find(i => i.id === 'patch-governance');
      if (pgItem) pgItem.status = parsed.enabled ? 'complete' : 'failed';
    } catch {
      const pgItem = checklist.find(i => i.id === 'patch-governance');
      if (pgItem) pgItem.status = 'pending';
    }

    // 9. Code intelligence enabled
    try {
      const ciStatus = await invoke<string>('check_code_intel_cmd', { projectRoot: projectRoot || '' });
      const parsed = JSON.parse(ciStatus);
      const ciItem = checklist.find(i => i.id === 'code-intel');
      if (ciItem) ciItem.status = parsed.available ? 'complete' : 'failed';
    } catch {
      const ciItem = checklist.find(i => i.id === 'code-intel');
      if (ciItem) ciItem.status = 'pending';
    }

    // 10. Git provider resolved
    if (projectRoot) {
      try {
        const providerStatus = await invoke<string>('check_git_provider_cmd', { projectRoot });
        const parsed = JSON.parse(providerStatus);
        const provItem = checklist.find(i => i.id === 'git-provider');
        if (provItem) provItem.status = parsed.resolved ? 'complete' : 'failed';
      } catch {
        const provItem = checklist.find(i => i.id === 'git-provider');
        if (provItem) provItem.status = 'failed';
      }
    }

    setItems(checklist);
  };

  const handleDismiss = () => {
    setDismissed(true);
    onDismiss();
  };

  if (dismissed) return null;

  const allComplete = items.every(
    i => i.status === 'complete' || i.status === 'optional' || i.status === 'configured' || i.status === 'degraded'
  );

  const statusIcon = (status: CheckStatus) => {
    switch (status) {
      case 'complete': return '✓';
      case 'configured': return '✓';
      case 'pending': return '○';
      case 'failed': return '✗';
      case 'optional': return '○';
      case 'missing-key': return '!';
      case 'degraded': return '⚠';
    }
  };

  const statusClass = (status: CheckStatus) => {
    switch (status) {
      case 'complete':
      case 'configured': return 'text-green-600';
      case 'pending':
      case 'optional': return 'text-gray-400';
      case 'failed': return 'text-red-500';
      case 'missing-key': return 'text-yellow-500';
      case 'degraded': return 'text-amber-500';
    }
  };

  const statusBadge = (status: CheckStatus) => {
    switch (status) {
      case 'optional':
        return <span className="text-xs bg-gray-100 text-gray-500 px-1 rounded">optional</span>;
      case 'degraded':
        return <span className="text-xs bg-amber-50 text-amber-600 px-1 rounded">degraded — AI features unavailable</span>;
      case 'configured':
        return <span className="text-xs bg-green-50 text-green-600 px-1 rounded">configured</span>;
      default:
        return null;
    }
  };

  return (
    <div className="border rounded-lg p-4 mb-4 bg-white shadow-sm">
      <div className="flex justify-between items-center mb-3">
        <h3 className="text-lg font-semibold">Environment Checks</h3>
        <button
          onClick={handleDismiss}
          className="text-sm text-gray-400 hover:text-gray-600"
        >
          Dismiss
        </button>
      </div>
      <ul className="space-y-2">
        {items.map(item => (
          <li key={item.id} className="flex items-start gap-2">
            <span className={`font-mono ${statusClass(item.status)}`}>
              {statusIcon(item.status)}
            </span>
            <div className="flex-1">
              <div className="flex items-center gap-2">
                <span className={item.status === 'complete' || item.status === 'configured' ? 'line-through text-gray-500' : ''}>
                  {item.label}
                </span>
                {statusBadge(item.status)}
              </div>
              <div className="text-xs text-gray-400">{item.description}</div>
            </div>
          </li>
        ))}
      </ul>
      {allComplete && (
        <p className="mt-3 text-sm text-green-600">
          Environment checks complete. Orqestra is ready.
        </p>
      )}
      <p className="mt-2 text-xs text-gray-400">
        Governed AI-native development beta — agents operate in review-only mode. No autonomous commits.
      </p>
    </div>
  );
};
