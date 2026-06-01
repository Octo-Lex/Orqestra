import { useRef, useEffect } from 'react';
import type { Task } from '../lib/orqestra';

interface TimeTrackingProps {
  tasks: Task[];
}

const SPRINT_COLORS = ['#3b82f6', '#f59e0b', '#10b981', '#8b5cf6', '#ef4444', '#ec4899'];

interface SprintData {
  sprint: string;
  totalEstimate: number;  // minutes
  totalLogged: number;    // minutes
  taskCount: number;
  completedCount: number;
}

function minutesToHours(m: number): string {
  const h = Math.floor(m / 60);
  const mins = m % 60;
  return mins > 0 ? `${h}h ${mins}m` : `${h}h`;
}

function groupBySprint(tasks: Task[]): SprintData[] {
  const groups = new Map<string, SprintData>();

  for (const task of tasks) {
    const sprint = task.frontmatter.sprint || 'No Sprint';
    const existing = groups.get(sprint) || {
      sprint,
      totalEstimate: 0,
      totalLogged: 0,
      taskCount: 0,
      completedCount: 0,
    };

    existing.totalEstimate += task.frontmatter.time_estimate || 0;
    existing.totalLogged += task.frontmatter.time_logged || 0;
    existing.taskCount++;
    if (task.frontmatter.status === 'done') existing.completedCount++;

    groups.set(sprint, existing);
  }

  return [...groups.values()];
}

/**
 * Draw a simple burndown-style bar chart on canvas.
 * Each bar group: estimated (light) + logged (dark).
 */
function drawBurndown(
  canvas: HTMLCanvasElement,
  sprints: SprintData[],
): void {
  const dpr = window.devicePixelRatio || 1;
  const w = canvas.parentElement?.clientWidth ?? 400;
  const h = 120;

  canvas.width = w * dpr;
  canvas.height = h * dpr;
  canvas.style.width = `${w}px`;
  canvas.style.height = `${h}px`;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  ctx.scale(dpr, dpr);

  if (sprints.length === 0) {
    ctx.fillStyle = '#9ca3af';
    ctx.font = '13px system-ui';
    ctx.textAlign = 'center';
    ctx.fillText('No time data', w / 2, h / 2);
    return;
  }

  const padding = { left: 40, right: 16, top: 16, bottom: 24 };
  const chartW = w - padding.left - padding.right;
  const chartH = h - padding.top - padding.bottom;

  const maxMinutes = Math.max(
    ...sprints.map(s => Math.max(s.totalEstimate, s.totalLogged)),
    1,
  );

  const barGroupWidth = chartW / sprints.length;
  const barWidth = barGroupWidth * 0.3;
  const gap = barGroupWidth * 0.1;

  // Y-axis scale
  const maxHours = Math.ceil(maxMinutes / 60);
  ctx.font = '10px system-ui';
  ctx.fillStyle = '#9ca3af';
  ctx.textAlign = 'right';
  for (let i = 0; i <= 4; i++) {
    const y = padding.top + chartH - (chartH * i) / 4;
    const hours = (maxHours * i) / 4;
    ctx.fillText(`${hours}h`, padding.left - 4, y + 3);

    // Grid line
    ctx.strokeStyle = '#f3f4f6';
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.moveTo(padding.left, y);
    ctx.lineTo(w - padding.right, y);
    ctx.stroke();
  }

  // Bars
  sprints.forEach((sprint, i) => {
    const x = padding.left + i * barGroupWidth + gap;

    // Estimated bar (light)
    const estH = (sprint.totalEstimate / (maxHours * 60)) * chartH;
    ctx.fillStyle = '#dbeafe';
    ctx.fillRect(x, padding.top + chartH - estH, barWidth, estH);

    // Logged bar (dark)
    const logH = (sprint.totalLogged / (maxHours * 60)) * chartH;
    ctx.fillStyle = '#3b82f6';
    ctx.fillRect(x + barWidth + 2, padding.top + chartH - logH, barWidth, logH);

    // Sprint label
    ctx.fillStyle = '#6b7280';
    ctx.font = '10px system-ui';
    ctx.textAlign = 'center';
    const label = sprint.sprint.replace('Sprint ', 'S');
    ctx.fillText(label, x + barWidth, padding.top + chartH + 14);
  });
}

export function TimeTracking({ tasks }: TimeTrackingProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sprints = groupBySprint(tasks);

  const totalEstimate = sprints.reduce((s, sp) => s + sp.totalEstimate, 0);
  const totalLogged = sprints.reduce((s, sp) => s + sp.totalLogged, 0);
  const completionPct = totalEstimate > 0 ? Math.round((totalLogged / totalEstimate) * 100) : 0;

  useEffect(() => {
    if (canvasRef.current) {
      drawBurndown(canvasRef.current, sprints);
    }
  }, [tasks]);

  return (
    <div style={{
      padding: '0.75rem',
      border: '1px solid #e5e7eb',
      borderRadius: '8px',
      background: '#fafafa',
      marginBottom: '0.75rem',
    }}>
      <h4 style={{ margin: '0 0 0.5rem', fontSize: '0.9rem' }}>Time Tracking</h4>

      {/* Summary stats */}
      <div style={{ display: 'flex', gap: '1.5rem', marginBottom: '0.75rem', fontSize: '0.85em' }}>
        <div>
          <span style={{ color: '#6b7280' }}>Estimated:</span>{' '}
          <strong>{minutesToHours(totalEstimate)}</strong>
        </div>
        <div>
          <span style={{ color: '#6b7280' }}>Logged:</span>{' '}
          <strong>{minutesToHours(totalLogged)}</strong>
        </div>
        <div>
          <span style={{ color: '#6b7280' }}>Progress:</span>{' '}
          <strong style={{ color: completionPct >= 50 ? '#10b981' : '#f59e0b' }}>
            {completionPct}%
          </strong>
        </div>
        <div>
          <span style={{ color: '#6b7280' }}>Tasks:</span>{' '}
          <strong>{tasks.length}</strong>
          <span style={{ color: '#9ca3af', marginLeft: '0.25rem' }}>
            ({sprints.reduce((s, sp) => s + sp.completedCount, 0)} done)
          </span>
        </div>
      </div>

      {/* Per-sprint breakdown */}
      <div style={{ display: 'flex', gap: '0.75rem', marginBottom: '0.75rem', flexWrap: 'wrap' }}>
        {sprints.map((sp, i) => {
          const pct = sp.totalEstimate > 0 ? Math.round((sp.totalLogged / sp.totalEstimate) * 100) : 0;
          return (
            <div key={sp.sprint} style={{
              padding: '0.4rem 0.6rem',
              borderLeft: `3px solid ${SPRINT_COLORS[i % SPRINT_COLORS.length]}`,
              background: 'white',
              borderRadius: '4px',
              fontSize: '0.8em',
            }}>
              <div style={{ fontWeight: 600, marginBottom: '0.15rem' }}>{sp.sprint}</div>
              <div style={{ color: '#6b7280' }}>
                {minutesToHours(sp.totalLogged)} / {minutesToHours(sp.totalEstimate)} ({pct}%)
              </div>
              <div style={{ color: '#9ca3af', fontSize: '0.9em' }}>
                {sp.completedCount}/{sp.taskCount} tasks done
              </div>
            </div>
          );
        })}
      </div>

      {/* Burndown chart */}
      <canvas ref={canvasRef} style={{ width: '100%', height: '120px' }} />

      {/* Chart legend */}
      <div style={{ display: 'flex', gap: '1rem', marginTop: '0.25rem', fontSize: '0.7em', color: '#9ca3af' }}>
        <span style={{ display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
          <span style={{ display: 'inline-block', width: 10, height: 10, backgroundColor: '#dbeafe', borderRadius: 2 }} />
          Estimated
        </span>
        <span style={{ display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
          <span style={{ display: 'inline-block', width: 10, height: 10, backgroundColor: '#3b82f6', borderRadius: 2 }} />
          Logged
        </span>
      </div>
    </div>
  );
}
