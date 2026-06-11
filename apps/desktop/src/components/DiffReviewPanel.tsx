/**
 * DiffReviewPanel — Shows docs-agent proposed edits for human review.
 * Spec §9.6: all docs-agent output forces propose mode, no auto-commit.
 *
 * v2.14.3: Accept/reject are review-only labels. No file mutation or commit
 * occurs from this panel. Accepting marks the proposal for human reference;
 * files must be applied through a separate guarded flow if desired.
 */
import React, { useState } from 'react';

export type AgentEdit = {
  path: string;
  before: string;
  after: string;
};

export type AgentEditResponse = {
  summary: string;
  confidence: number;
  hasBreakingChange: boolean;
  edits: AgentEdit[];
  notes: string[];
};

type Props = {
  result: AgentEditResponse | null;
  loading: boolean;
  error: string | null;
  onAccept: () => void;
  onReject: () => void;
};

export const DiffReviewPanel: React.FC<Props> = ({ result, loading, error, onAccept, onReject }) => {
  const [accepted, setAccepted] = useState(false);
  const [rejected, setRejected] = useState(false);

  if (loading) {
    return (
      <div style={{ padding: 16, backgroundColor: '#1e293b', borderRadius: 8 }}>
        <div style={{ color: '#94a3b8', fontSize: 14 }}>Agent is running...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ padding: 16, backgroundColor: '#1e293b', borderRadius: 8, borderLeft: '3px solid #ef4444' }}>
        <div style={{ color: '#ef4444', fontSize: 14, fontWeight: 600 }}>Agent Error</div>
        <div style={{ color: '#94a3b8', fontSize: 13, marginTop: 4 }}>{error}</div>
      </div>
    );
  }

  if (!result) {
    return null;
  }

  if (accepted) {
    return (
      <div style={{ padding: 16, backgroundColor: '#22c55e11', borderRadius: 8, borderLeft: '3px solid #22c55e' }}>
        <div style={{ color: '#22c55e', fontSize: 14, fontWeight: 600 }}>Proposal Accepted (Review-Only)</div>
        <div style={{ color: '#94a3b8', fontSize: 12, marginTop: 4 }}>
          Proposal marked as accepted for review. No files were changed and no commit was created.
          Apply changes through the guarded patch flow if desired.
        </div>
      </div>
    );
  }

  if (rejected) {
    return (
      <div style={{ padding: 16, backgroundColor: '#ef444411', borderRadius: 8, borderLeft: '3px solid #ef4444' }}>
        <div style={{ color: '#ef4444', fontSize: 14, fontWeight: 600 }}>Proposal Rejected</div>
        <div style={{ color: '#94a3b8', fontSize: 12, marginTop: 4 }}>
          No changes were applied. Proposal dismissed.
        </div>
      </div>
    );
  }

  const gateAction = result.confidence >= 0.90 ? 'auto_commit (blocked by policy)' :
                     result.confidence >= 0.70 ? 'propose' :
                     result.confidence >= 0.50 ? 'flag' : 'abort';

  return (
    <div style={{ padding: 16, backgroundColor: '#1e293b', borderRadius: 8 }}>
      {/* Header */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
        <div>
          <div style={{ fontWeight: 600, fontSize: 14 }}>Docs Agent Proposal</div>
          <div style={{ color: '#94a3b8', fontSize: 13, marginTop: 2 }}>{result.summary}</div>
        </div>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <span style={{
            fontSize: 13,
            padding: '4px 8px',
            borderRadius: 4,
            backgroundColor: result.confidence >= 0.70 ? '#22c55e33' : '#f59e0b33',
            color: result.confidence >= 0.70 ? '#22c55e' : '#f59e0b',
            fontWeight: 600,
          }}>
            {(result.confidence * 100).toFixed(0)}%
          </span>
          <span style={{
            fontSize: 12,
            padding: '2px 6px',
            borderRadius: 3,
            backgroundColor: '#3b82f633',
            color: '#3b82f6',
          }}>
            {gateAction}
          </span>
        </div>
      </div>

      {/* Policy notice */}
      <div style={{
        padding: '8px 12px', marginBottom: 12,
        backgroundColor: '#f59e0b11', borderRadius: 6,
        fontSize: 12, color: '#f59e0b',
      }}>
        ⚠️ Review-only: accept/reject labels the proposal for your reference. No files are changed and no commit is created from this panel.
      </div>

      {/* Edits */}
      {result.edits.map((edit, i) => (
        <div key={i} style={{ marginBottom: 12 }}>
          <div style={{ fontSize: 12, color: '#64748b', fontFamily: 'monospace', marginBottom: 4 }}>
            {edit.path}
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            <div style={{ flex: 1, padding: 8, backgroundColor: '#ef444411', borderRadius: 4, fontFamily: 'monospace', fontSize: 12, whiteSpace: 'pre-wrap' }}>
              <div style={{ color: '#ef4444', fontSize: 10, marginBottom: 4, fontWeight: 600 }}>BEFORE</div>
              {edit.before}
            </div>
            <div style={{ flex: 1, padding: 8, backgroundColor: '#22c55e11', borderRadius: 4, fontFamily: 'monospace', fontSize: 12, whiteSpace: 'pre-wrap' }}>
              <div style={{ color: '#22c55e', fontSize: 10, marginBottom: 4, fontWeight: 600 }}>AFTER</div>
              {edit.after}
            </div>
          </div>
        </div>
      ))}

      {/* Notes */}
      {result.notes.length > 0 && (
        <div style={{ fontSize: 12, color: '#64748b', marginBottom: 12 }}>
          {result.notes.map((n, i) => (
            <div key={i}>• {n}</div>
          ))}
        </div>
      )}

      {/* Actions */}
      <div style={{ display: 'flex', gap: 8 }}>
        <button
          onClick={() => { setAccepted(true); onAccept(); }}
          style={{
            padding: '8px 16px', backgroundColor: '#22c55e', color: '#fff',
            border: 'none', borderRadius: 6, cursor: 'pointer', fontWeight: 600,
          }}
        >
          Accept (Review-Only)
        </button>
        <button
          onClick={() => { setRejected(true); onReject(); }}
          style={{
            padding: '8px 16px', backgroundColor: '#ef4444', color: '#fff',
            border: 'none', borderRadius: 6, cursor: 'pointer', fontWeight: 600,
          }}
        >
          Reject
        </button>
      </div>
    </div>
  );
};
