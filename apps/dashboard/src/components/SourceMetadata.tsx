/**
 * SourceMetadata — v2.2.0 dashboard export metadata.
 *
 * Shows WHERE dashboard data came from (static, export-time info).
 * Does NOT claim freshness relative to a viewer's local HEAD.
 * Desktop-computed freshness lives in DashboardCoherencePanel.
 */

import React from 'react';
import type { CoherenceMetadata } from '../lib/data';

interface Props {
  source: { repo: string; branch: string; commit: string };
  generatedAt: string;
  taskCount: number;
  coherence?: CoherenceMetadata;
}

export const SourceMetadata: React.FC<Props> = ({ source, generatedAt, taskCount, coherence }) => {
  const exportState = coherence?.export_state ?? 'unknown';
  const stateColors: Record<string, string> = {
    'local-only': '#3b82f6',
    'relay-metadata-present': '#22c55e',
    'unknown': '#6b7280',
  };

  return (
    <div style={{
      padding: '8px 16px',
      backgroundColor: '#0f172a',
      borderRadius: 8,
      fontSize: 12,
      color: '#94a3b8',
      marginBottom: 16,
    }}>
      <div style={{ display: 'flex', gap: 16, flexWrap: 'wrap', alignItems: 'center' }}>
        <span>
          <strong>Source:</strong> {source.repo}@{source.branch} ({source.commit.slice(0, 12)})
        </span>
        <span>
          <strong>Generated:</strong> {new Date(generatedAt).toLocaleString()}
        </span>
        <span>
          <strong>Tasks:</strong> {taskCount}
        </span>
        {coherence?.roadmap_state_hash && (
          <span>
            <strong>Index hash:</strong> {coherence.roadmap_state_hash.slice(0, 20)}...
          </span>
        )}
        <span style={{
          padding: '2px 8px',
          borderRadius: 4,
          backgroundColor: `${stateColors[exportState]}22`,
          color: stateColors[exportState],
          fontWeight: 600,
        }}>
          {exportState}
        </span>
        {coherence?.relay_snapshot_hash && (
          <span>
            <strong>Relay:</strong> {coherence.relay_snapshot_hash.slice(0, 20)}...
          </span>
        )}
      </div>
    </div>
  );
};
