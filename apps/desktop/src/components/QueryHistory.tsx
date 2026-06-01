import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface QueryResult {
  answer: string;
  supporting_commits: string[];
}

interface CommitDetail {
  hash: string;
  conventional_message: string;
  timestamp: string;
  semantic: {
    status: string;
    intent_summary: string;
    affected_concepts: string[];
    confidence: number;
    reasoning_trace_id: string | null;
    task_ids: string[];
    risk_assessment: {
      breaking_change: boolean;
      rollback_complexity: string;
    };
  };
}

interface Props {
  projectRoot: string;
}

export default function QueryHistory({ projectRoot }: Props) {
  const [question, setQuestion] = useState('');
  const [result, setResult] = useState<QueryResult | null>(null);
  const [selectedCommit, setSelectedCommit] = useState<CommitDetail | null>(null);
  const [trace, setTrace] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [graphStatus, setGraphStatus] = useState<string>('');

  const handleIndex = useCallback(async () => {
    try {
      setGraphStatus('Indexing...');
      const r = await invoke<{ indexed: number; total_triples: number }>('index_graph_cmd', {
        projectRoot,
      });
      setGraphStatus(`Indexed ${r.indexed} commits → ${r.total_triples} triples`);
    } catch (e) {
      setGraphStatus(`Error: ${e}`);
    }
  }, [projectRoot]);

  const handleQuery = useCallback(async () => {
    if (!question.trim()) return;
    setLoading(true);
    setSelectedCommit(null);
    setTrace(null);
    try {
      // Try Python AI service first
      const r = await invoke<QueryResult>('query_history_cmd', {
        projectRoot,
        question,
      });
      setResult(r);
    } catch {
      // Fallback: query triple store directly
      try {
        const triples = await invoke<
          { uuid: string; subject: string; predicate: string; object: string }[]
        >('query_graph_cmd', {
          projectRoot,
          predicate: 'has_intent',
          subject: null,
          object: null,
        });
        if (triples.length === 0) {
          setResult({ answer: 'No commits indexed. Click "Index Graph" first.', supporting_commits: [] });
        } else {
          const lines = triples.map(
            (t) => `  ${t.subject.slice(0, 12)}: ${t.object}`
          );
          setResult({
            answer: `All commit intents:\n${lines.join('\n')}`,
            supporting_commits: triples.map((t) => t.subject),
          });
        }
      } catch (e2) {
        setResult({ answer: `Error: ${e2}`, supporting_commits: [] });
      }
    } finally {
      setLoading(false);
    }
  }, [projectRoot, question]);

  const handleSelectCommit = useCallback(
    async (hash: string) => {
      try {
        const stub = await invoke<CommitDetail>('read_commit_stub_cmd', {
          projectRoot,
          hash,
        });
        setSelectedCommit(stub);

        // Load reasoning trace
        if (stub.semantic?.reasoning_trace_id) {
          const t = await invoke<string>('read_trace_cmd', {
            projectRoot,
            traceId: stub.semantic.reasoning_trace_id,
          });
          setTrace(t);
        } else {
          setTrace(null);
        }
      } catch (e) {
        setSelectedCommit(null);
        setTrace(`Error loading: ${e}`);
      }
    },
    [projectRoot]
  );

  return (
    <div style={{ padding: '12px', borderTop: '1px solid #374151' }}>
      <h3 style={{ margin: '0 0 8px', color: '#e5e7eb' }}>Semantic History Query</h3>

      <div style={{ display: 'flex', gap: 8, marginBottom: 8 }}>
        <button
          onClick={handleIndex}
          style={{
            padding: '4px 12px',
            background: '#4f46e5',
            color: 'white',
            border: 'none',
            borderRadius: 4,
            cursor: 'pointer',
          }}
        >
          Index Graph
        </button>
        {graphStatus && (
          <span style={{ fontSize: 12, color: '#9ca3af', lineHeight: '28px' }}>
            {graphStatus}
          </span>
        )}
      </div>

      <div style={{ display: 'flex', gap: 8 }}>
        <input
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleQuery()}
          placeholder='Ask: "When did we introduce rate limiting?"'
          style={{
            flex: 1,
            padding: '8px',
            background: '#1f2937',
            border: '1px solid #374151',
            borderRadius: 4,
            color: '#e5e7eb',
            fontSize: 14,
          }}
        />
        <button
          onClick={handleQuery}
          disabled={loading}
          style={{
            padding: '8px 16px',
            background: loading ? '#6b7280' : '#10b981',
            color: 'white',
            border: 'none',
            borderRadius: 4,
            cursor: loading ? 'not-allowed' : 'pointer',
          }}
        >
          {loading ? 'Searching...' : 'Search'}
        </button>
      </div>

      {result && (
        <div
          style={{
            marginTop: 12,
            background: '#111827',
            border: '1px solid #374151',
            borderRadius: 6,
            padding: 12,
          }}
        >
          <pre
            style={{
              whiteSpace: 'pre-wrap',
              fontFamily: 'inherit',
              margin: 0,
              color: '#d1d5db',
              fontSize: 13,
            }}
          >
            {result.answer}
          </pre>

          {result.supporting_commits.length > 0 && (
            <div style={{ marginTop: 12 }}>
              <h4 style={{ margin: '0 0 6px', color: '#9ca3af', fontSize: 12 }}>
                Supporting Commits (click to expand)
              </h4>
              {result.supporting_commits.map((hash) => (
                <button
                  key={hash}
                  onClick={() => handleSelectCommit(hash)}
                  style={{
                    display: 'block',
                    margin: '2px 0',
                    padding: '4px 8px',
                    background:
                      selectedCommit?.hash === hash ? '#374151' : 'transparent',
                    border: '1px solid #374151',
                    borderRadius: 4,
                    color: '#93c5fd',
                    cursor: 'pointer',
                    fontSize: 12,
                    fontFamily: 'monospace',
                    textAlign: 'left',
                    width: '100%',
                  }}
                >
                  {hash.slice(0, 12)}
                  {selectedCommit?.hash === hash ? ' ◀' : ''}
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {selectedCommit && (
        <div
          style={{
            marginTop: 12,
            background: '#0f172a',
            border: '1px solid #1e40af',
            borderRadius: 6,
            padding: 12,
          }}
        >
          <h4 style={{ margin: '0 0 8px', color: '#60a5fa' }}>
            Commit: {selectedCommit.hash.slice(0, 12)}
          </h4>
          <p style={{ margin: '4px 0', color: '#d1d5db', fontSize: 13 }}>
            <strong>Message:</strong> {selectedCommit.conventional_message}
          </p>
          <p style={{ margin: '4px 0', color: '#d1d5db', fontSize: 13 }}>
            <strong>Intent:</strong> {selectedCommit.semantic?.intent_summary}
          </p>
          <p style={{ margin: '4px 0', color: '#d1d5db', fontSize: 13 }}>
            <strong>Confidence:</strong>{' '}
            <span
              style={{
                color:
                  (selectedCommit.semantic?.confidence ?? 0) >= 0.9
                    ? '#34d399'
                    : '#fbbf24',
              }}
            >
              {((selectedCommit.semantic?.confidence ?? 0) * 100).toFixed(0)}%
            </span>
          </p>
          {selectedCommit.semantic?.affected_concepts?.length > 0 && (
            <div style={{ margin: '4px 0' }}>
              <strong style={{ color: '#9ca3af', fontSize: 12 }}>Concepts:</strong>{' '}
              {selectedCommit.semantic.affected_concepts.map((c) => (
                <span
                  key={c}
                  style={{
                    display: 'inline-block',
                    background: '#1e3a5f',
                    color: '#93c5fd',
                    padding: '2px 8px',
                    borderRadius: 10,
                    fontSize: 11,
                    margin: '2px',
                  }}
                >
                  {c}
                </span>
              ))}
            </div>
          )}
          {selectedCommit.semantic?.task_ids?.length > 0 && (
            <p style={{ margin: '4px 0', color: '#d1d5db', fontSize: 13 }}>
              <strong>Tasks:</strong> {selectedCommit.semantic.task_ids.join(', ')}
            </p>
          )}
          {selectedCommit.semantic?.risk_assessment && (
            <p style={{ margin: '4px 0', fontSize: 13 }}>
              <strong style={{ color: '#9ca3af' }}>Risk:</strong>{' '}
              <span
                style={{
                  color: selectedCommit.semantic.risk_assessment.breaking_change
                    ? '#f87171'
                    : '#34d399',
                }}
              >
                {selectedCommit.semantic.risk_assessment.breaking_change
                  ? '⚠ BREAKING'
                  : '✓ Safe'}
              </span>
              <span style={{ color: '#6b7280', marginLeft: 8 }}>
                rollback: {selectedCommit.semantic.risk_assessment.rollback_complexity}
              </span>
            </p>
          )}

          {trace && (
            <div
              style={{
                marginTop: 8,
                padding: 8,
                background: '#1a1a2e',
                borderRadius: 4,
                border: '1px solid #374151',
              }}
            >
              <h5 style={{ margin: '0 0 4px', color: '#a78bfa', fontSize: 12 }}>
                Reasoning Trace
              </h5>
              <pre
                style={{
                  whiteSpace: 'pre-wrap',
                  fontFamily: 'inherit',
                  margin: 0,
                  color: '#c4b5fd',
                  fontSize: 12,
                }}
              >
                {trace}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
