import { useState, useEffect } from 'react';

interface CommitGroup {
  scope: string;
  change_type: string;
  files: string[];
  risk: string;
  suggested_title: string;
  suggested_body: string;
  requires_manual_review: boolean;
}

interface SemanticCommitProposal {
  title: string;
  body: string;
  scope: string;
  change_type: string;
  confidence: number;
  risk_level: string;
  risk_notes: string[];
  groups: CommitGroup[];
  provider: string;
  write_operations: boolean;
  requires_review: boolean;
}

interface SemanticCommitPrepPanelProps {
  projectRoot: string;
}

export function SemanticCommitPrepPanel({ projectRoot }: SemanticCommitPrepPanelProps) {
  const [proposal, setProposal] = useState<SemanticCommitProposal | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const prepare = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await (window as any).__TAURI_INTERNALS__.invoke(
        'prepare_semantic_commit_cmd',
        { projectRoot }
      );
      setProposal(JSON.parse(result));
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { prepare(); }, [projectRoot]);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  if (loading) return <div style={{ fontSize: '12px', color: '#6b7280' }}>Preparing commit proposal…</div>;
  if (error) return <div style={{ fontSize: '12px', color: '#dc2626' }}>Error: {error}</div>;
  if (!proposal) return null;

  const riskColor = (level: string) => {
    switch (level) {
      case 'elevated': return '#dc2626';
      case 'caution': return '#d97706';
      default: return '#22c55e';
    }
  };

  return (
    <div className="semantic-commit-prep-panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
        <h3 style={{ margin: 0 }}>Commit Proposal</h3>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <span style={{ fontSize: '10px', color: '#9ca3af' }}>Provider: {proposal.provider}</span>
          <button onClick={prepare} style={{ fontSize: '12px', padding: '4px 10px' }}>
            Refresh
          </button>
        </div>
      </div>

      {/* Review Required badge */}
      {proposal.requires_review && (
        <div style={{
          padding: '4px 8px',
          backgroundColor: '#fef3c7',
          border: '1px solid #f59e0b',
          borderRadius: '4px',
          fontSize: '11px',
          color: '#92400e',
          marginBottom: '12px',
        }}>
          ⚠ Review Required — This is a proposal only. No files are staged or committed.
        </div>
      )}

      {/* Title */}
      <div style={{ marginBottom: '12px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
          <strong style={{ fontSize: '12px' }}>Proposed Title</strong>
          <button onClick={() => copyToClipboard(proposal.title)} style={{ fontSize: '10px', padding: '2px 6px' }}>
            Copy
          </button>
        </div>
        <div style={{
          padding: '8px',
          backgroundColor: '#f0fdf4',
          border: '1px solid #86efac',
          borderRadius: '4px',
          fontFamily: 'monospace',
          fontSize: '12px',
        }}>
          {proposal.title}
        </div>
      </div>

      {/* Body */}
      <div style={{ marginBottom: '12px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
          <strong style={{ fontSize: '12px' }}>Proposed Body</strong>
          <button onClick={() => copyToClipboard(proposal.body)} style={{ fontSize: '10px', padding: '2px 6px' }}>
            Copy
          </button>
        </div>
        <pre style={{
          padding: '8px',
          backgroundColor: '#f8fafc',
          border: '1px solid #e2e8f0',
          borderRadius: '4px',
          fontSize: '11px',
          whiteSpace: 'pre-wrap',
          maxHeight: '150px',
          overflow: 'auto',
        }}>
          {proposal.body}
        </pre>
      </div>

      {/* Metadata */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '8px', marginBottom: '12px', fontSize: '12px' }}>
        <div>
          <span style={{ color: '#6b7280' }}>Scope:</span> {proposal.scope}
        </div>
        <div>
          <span style={{ color: '#6b7280' }}>Type:</span> {proposal.change_type}
        </div>
        <div>
          <span style={{ color: '#6b7280' }}>Confidence:</span> {(proposal.confidence * 100).toFixed(0)}%
        </div>
      </div>

      {/* Risk level */}
      <div style={{ marginBottom: '12px' }}>
        <span style={{ color: '#6b7280', fontSize: '12px' }}>Risk Level: </span>
        <span style={{ color: riskColor(proposal.risk_level), fontWeight: 600, fontSize: '12px' }}>
          {proposal.risk_level}
        </span>
      </div>

      {/* Risk notes */}
      {proposal.risk_notes.length > 0 && (
        <div style={{ marginBottom: '12px' }}>
          <strong style={{ fontSize: '12px' }}>Risk Notes</strong>
          <ul style={{ margin: '4px 0', paddingLeft: '16px', fontSize: '11px', color: '#6b7280' }}>
            {proposal.risk_notes.map((note, i) => (
              <li key={i}>{note}</li>
            ))}
          </ul>
        </div>
      )}

      {/* Use in commit form */}
      <div style={{ padding: '8px', backgroundColor: '#eff6ff', borderRadius: '4px', fontSize: '11px' }}>
        <button
          onClick={() => {
            copyToClipboard(`${proposal.title}\n\n${proposal.body}`);
          }}
          style={{ fontSize: '11px', padding: '4px 10px', backgroundColor: '#3b82f6', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
        >
          Use in commit form
        </button>
        <span style={{ marginLeft: '8px', color: '#6b7280' }}>
          Copies title and body to clipboard. Does not stage, commit, or push.
        </span>
      </div>

      {/* Safety invariants */}
      <div style={{ marginTop: '8px', fontSize: '10px', color: '#9ca3af' }}>
        write_operations: {String(proposal.write_operations)} · requires_review: {String(proposal.requires_review)}
      </div>
    </div>
  );
}
