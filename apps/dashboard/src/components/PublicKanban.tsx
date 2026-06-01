/**
 * PublicKanban — Read-only Kanban board for stakeholders.
 * Tasks grouped by status, color-coded by priority.
 */
import React from 'react';
import { STATUS_COLORS, PRIORITY_COLORS, type Task, type TaskStatus } from '../lib/data';

const COLUMNS: { status: string; label: string }[] = [
  { status: 'backlog', label: 'Backlog' },
  { status: 'ready', label: 'Ready' },
  { status: 'in-progress', label: 'In Progress' },
  { status: 'in-review', label: 'Review' },
  { status: 'done', label: 'Done' },
];

const TaskCard: React.FC<{ task: Task }> = ({ task }) => (
  <div style={{
    padding: 12,
    backgroundColor: '#1e293b',
    borderRadius: 8,
    marginBottom: 8,
    borderLeft: `3px solid ${PRIORITY_COLORS[task.priority] || '#6b7280'}`,
  }}>
    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
      <span style={{ fontSize: 12, color: '#64748b', fontFamily: 'monospace' }}>{task.id}</span>
      <span style={{
        fontSize: 10,
        padding: '2px 6px',
        borderRadius: 4,
        backgroundColor: (PRIORITY_COLORS[task.priority] || '#6b7280') + '33',
        color: PRIORITY_COLORS[task.priority] || '#6b7280',
        fontWeight: 600,
        textTransform: 'uppercase',
      }}>
        {task.priority}
      </span>
    </div>
    <div style={{ fontSize: 14, fontWeight: 600, marginBottom: 4 }}>{task.title}</div>
    <div style={{ display: 'flex', gap: 8, fontSize: 11, color: '#64748b' }}>
      {task.assignee && <span>@{task.assignee}</span>}
      {task.sprint && <span>{task.sprint}</span>}
    </div>
    {task.labels.length > 0 && (
      <div style={{ display: 'flex', gap: 4, marginTop: 6, flexWrap: 'wrap' }}>
        {task.labels.slice(0, 3).map(l => (
          <span key={l} style={{
            fontSize: 10, padding: '1px 6px', borderRadius: 3,
            backgroundColor: '#334155', color: '#94a3b8',
          }}>
            {l}
          </span>
        ))}
      </div>
    )}
  </div>
);

export const PublicKanban: React.FC<{ tasks: Task[] }> = ({ tasks }) => {
  return (
    <div>
      <h3 style={{ marginTop: 0, marginBottom: 12 }}>Kanban Board</h3>
      <div style={{ display: 'grid', gridTemplateColumns: `repeat(${COLUMNS.length}, 1fr)`, gap: 12 }}>
        {COLUMNS.map(col => {
          const colTasks = tasks.filter(t => t.status === col.status);
          return (
            <div key={col.status}>
              <div style={{
                display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                marginBottom: 8, padding: '8px 12px',
                backgroundColor: '#1e293b', borderRadius: 8,
              }}>
                <span style={{ fontWeight: 600, fontSize: 14 }}>{col.label}</span>
                <span style={{
                  fontSize: 12, fontWeight: 700,
                  backgroundColor: (STATUS_COLORS[col.status] || '#6b7280') + '33',
                  color: STATUS_COLORS[col.status] || '#6b7280',
                  padding: '2px 8px', borderRadius: 10,
                }}>
                  {colTasks.length}
                </span>
              </div>
              <div>
                {colTasks.map(task => (
                  <TaskCard key={task.id} task={task} />
                ))}
                {colTasks.length === 0 && (
                  <div style={{ padding: 16, textAlign: 'center', color: '#475569', fontSize: 13 }}>
                    No tasks
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
