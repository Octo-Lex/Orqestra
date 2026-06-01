import { useEffect, useState } from 'react';
import { indexRoadmap, Task } from '../lib/orqestra';

const STATUS_COLORS: Record<string, string> = {
  'backlog': '#9ca3af',
  'ready': '#3b82f6',
  'in-progress': '#f59e0b',
  'in-review': '#8b5cf6',
  'done': '#10b981',
  'cancelled': '#ef4444',
};

interface TaskTableProps {
  projectRoot: string;
  onTasksLoaded?: (tasks: Task[]) => void;
}

export function TaskTable({ projectRoot, onTasksLoaded }: TaskTableProps) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [warnings, setWarnings] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    indexRoadmap(projectRoot)
      .then(result => {
        setTasks(result.tasks);
        setWarnings(result.warnings);
        onTasksLoaded?.(result.tasks);
      })
      .catch(err => setError(err.message ?? String(err)));
  }, [projectRoot]);

  if (error) return <div className="error">Error: {error}</div>;

  return (
    <div>
      {warnings.map((w, i) => (
        <div key={i} className="warning">⚠ {w}</div>
      ))}
      <table style={{ borderCollapse: 'collapse', width: '100%' }}>
        <thead>
          <tr>
            <th style={{ textAlign: 'left', padding: '0.4rem 0.5rem', borderBottom: '2px solid #e5e7eb' }}>ID</th>
            <th style={{ textAlign: 'left', padding: '0.4rem 0.5rem', borderBottom: '2px solid #e5e7eb' }}>Title</th>
            <th style={{ textAlign: 'left', padding: '0.4rem 0.5rem', borderBottom: '2px solid #e5e7eb' }}>Status</th>
            <th style={{ textAlign: 'left', padding: '0.4rem 0.5rem', borderBottom: '2px solid #e5e7eb' }}>Priority</th>
            <th style={{ textAlign: 'left', padding: '0.4rem 0.5rem', borderBottom: '2px solid #e5e7eb' }}>Sprint</th>
            <th style={{ textAlign: 'left', padding: '0.4rem 0.5rem', borderBottom: '2px solid #e5e7eb' }}>Assignee</th>
            <th style={{ textAlign: 'left', padding: '0.4rem 0.5rem', borderBottom: '2px solid #e5e7eb' }}>Progress</th>
          </tr>
        </thead>
        <tbody>
          {tasks.map(task => {
            const fm = task.frontmatter;
            return (
              <tr key={fm.id} style={{ borderBottom: '1px solid #f3f4f6' }}>
                <td style={{ padding: '0.4rem 0.5rem' }}><code style={{ fontSize: '0.85em' }}>{fm.id}</code></td>
                <td style={{ padding: '0.4rem 0.5rem' }}>{fm.title}</td>
                <td style={{ padding: '0.4rem 0.5rem' }}>
                  <span style={{
                    color: STATUS_COLORS[fm.status],
                    fontWeight: 500,
                    fontSize: '0.9em',
                  }}>
                    {fm.status}
                  </span>
                </td>
                <td style={{ padding: '0.4rem 0.5rem', fontSize: '0.9em' }}>{fm.priority}</td>
                <td style={{ padding: '0.4rem 0.5rem', fontSize: '0.9em', color: '#6b7280' }}>{fm.sprint ?? '—'}</td>
                <td style={{ padding: '0.4rem 0.5rem', fontSize: '0.9em', color: '#6b7280' }}>{fm.assignee ?? '—'}</td>
                <td style={{ padding: '0.4rem 0.5rem' }}>
                  <progress value={fm.progress} max={100} />
                  {' '}{fm.progress}%
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
