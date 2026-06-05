/**
 * DashboardCoherencePanel — v2.2.0 desktop coherence display.
 *
 * Shows freshness of dashboard export relative to local workspace.
 * Computed by desktop (check_dashboard_coherence_cmd), not by the static dashboard.
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface CoherenceResult {
  dashboard_export_exists: boolean;
  dashboard_commit: string | null;
  local_head: string | null;
  commits_behind: number | null;
  freshness: string;
  local_roadmap_state_hash: string | null;
  dashboard_roadmap_state_hash: string | null;
  task_count_local: number | null;
  task_count_dashboard: number | null;
  relay: {
    available: boolean;
    relay_url_host: string | null;
    workspace_id_hash: string | null;
    last_snapshot_hash: string | null;
    connected: boolean;
  };
}

interface Props {
  projectRoot: string | null;
}

const freshnessColor: Record<string, string> = {
  current: '#22c55e',
  stale: '#f59e0b',
  diverged: '#ef4444',
  'local-only': '#3b82f6',
  'relay-unavailable': '#6b7280',
  unknown: '#64748b',
};

const freshnessIcon: Record<string, string> = {
  current: '🟢',
  stale: '🟡',
  diverged: '🔴',
  'local-only': '🔵',
  'relay-unavailable': '⚪',
  unknown: '⚪',
};

export const DashboardCoherencePanel: React.FC<Props> = ({ projectRoot }) => {
  const [result, setResult] = useState<CoherenceResult | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (projectRoot) {
      checkCoherence();
    }
  }, [projectRoot]);

  const checkCoherence = async () => {
    if (!projectRoot) return;
    setLoading(true);
    try {
      const res = await invoke<CoherenceResult>('check_dashboard_coherence_cmd', { projectRoot });
      setResult(res);
    } catch {
      setResult(null);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div className="text-sm text-gray-400 p-2">Checking dashboard coherence...</div>;
  }

  if (!result) {
    return <div className="text-sm text-gray-400 p-2">Coherence check unavailable</div>;
  }

  const color = freshnessColor[result.freshness] || '#64748b';
  const icon = freshnessIcon[result.freshness] || '⚪';

  return (
    <div className="border rounded-lg p-4 mb-4 bg-white shadow-sm">
      <h3 className="font-semibold mb-3">Dashboard Coherence</h3>

      {/* Freshness badge */}
      <div className="flex items-center gap-2 mb-3">
        <span style={{ fontSize: 16 }}>{icon}</span>
        <span style={{ fontWeight: 600, color, textTransform: 'capitalize' }}>
          {result.freshness}
        </span>
        {result.commits_behind !== null && result.commits_behind > 0 && (
          <span className="text-xs text-gray-500">
            ({result.commits_behind} commits behind)
          </span>
        )}
      </div>

      {/* Details */}
      <div className="grid grid-cols-2 gap-2 text-sm">
        <div>
          <span className="text-gray-500">Local HEAD:</span>{' '}
          <code className="text-xs">{result.local_head?.slice(0, 12) ?? '—'}</code>
        </div>
        <div>
          <span className="text-gray-500">Dashboard commit:</span>{' '}
          <code className="text-xs">{result.dashboard_commit?.slice(0, 12) ?? '—'}</code>
        </div>
        <div>
          <span className="text-gray-500">Local tasks:</span>{' '}
          {result.task_count_local ?? '—'}
        </div>
        <div>
          <span className="text-gray-500">Dashboard tasks:</span>{' '}
          {result.task_count_dashboard ?? '—'}
        </div>
        {result.local_roadmap_state_hash && (
          <div className="col-span-2">
            <span className="text-gray-500">Local state hash:</span>{' '}
            <code className="text-xs">{result.local_roadmap_state_hash.slice(0, 20)}...</code>
          </div>
        )}
        {result.dashboard_roadmap_state_hash && (
          <div className="col-span-2">
            <span className="text-gray-500">Dashboard state hash:</span>{' '}
            <code className="text-xs">{result.dashboard_roadmap_state_hash.slice(0, 20)}...</code>
          </div>
        )}
      </div>

      {/* Relay */}
      <div className="mt-3 pt-3 border-t text-xs text-gray-400">
        <span>Relay: {result.relay.available ? '✓' : '✗'}</span>
        {result.relay.relay_url_host && (
          <span className="ml-2">Host: {result.relay.relay_url_host}</span>
        )}
        {result.relay.workspace_id_hash && (
          <span className="ml-2">Workspace: {result.relay.workspace_id_hash.slice(0, 16)}...</span>
        )}
      </div>

      {!result.dashboard_export_exists && (
        <p className="mt-3 text-xs text-gray-400">
          No dashboard export found. Run the export CLI to generate orqestra-roadmap.json.
        </p>
      )}
    </div>
  );
};
