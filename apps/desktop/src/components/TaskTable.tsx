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

export function TaskTable({ projectRoot }: { projectRoot: string }) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [warnings, setWarnings] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    indexRoadmap(projectRoot)
      .then(result => {
        setTasks(result.tasks);
        setWarnings(result.warnings);
      })
      .catch(err => setError(err.message ?? String(err)));
  }, [projectRoot]);

  if (error) return <div className="error">Error: {error}</div>;

  return (
    <div>
      {warnings.map((w, i) => (
        <div key={i} className="warning">⚠ {w}</div>
      ))}
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>Title</th>
            <th>Status</th>
            <th>Priority</th>
            <th>Sprint</th>
            <th>Assignee</th>
            <th>Progress</th>
          </tr>
        </thead>
        <tbody>
          {tasks.map(task => {
            const fm = task.frontmatter;
            return (
              <tr key={fm.id}>
                <td><code>{fm.id}</code></td>
                <td>{fm.title}</td>
                <td>
                  <span style={{ color: STATUS_COLORS[fm.status] }}>
                    {fm.status}
                  </span>
                </td>
                <td>{fm.priority}</td>
                <td>{fm.sprint ?? '—'}</td>
                <td>{fm.assignee ?? '—'}</td>
                <td>
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
