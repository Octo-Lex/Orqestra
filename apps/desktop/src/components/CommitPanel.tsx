import { useState } from 'react';
import { semanticCommit, backfill } from '../lib/commit';
import { ConfidenceGate, type GateAction } from '../lib/ConfidenceGate';

type CommitPhase = 'idle' | 'committing' | 'backfilling' | 'done' | 'error';

interface CommitState {
  phase: CommitPhase;
  hash: string | null;
  confidence: number | null;
  intentSummary: string | null;
  gateAction: GateAction | null;
  error: string | null;
}

const AI_SERVICE_URL = 'http://localhost:8000';

const GATE_ACTION_LABELS: Record<string, { label: string; color: string }> = {
  auto_commit: { label: 'Auto-Committed', color: '#10b981' },
  propose: { label: 'Proposed — Review Required', color: '#f59e0b' },
  flag: { label: 'Flagged — Human Review', color: '#ef4444' },
  abort: { label: 'Aborted — Low Confidence', color: '#991b1b' },
};

export function CommitPanel({ projectRoot }: { projectRoot: string }) {
  const [message, setMessage] = useState('');
  const [taskIds, setTaskIds] = useState('');
  const [state, setState] = useState<CommitState>({
    phase: 'idle',
    hash: null,
    confidence: null,
    intentSummary: null,
    gateAction: null,
    error: null,
  });

  async function handleCommit() {
    if (!message.trim()) return;

    setState({
      phase: 'committing',
      hash: null,
      confidence: null,
      intentSummary: null,
      gateAction: null,
      error: null,
    });

    try {
      // Step 1: semantic commit
      const ids = taskIds
        .split(',')
        .map(s => s.trim())
        .filter(Boolean);
      const commitResult = await semanticCommit(projectRoot, message.trim(), ids);

      setState(prev => ({ ...prev, phase: 'backfilling', hash: commitResult.hash }));

      // Step 2: AI backfill
      let backfillResult;
      try {
        backfillResult = await backfill(projectRoot, commitResult.hash, AI_SERVICE_URL);
      } catch {
        // AI service failed — default to 'propose' per spec §4.5
        const proposeAction: GateAction = { type: 'propose', ui: 'diff_review_modal', reason: 'ai_unavailable' };
        setState({
          phase: 'done',
          hash: commitResult.hash,
          confidence: null,
          intentSummary: null,
          gateAction: proposeAction,
          error: null,
        });
        return;
      }

      // Step 3: ConfidenceGate
      const gate = new ConfidenceGate();
      const gateAction = gate.resolve(backfillResult.confidence, false);

      setState({
        phase: 'done',
        hash: commitResult.hash,
        confidence: backfillResult.confidence,
        intentSummary: backfillResult.intent_summary,
        gateAction,
        error: null,
      });
    } catch (e) {
      setState(prev => ({
        ...prev,
        phase: 'error',
        error: e instanceof Error ? e.message : String(e),
      }));
    }
  }

  const gateInfo = state.gateAction
    ? GATE_ACTION_LABELS[state.gateAction.type]
    : null;

  return (
    <div style={{ marginTop: '1rem', padding: '0.75rem', border: '1px solid #ddd', borderRadius: '4px' }}>
      <div style={{ fontWeight: 'bold', marginBottom: '0.5rem' }}>Semantic Commit</div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
        <input
          type="text"
          value={message}
          onChange={e => setMessage(e.target.value)}
          placeholder="feat(auth): replace session auth with JWT"
          style={{ padding: '0.25rem 0.5rem', width: '100%' }}
          disabled={state.phase === 'committing' || state.phase === 'backfilling'}
        />
        <input
          type="text"
          value={taskIds}
          onChange={e => setTaskIds(e.target.value)}
          placeholder="Task IDs (comma-separated, e.g. TASK-2026-042)"
          style={{ padding: '0.25rem 0.5rem', width: '100%' }}
          disabled={state.phase === 'committing' || state.phase === 'backfilling'}
        />
        <button
          onClick={handleCommit}
          disabled={!message.trim() || state.phase === 'committing' || state.phase === 'backfilling'}
        >
          {state.phase === 'committing'
            ? 'Committing...'
            : state.phase === 'backfilling'
              ? 'Analyzing intent...'
              : 'Commit'}
        </button>
      </div>

      {/* Status display */}
      {state.phase === 'backfilling' && (
        <div style={{ marginTop: '0.5rem', fontSize: '0.85em', color: '#666' }}>
          Committed {state.hash?.slice(0, 8)}… — calling AI service…
        </div>
      )}

      {state.phase === 'done' && (
        <div style={{ marginTop: '0.5rem', fontSize: '0.85em' }}>
          <div>
            <strong>Commit:</strong>{' '}
            <code>{state.hash?.slice(0, 12)}</code>
          </div>
          {state.confidence !== null && (
            <div>
              <strong>Confidence:</strong>{' '}
              <span style={{ fontWeight: 'bold' }}>
                {(state.confidence * 100).toFixed(1)}%
              </span>
            </div>
          )}
          {state.intentSummary && (
            <div>
              <strong>Intent:</strong> {state.intentSummary}
            </div>
          )}
          {gateInfo && (
            <div style={{ marginTop: '0.25rem' }}>
              <strong>Gate:</strong>{' '}
              <span style={{ color: gateInfo.color, fontWeight: 'bold' }}>
                {gateInfo.label}
              </span>
            </div>
          )}
        </div>
      )}

      {state.phase === 'error' && (
        <div style={{ marginTop: '0.5rem', fontSize: '0.85em', color: '#ef4444' }}>
          {state.error}
        </div>
      )}
    </div>
  );
}
