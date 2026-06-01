import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface CommitDetail {
  hash: string;
  conventional_message: string;
  timestamp: string;
  author: { name: string; type: string };
  semantic: {
    status: string;
    intent_summary: string;
    affected_concepts: string[];
    affected_apis: string[];
    confidence: number;
    reasoning_trace_id: string | null;
    task_ids: string[];
    risk_assessment: {
      breaking_change: boolean;
      migration_required: string | null;
      rollback_complexity: string;
    };
  };
}

interface Props {
  projectRoot: string;
}

export default function SemanticDiff({ projectRoot }: Props) {
  const [hash, setHash] = useState('');
  const [commit, setCommit] = useState<CommitDetail | null>(null);
  const [trace, setTrace] = useState<string | null>(null);
  const [error, setError] = useState('');

  const handleLoad = useCallback(async () => {
    if (!hash.trim()) return;
    setError('');
    try {
      const stub = await invoke<CommitDetail>('read_commit_stub_cmd', {
        projectRoot,
        hash: hash.trim(),
      });
      setCommit(stub);

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
      setError(String(e));
      setCommit(null);
      setTrace(null);
    }
  }, [projectRoot, hash]);

  return (
    <div style={{ padding: 12, borderTop: '1px solid #374151' }}>
      <h3 style={{ margin: '0 0 8px', color: '#e5e7eb' }}>Semantic Diff View</h3>
      <p style={{ margin: '0 0 8px', color: '#6b7280', fontSize: 12 }}>
        Enter a commit hash to see its intent, concepts, and reasoning alongside the
        diff.
      </p>

      <div style={{ display: 'flex', gap: 8 }}>
        <input
          value={hash}
          onChange={(e) => setHash(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleLoad()}
          placeholder="Commit hash"
          style={{
            flex: 1,
            padding: '6px 8px',
            background: '#1f2937',
            border: '1px solid #374151',
            borderRadius: 4,
            color: '#e5e7eb',
            fontFamily: 'monospace',
            fontSize: 13,
          }}
        />
        <button
          onClick={handleLoad}
          style={{
            padding: '6px 16px',
            background: '#4f46e5',
            color: 'white',
            border: 'none',
            borderRadius: 4,
            cursor: 'pointer',
          }}
        >
          Load
        </button>
      </div>

      {error && (
        <p style={{ color: '#f87171', fontSize: 13, marginTop: 4 }}>{error}</p>
      )}

      {commit && (
        <div
          style={{
            marginTop: 12,
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: 12,
          }}
        >
          {/* Left: Conventional diff info */}
          <div
            style={{
              background: '#111827',
              border: '1px solid #374151',
              borderRadius: 6,
              padding: 12,
            }}
          >
            <h4 style={{ margin: '0 0 8px', color: '#60a5fa' }}>What Changed</h4>
            <div style={{ marginBottom: 8 }}>
              <span
                style={{
                  background: '#1e3a5f',
                  color: '#93c5fd',
                  padding: '2px 8px',
                  borderRadius: 4,
                  fontSize: 11,
                  fontFamily: 'monospace',
                }}
              >
                {commit.hash.slice(0, 12)}
              </span>
              <span style={{ marginLeft: 8, color: '#9ca3af', fontSize: 12 }}>
                by {commit.author?.name}
              </span>
            </div>
            <p
              style={{
                fontFamily: 'monospace',
                color: '#34d399',
                fontSize: 13,
                margin: '4px 0',
              }}
            >
              {commit.conventional_message}
            </p>
            <p style={{ color: '#6b7280', fontSize: 12, margin: '4px 0' }}>
              {commit.timestamp}
            </p>
            {commit.semantic?.task_ids?.map((t) => (
              <span
                key={t}
                style={{
                  display: 'inline-block',
                  background: '#1e293b',
                  color: '#a78bfa',
                  padding: '2px 6px',
                  borderRadius: 4,
                  fontSize: 11,
                  margin: '2px',
                }}
              >
                {t}
              </span>
            ))}
          </div>

          {/* Right: Semantic "Why" */}
          <div
            style={{
              background: '#0f172a',
              border: '1px solid #1e40af',
              borderRadius: 6,
              padding: 12,
            }}
          >
            <h4 style={{ margin: '0 0 8px', color: '#a78bfa' }}>Why (Intent)</h4>
            <p style={{ color: '#d1d5db', fontSize: 13, margin: '4px 0' }}>
              {commit.semantic?.intent_summary}
            </p>

            {commit.semantic?.affected_concepts?.length > 0 && (
              <div style={{ margin: '8px 0' }}>
                <span style={{ color: '#9ca3af', fontSize: 11 }}>Concepts:</span>{' '}
                {commit.semantic.affected_concepts.map((c) => (
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

            {commit.semantic?.affected_apis?.length > 0 && (
              <div style={{ margin: '4px 0' }}>
                <span style={{ color: '#9ca3af', fontSize: 11 }}>APIs:</span>{' '}
                {commit.semantic.affected_apis.map((a) => (
                  <span
                    key={a}
                    style={{
                      display: 'inline-block',
                      background: '#3b1f1f',
                      color: '#fca5a5',
                      padding: '2px 8px',
                      borderRadius: 10,
                      fontSize: 11,
                      margin: '2px',
                    }}
                  >
                    {a}
                  </span>
                ))}
              </div>
            )}

            <div style={{ marginTop: 8, fontSize: 12 }}>
              <span style={{ color: '#9ca3af' }}>Risk:</span>{' '}
              <span
                style={{
                  color: commit.semantic?.risk_assessment?.breaking_change
                    ? '#f87171'
                    : '#34d399',
                }}
              >
                {commit.semantic?.risk_assessment?.breaking_change
                  ? '⚠ BREAKING'
                  : '✓ Safe'}
              </span>
              <span style={{ color: '#6b7280', marginLeft: 8 }}>
                confidence:{' '}
                {((commit.semantic?.confidence ?? 0) * 100).toFixed(0)}%
              </span>
            </div>
          </div>

          {/* Full-width: Reasoning Trace */}
          {trace && (
            <div
              style={{
                gridColumn: '1 / -1',
                background: '#1a1a2e',
                border: '1px solid #374151',
                borderRadius: 6,
                padding: 12,
              }}
            >
              <h4 style={{ margin: '0 0 6px', color: '#c084fc', fontSize: 12 }}>
                AI Reasoning Trace
              </h4>
              <pre
                style={{
                  whiteSpace: 'pre-wrap',
                  fontFamily: 'inherit',
                  margin: 0,
                  color: '#c4b5fd',
                  fontSize: 12,
                  lineHeight: 1.5,
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
