import { useRef, useEffect, useState, useCallback } from 'react';
import type { Task } from '../lib/orqestra';
import { autoSchedule, computeDateRange } from './SmartScheduler';
import { computeLayout, drawGantt, hitTest } from './gantt-canvas';
import type { GanttLayout } from './gantt-canvas';

interface GanttViewProps {
  tasks: Task[];
  onAutoSchedule: () => void;
}

export function GanttView({ tasks, onAutoSchedule }: GanttViewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [hoverRow, setHoverRow] = useState<number | null>(null);
  const [layout, setLayout] = useState<GanttLayout | null>(null);

  const scheduled = autoSchedule(tasks);
  const range = computeDateRange(tasks);

  // Compute layout and draw
  useEffect(() => {
    const newLayout = computeLayout(tasks, scheduled, range);
    setLayout(newLayout);

    const canvas = canvasRef.current;
    if (!canvas) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = newLayout.canvasWidth * dpr;
    canvas.height = newLayout.canvasHeight * dpr;
    canvas.style.width = `${newLayout.canvasWidth}px`;
    canvas.style.height = `${newLayout.canvasHeight}px`;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.scale(dpr, dpr);

    drawGantt(ctx, newLayout, hoverRow);
  }, [tasks, hoverRow]);

  // Mouse hover handler
  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas || !layout) return;

    const rect = canvas.getBoundingClientRect();
    const scaleX = layout.canvasWidth / rect.width;
    const scaleY = layout.canvasHeight / rect.height;
    const x = (e.clientX - rect.left) * scaleX;
    const y = (e.clientY - rect.top) * scaleY;

    const row = hitTest(x, y, layout);
    setHoverRow(row);
  }, [layout]);

  const handleMouseLeave = useCallback(() => {
    setHoverRow(null);
  }, []);

  // Tooltip
  const tooltipTask = hoverRow !== null && layout ? layout.rows[hoverRow]?.task : null;

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' }}>
        <h3 style={{ margin: 0, fontSize: '1rem' }}>Gantt View</h3>
        <button onClick={onAutoSchedule} style={{ fontSize: '0.85em', padding: '0.25rem 0.75rem' }}>
          Auto-Schedule
        </button>
      </div>

      <div
        ref={containerRef}
        style={{
          overflowX: 'auto',
          overflowY: 'auto',
          maxHeight: '400px',
          border: '1px solid #e5e7eb',
          borderRadius: '4px',
          position: 'relative',
        }}
      >
        <canvas
          ref={canvasRef}
          onMouseMove={handleMouseMove}
          onMouseLeave={handleMouseLeave}
          style={{ display: 'block', cursor: hoverRow !== null ? 'pointer' : 'default' }}
        />

        {/* Hover tooltip */}
        {tooltipTask && (
          <div
            style={{
              position: 'absolute',
              top: '8px',
              right: '8px',
              background: 'white',
              border: '1px solid #d1d5db',
              borderRadius: '6px',
              padding: '0.5rem 0.75rem',
              fontSize: '0.8em',
              boxShadow: '0 2px 8px rgba(0,0,0,0.1)',
              zIndex: 10,
              maxWidth: '250px',
            }}
          >
            <div style={{ fontWeight: 'bold', marginBottom: '0.25rem' }}>
              {tooltipTask.frontmatter.id}
            </div>
            <div>{tooltipTask.frontmatter.title}</div>
            <div style={{ color: '#6b7280', marginTop: '0.25rem' }}>
              {tooltipTask.frontmatter.status} · {tooltipTask.frontmatter.priority}
            </div>
            {tooltipTask.frontmatter.start_date && (
              <div style={{ color: '#6b7280', fontSize: '0.9em' }}>
                {tooltipTask.frontmatter.start_date}
                {tooltipTask.frontmatter.due_date ? ` → ${tooltipTask.frontmatter.due_date}` : ''}
              </div>
            )}
            {tooltipTask.frontmatter.dependencies.length > 0 && (
              <div style={{ color: '#9ca3af', fontSize: '0.85em', marginTop: '0.25rem' }}>
                Depends on: {tooltipTask.frontmatter.dependencies.join(', ')}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Legend */}
      <div style={{ display: 'flex', gap: '1rem', marginTop: '0.5rem', fontSize: '0.75em', color: '#6b7280' }}>
        {[
          ['backlog', 'Backlog'],
          ['ready', 'Ready'],
          ['in-progress', 'In Progress'],
          ['in-review', 'In Review'],
          ['done', 'Done'],
          ['cancelled', 'Cancelled'],
        ].map(([status, label]) => {
          const colors: Record<string, string> = {
            'backlog': '#9ca3af', 'ready': '#3b82f6', 'in-progress': '#f59e0b',
            'in-review': '#8b5cf6', 'done': '#10b981', 'cancelled': '#ef4444',
          };
          return (
            <span key={status} style={{ display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
              <span style={{
                display: 'inline-block', width: 10, height: 10,
                borderRadius: 2, backgroundColor: colors[status],
              }} />
              {label}
            </span>
          );
        })}
      </div>
    </div>
  );
}
