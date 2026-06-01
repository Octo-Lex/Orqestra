import { useState } from 'react';

interface ConflictFile {
  path: string;
  leftIntent: string;
  rightIntent: string;
  leftAuthor: string;
  rightAuthor: string;
  diff: string;
  resolution: string | null;
}

/** Mock conflict data for browser testing */
const MOCK_CONFLICTS: ConflictFile[] = [
  {
    path: 'src/lib/auth.rs',
    leftIntent: 'Alice refactored auth middleware for performance',
    rightAuthor: 'bob',
    leftAuthor: 'alice',
    rightIntent: 'Bob added OAuth2 PKCE flow to auth endpoints',
    diff: `<<<<<<< HEAD (alice)
fn check_rate_limit(req: &Request) -> Result<bool, AuthError> {
    let key = format!("rl:{}:{}", req.client_id, req.ip);
    let count = cache::incr(&key, RATE_WINDOW)?;
    Ok(count <= RATE_LIMIT)
}
=======
fn check_rate_limit(req: &Request) -> Result<bool, AuthError> {
    let key = format!("rl:{}:{}", req.client_id, req.ip);
    let oauth_state = req.session.get::<String>("oauth_state")?;
    if oauth_state.is_some() {
        return Ok(true); // Skip rate limit during OAuth flow
    }
    let count = cache::incr(&key, RATE_WINDOW)?;
    Ok(count <= RATE_LIMIT)
}
>>>>>>> feature/oauth-pkce (bob)`,
    resolution: 'Merge both: keep performance refactor AND add OAuth PKCE state check before rate limit. The OAuth state bypass is safe because PKCE has its own rate protection.',
  },
];

interface Props {
  projectRoot: string;
}

export default function ShockwaveMerge({ projectRoot }: Props) {
  const [conflicts] = useState<ConflictFile[]>(MOCK_CONFLICTS);
  const [expanded, setExpanded] = useState<number | null>(null);
  const [showResolution, setShowResolution] = useState<Set<number>>(new Set());

  const toggleExpand = (idx: number) => {
    setExpanded(expanded === idx ? null : idx);
    // Reset resolution visibility when collapsing
    if (expanded === idx) {
      setShowResolution(new Set());
    }
  };

  const toggleResolution = (idx: number) => {
    setShowResolution((prev) => {
      const next = new Set(prev);
      if (next.has(idx)) {
        next.delete(idx);
      } else {
        next.add(idx);
      }
      return next;
    });
  };

  return (
    <div style={{ padding: 12, borderTop: '1px solid #374151' }}>
      <h3 style={{ margin: '0 0 8px', color: '#e5e7eb' }}>
        ⚡ Shockwave Merge Conflict UI
      </h3>
      <p style={{ margin: '0 0 8px', color: '#6b7280', fontSize: 12 }}>
        Shows intent alongside the diff. AI proposes resolutions based on semantic
        understanding of both sides.
      </p>

      {conflicts.length === 0 && (
        <div
          style={{
            padding: 12,
            background: '#111827',
            borderRadius: 6,
            border: '1px solid #374151',
          }}
        >
          <span style={{ color: '#34d399', fontSize: 13 }}>✓ No merge conflicts</span>
          {projectRoot && (
            <p style={{ color: '#6b7280', fontSize: 11, margin: '4px 0 0' }}>
              Conflict detection will be active when running inside Tauri
            </p>
          )}
        </div>
      )}

      {conflicts.map((conflict, idx) => (
        <div
          key={idx}
          style={{
            marginTop: 8,
            background: '#111827',
            borderRadius: 6,
            border: expanded === idx ? '1px solid #f59e0b' : '1px solid #374151',
            overflow: 'hidden',
          }}
        >
          {/* Header */}
          <div
            onClick={() => toggleExpand(idx)}
            style={{
              padding: '8px 12px',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              cursor: 'pointer',
              background: expanded === idx ? '#1f1a0f' : 'transparent',
            }}
          >
            <div>
              <span style={{ color: '#fbbf24', fontSize: 13 }}>
                ⚠ {conflict.path}
              </span>
            </div>
            <span style={{ color: '#6b7280', fontSize: 12 }}>
              {expanded === idx ? '▲' : '▼'}
            </span>
          </div>

          {/* Expanded content */}
          {expanded === idx && (
            <div style={{ padding: '0 12px 12px' }}>
              {/* Intent cards */}
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: '1fr 1fr',
                  gap: 8,
                  marginBottom: 8,
                }}
              >
                <div
                  style={{
                    padding: 8,
                    background: '#0c1f0c',
                    border: '1px solid #166534',
                    borderRadius: 4,
                  }}
                >
                  <div
                    style={{
                      color: '#4ade80',
                      fontSize: 11,
                      fontWeight: 600,
                      marginBottom: 4,
                    }}
                  >
                    ← {conflict.leftAuthor} (ours)
                  </div>
                  <p style={{ margin: 0, color: '#86efac', fontSize: 12 }}>
                    {conflict.leftIntent}
                  </p>
                </div>
                <div
                  style={{
                    padding: 8,
                    background: '#1f0c0c',
                    border: '1px solid #991b1b',
                    borderRadius: 4,
                  }}
                >
                  <div
                    style={{
                      color: '#f87171',
                      fontSize: 11,
                      fontWeight: 600,
                      marginBottom: 4,
                    }}
                  >
                    → {conflict.rightAuthor} (theirs)
                  </div>
                  <p style={{ margin: 0, color: '#fca5a5', fontSize: 12 }}>
                    {conflict.rightIntent}
                  </p>
                </div>
              </div>

              {/* Diff */}
              <div
                style={{
                  background: '#0d0d0d',
                  borderRadius: 4,
                  padding: 8,
                  fontFamily: 'monospace',
                  fontSize: 11,
                  lineHeight: 1.5,
                  whiteSpace: 'pre-wrap',
                  color: '#9ca3af',
                  maxHeight: 200,
                  overflow: 'auto',
                }}
              >
                {conflict.diff.split('\n').map((line, li) => {
                  if (line.startsWith('<<<<<<<'))
                    return (
                      <div key={li} style={{ color: '#f87171', fontWeight: 600 }}>
                        {line}
                      </div>
                    );
                  if (line.startsWith('======='))
                    return (
                      <div key={li} style={{ color: '#fbbf24', fontWeight: 600 }}>
                        {line}
                      </div>
                    );
                  if (line.startsWith('>>>>>>>'))
                    return (
                      <div key={li} style={{ color: '#60a5fa', fontWeight: 600 }}>
                        {line}
                      </div>
                    );
                  return <div key={li}>{line}</div>;
                })}
              </div>

              {/* AI Resolution */}
              <div style={{ marginTop: 8 }}>
                <button
                  onClick={() => toggleResolution(idx)}
                  style={{
                    padding: '4px 12px',
                    background: showResolution.has(idx) ? '#6b21a8' : '#7c3aed',
                    color: 'white',
                    border: 'none',
                    borderRadius: 4,
                    cursor: 'pointer',
                    fontSize: 12,
                  }}
                >
                  {showResolution.has(idx) ? 'Hide' : 'Show'} AI Resolution Proposal
                </button>
                {showResolution.has(idx) && conflict.resolution && (
                  <div
                    style={{
                      marginTop: 8,
                      padding: 8,
                      background: '#1a0f2e',
                      border: '1px solid #6d28d9',
                      borderRadius: 4,
                    }}
                  >
                    <h5
                      style={{
                        margin: '0 0 4px',
                        color: '#a78bfa',
                        fontSize: 11,
                      }}
                    >
                      🤖 AI Proposed Resolution
                    </h5>
                    <p style={{ margin: 0, color: '#c4b5fd', fontSize: 12 }}>
                      {conflict.resolution}
                    </p>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
