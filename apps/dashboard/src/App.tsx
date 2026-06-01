/**
 * Orqestra Public Dashboard
 *
 * Read-only Gantt + Kanban views rendered from roadmap/ data.
 * Token-based authentication gates write operations.
 */
import React, { useState } from 'react';
import { PublicGantt } from './components/PublicGantt';
import { PublicKanban } from './components/PublicKanban';
import { TokenGate } from './components/TokenGate';
import { TASKS } from './lib/data';

type View = 'gantt' | 'kanban' | 'table';

export function App() {
  const [view, setView] = useState<View>('kanban');
  const [authScope, setAuthScope] = useState<string | null>(null);

  const statusCounts = {
    total: TASKS.length,
    done: TASKS.filter(t => t.status === 'done').length,
    inProgress: TASKS.filter(t => t.status === 'in-progress').length,
    blocked: TASKS.filter(t => t.status === 'blocked').length,
  };

  return (
    <div style={{ maxWidth: 1400, margin: '0 auto', padding: 24 }}>
      {/* Header */}
      <div style={{ marginBottom: 24 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <div>
            <h1 style={{ margin: 0, fontSize: 24, fontWeight: 700 }}>Orqestra Dashboard</h1>
            <span style={{ color: '#64748b', fontSize: 13 }}>Public project tracker · orqestra.pages.dev</span>
          </div>
          {authScope && (
            <div style={{
              padding: '4px 12px', borderRadius: 6,
              backgroundColor: '#22c55e33', color: '#22c55e', fontSize: 13, fontWeight: 600,
            }}>
              Authenticated ({authScope})
            </div>
          )}
        </div>

        {/* Stats bar */}
        <div style={{ display: 'flex', gap: 16, marginBottom: 16 }}>
          {[
            { label: 'Total Tasks', value: statusCounts.total, color: '#e2e8f0' },
            { label: 'Done', value: statusCounts.done, color: '#22c55e' },
            { label: 'In Progress', value: statusCounts.inProgress, color: '#3b82f6' },
            { label: 'Blocked', value: statusCounts.blocked, color: '#ef4444' },
          ].map(s => (
            <div key={s.label} style={{
              padding: '8px 16px', backgroundColor: '#1e293b', borderRadius: 8,
              minWidth: 100,
            }}>
              <div style={{ fontSize: 20, fontWeight: 700, color: s.color }}>{s.value}</div>
              <div style={{ fontSize: 11, color: '#64748b' }}>{s.label}</div>
            </div>
          ))}
        </div>

        {/* Token Gate */}
        <TokenGate onAuth={(_, scope) => setAuthScope(scope)} />

        {/* View switcher */}
        <div style={{ display: 'flex', gap: 4, marginBottom: 16 }}>
          {(['kanban', 'gantt', 'table'] as View[]).map(v => (
            <button
              key={v}
              onClick={() => setView(v)}
              style={{
                padding: '6px 16px', border: 'none', borderRadius: 6,
                cursor: 'pointer', fontWeight: 600, fontSize: 13,
                backgroundColor: view === v ? '#3b82f6' : '#1e293b',
                color: view === v ? '#fff' : '#94a3b8',
              }}
            >
              {v.charAt(0).toUpperCase() + v.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* Content */}
      {view === 'gantt' && <PublicGantt />}
      {view === 'kanban' && <PublicKanban />}
      {view === 'table' && (
        <div style={{ overflowX: 'auto' }}>
          <table style={{
            width: '100%', borderCollapse: 'collapse',
            backgroundColor: '#0f172a', borderRadius: 8, overflow: 'hidden',
          }}>
            <thead>
              <tr style={{ backgroundColor: '#1e293b' }}>
                {['ID', 'Title', 'Status', 'Priority', 'Assignee', 'Sprint', 'Dates'].map(h => (
                  <th key={h} style={{
                    padding: '10px 12px', textAlign: 'left', fontSize: 12,
                    fontWeight: 600, color: '#94a3b8', borderBottom: '1px solid #334155',
                  }}>
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {TASKS.map(t => (
                <tr key={t.id} style={{ borderBottom: '1px solid #1e293b' }}>
                  <td style={{ padding: '8px 12px', fontFamily: 'monospace', fontSize: 13 }}>{t.id}</td>
                  <td style={{ padding: '8px 12px', fontSize: 13 }}>{t.title}</td>
                  <td style={{ padding: '8px 12px', fontSize: 13, textTransform: 'capitalize' }}>{t.status}</td>
                  <td style={{ padding: '8px 12px', fontSize: 13, textTransform: 'capitalize' }}>{t.priority}</td>
                  <td style={{ padding: '8px 12px', fontSize: 13 }}>{t.assignee || '—'}</td>
                  <td style={{ padding: '8px 12px', fontSize: 13 }}>{t.sprint || '—'}</td>
                  <td style={{ padding: '8px 12px', fontSize: 12, color: '#94a3b8' }}>
                    {t.start_date && t.end_date ? `${t.start_date} → ${t.end_date}` : '—'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
