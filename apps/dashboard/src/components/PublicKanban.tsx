/**
 * PublicKanban — Read-only Kanban board for stakeholders.
 * Tasks grouped by status, color-coded by priority.
 */
import React from 'react';
import { TASKS, STATUS_COLORS, PRIORITY_COLORS, type Task, type TaskStatus } from '../lib/data';

const COLUMNS: { status: TaskStatus; label: string }[] = [
  { status: 'todo', label: 'To Do' },
  { status: 'in-progress', label: 'In Progress' },
  { status: 'review', label: 'Review' },
  { status: 'done', label: 'Done' },
  { status: 'blocked', label: 'Blocked' },
];

const TaskCard: React.FC<{ task: Task }> = ({ task }) => (
  <div style={{
    padding: 12,
    backgroundColor: '#1e293b',
    borderRadius: 8,
    marginBottom: 8,
    borderLeft: `3px solid ${PRIORITY_COLORS[task.priority]}`,
  }}>
    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
      <span style={{ fontSize: 12, color: '#64748b', fontFamily: 'monospace' }}>{task.id}</span>
      <span style={{
        fontSize: 10,
        padding: '2px 6px',
        borderRadius: 4,
        backgroundColor: PRIORITY_COLORS[task.priority] + '33',
        color: PRIORITY_COLORS[task.priority],
        fontWeight: 600,
        textTransform: 'uppercase',
      }}>
        {task.priority}
      </span>
    </div>
    <div style={{ fontSize: 14, fontWeight: 600, marginBottom: 4 }}>{task.title}</div>
    {task.description && (
      <div style={{ fontSize: 12, color: '#94a3b8', marginBottom: 4 }}>{task.description}</div>
    )}
    <div style={{ display: 'flex', gap: 8, fontSize: 11, color: '#64748b' }}>
      {task.assignee && <span>@{task.assignee}</span>}
      {task.sprint && <span>{task.sprint}</span>}
    </div>
  </div>
);

export const PublicKanban: React.FC = () => {
  return (
    <div>
      <h3 style={{ marginTop: 0, marginBottom: 12 }}>Kanban Board</h3>
      <div style={{ display: 'grid', gridTemplateColumns: `repeat(${COLUMNS.length}, 1fr)`, gap: 12 }}>
        {COLUMNS.map(col => {
          const tasks = TASKS.filter(t => t.status === col.status);
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
                  backgroundColor: STATUS_COLORS[col.status] + '33',
                  color: STATUS_COLORS[col.status],
                  padding: '2px 8px', borderRadius: 10,
                }}>
                  {tasks.length}
                </span>
              </div>
              <div>
                {tasks.map(task => (
                  <TaskCard key={task.id} task={task} />
                ))}
                {tasks.length === 0 && (
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
