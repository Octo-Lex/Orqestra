/**
 * FirstRunGuide — v1.1.0 product-readiness checklist.
 *
 * Guides users from launch to open repo to roadmap to no-key demo to optional AI mode.
 * Dismissible and reopenable. AI mode is clearly optional, not implied as autonomous.
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface ChecklistItem {
  id: string;
  label: string;
  status: 'complete' | 'pending' | 'failed' | 'optional' | 'configured' | 'missing-key' | 'unavailable';
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
        id: 'open-repo',
        label: 'Open a repository',
        status: projectRoot ? 'complete' : 'pending',
        action: projectRoot ? undefined : 'Open repository',
      },
      {
        id: 'roadmap-loaded',
        label: 'Load roadmap tasks',
        status: projectRoot ? 'pending' : 'pending',
      },
      {
        id: 'dashboard-link',
        label: 'Open live dashboard',
        status: 'pending',
      },
      {
        id: 'no-key-demo',
        label: 'Try no-key beta mode',
        status: 'pending',
      },
      {
        id: 'ai-mode',
        label: 'Configure real-AI mode (optional)',
        status: 'optional',
      },
    ];

    // Check AI mode status
    try {
      const credStatus = await invoke<string>('get_github_token_status_cmd', { projectRoot: projectRoot || '' });
      const parsed = JSON.parse(credStatus);
      if (parsed.exists) {
        const aiItem = checklist.find(i => i.id === 'ai-mode');
        if (aiItem) aiItem.status = 'configured';
      }
    } catch {
      // Credential check not available — keep optional
    }

    setItems(checklist);
  };

  const handleDismiss = () => {
    setDismissed(true);
    onDismiss();
  };

  if (dismissed) return null;

  const allComplete = items.every(i => i.status === 'complete' || i.status === 'optional' || i.status === 'configured');

  const statusIcon = (status: ChecklistItem['status']) => {
    switch (status) {
      case 'complete': return '✓';
      case 'configured': return '✓';
      case 'pending': return '○';
      case 'failed': return '✗';
      case 'optional': return '○';
      case 'missing-key': return '!';
      case 'unavailable': return '—';
    }
  };

  const statusClass = (status: ChecklistItem['status']) => {
    switch (status) {
      case 'complete':
      case 'configured': return 'text-green-600';
      case 'pending':
      case 'optional': return 'text-gray-400';
      case 'failed': return 'text-red-500';
      case 'missing-key': return 'text-yellow-500';
      case 'unavailable': return 'text-gray-300';
    }
  };

  return (
    <div className="border rounded-lg p-4 mb-4 bg-white shadow-sm">
      <div className="flex justify-between items-center mb-3">
        <h3 className="text-lg font-semibold">Getting Started</h3>
        <button
          onClick={handleDismiss}
          className="text-sm text-gray-400 hover:text-gray-600"
        >
          Dismiss
        </button>
      </div>
      <ul className="space-y-2">
        {items.map(item => (
          <li key={item.id} className="flex items-center gap-2">
            <span className={`font-mono ${statusClass(item.status)}`}>
              {statusIcon(item.status)}
            </span>
            <span className={item.status === 'complete' || item.status === 'configured' ? 'line-through text-gray-500' : ''}>
              {item.label}
            </span>
            {item.status === 'optional' && (
              <span className="text-xs bg-gray-100 text-gray-500 px-1 rounded">optional</span>
            )}
            {item.status === 'configured' && (
              <span className="text-xs bg-green-50 text-green-600 px-1 rounded">configured</span>
            )}
          </li>
        ))}
      </ul>
      {allComplete && (
        <p className="mt-3 text-sm text-green-600">
          All steps complete! Orqestra is ready to use.
        </p>
      )}
      <p className="mt-2 text-xs text-gray-400">
        AI agents operate in review-only mode. No autonomous commits.
      </p>
    </div>
  );
};
