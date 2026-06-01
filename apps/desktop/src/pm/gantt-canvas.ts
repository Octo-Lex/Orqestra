/**
 * gantt-canvas — Low-level Canvas drawing for Gantt view.
 *
 * Renders task bars, dependency arrows, date grid, and tooltips.
 * Pure rendering — no React dependency.
 */

import type { Task } from '../lib/orqestra';
import { topoSort, type ScheduledTask } from './SmartScheduler';

/** Status → bar fill color */
const STATUS_BAR_COLORS: Record<string, string> = {
  'backlog': '#d1d5db',
  'ready': '#93c5fd',
  'in-progress': '#fbbf24',
  'in-review': '#c4b5fd',
  'done': '#6ee7b7',
  'cancelled': '#fca5a5',
};

const STATUS_PROGRESS_COLORS: Record<string, string> = {
  'backlog': '#9ca3af',
  'ready': '#3b82f6',
  'in-progress': '#f59e0b',
  'in-review': '#8b5cf6',
  'done': '#10b981',
  'cancelled': '#ef4444',
};

const ROW_HEIGHT = 36;
const HEADER_HEIGHT = 32;
const LEFT_MARGIN = 180;
const DAY_WIDTH = 32;

interface DateRange {
  start: Date;
  end: Date;
}

function daysBetween(a: Date, b: Date): number {
  return Math.round((b.getTime() - a.getTime()) / (1000 * 60 * 60 * 24));
}

function dateToX(date: Date, range: DateRange): number {
  return LEFT_MARGIN + daysBetween(range.start, date) * DAY_WIDTH;
}

function xToDate(x: number, range: DateRange): Date {
  const days = Math.round((x - LEFT_MARGIN) / DAY_WIDTH);
  const d = new Date(range.start);
  d.setDate(d.getDate() + days);
  return d;
}

export interface GanttLayout {
  canvasWidth: number;
  canvasHeight: number;
  range: DateRange;
  rows: Array<{
    task: Task;
    y: number;
    barX: number;
    barWidth: number;
    progressWidth: number;
    scheduled?: ScheduledTask;
  }>;
}

export function computeLayout(
  tasks: Task[],
  scheduled: ScheduledTask[],
  range: DateRange,
): GanttLayout {
  const sorted = topoSort(tasks);
  const schedMap = new Map(scheduled.map(s => [s.task.frontmatter.id, s]));

  const rows = sorted.map((task, i) => {
    const fm = task.frontmatter;
    const sched = schedMap.get(fm.id);
    const startDate = sched ? sched.computedStart : (fm.start_date ? new Date(fm.start_date + 'T00:00:00Z') : new Date());
    const endDate = sched ? sched.computedEnd : (fm.due_date ? new Date(fm.due_date + 'T00:00:00Z') : new Date(startDate.getTime() + 86400000));

    const barX = dateToX(startDate, range);
    const barEndX = dateToX(endDate, range);
    const barWidth = Math.max(DAY_WIDTH, barEndX - barX + DAY_WIDTH);
    const progressWidth = (barWidth * fm.progress) / 100;

    return {
      task,
      y: HEADER_HEIGHT + i * ROW_HEIGHT + 4,
      barX,
      barWidth,
      progressWidth,
      scheduled: sched,
    };
  });

  const canvasWidth = Math.max(800, LEFT_MARGIN + daysBetween(range.start, range.end) * DAY_WIDTH + 40);
  const canvasHeight = HEADER_HEIGHT + sorted.length * ROW_HEIGHT + 20;

  return { canvasWidth, canvasHeight, range, rows };
}

export function drawGantt(
  ctx: CanvasRenderingContext2D,
  layout: GanttLayout,
  hoverRow: number | null,
): void {
  const { canvasWidth, canvasHeight, range, rows } = layout;

  ctx.clearRect(0, 0, canvasWidth, canvasHeight);

  // Background
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);

  // Date grid
  const totalDays = daysBetween(range.start, range.end);
  ctx.font = '11px system-ui';
  ctx.textBaseline = 'middle';

  for (let d = 0; d <= totalDays; d++) {
    const x = LEFT_MARGIN + d * DAY_WIDTH;
    const date = new Date(range.start);
    date.setDate(date.getDate() + d);

    // Weekend shading
    const dayOfWeek = date.getDay();
    if (dayOfWeek === 0 || dayOfWeek === 6) {
      ctx.fillStyle = '#f3f4f6';
      ctx.fillRect(x, HEADER_HEIGHT, DAY_WIDTH, canvasHeight - HEADER_HEIGHT);
    }

    // Grid line
    ctx.strokeStyle = '#e5e7eb';
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.moveTo(x, HEADER_HEIGHT);
    ctx.lineTo(x, canvasHeight);
    ctx.stroke();

    // Date label (show every 3 days to avoid crowding)
    if (d % 3 === 0) {
      ctx.fillStyle = '#6b7280';
      const label = `${date.getMonth() + 1}/${date.getDate()}`;
      ctx.fillText(label, x + 2, HEADER_HEIGHT / 2);
    }
  }

  // Draw dependency arrows BEFORE bars so bars overlay them
  const idToRow = new Map(rows.map((r, i) => [r.task.frontmatter.id, i]));
  ctx.lineWidth = 1.5;

  for (let i = 0; i < rows.length; i++) {
    const { task, barX, y } = rows[i];
    for (const depId of task.frontmatter.dependencies) {
      const depIdx = idToRow.get(depId);
      if (depIdx === undefined) continue;
      const dep = rows[depIdx];

      const fromX = dep.barX + dep.barWidth;
      const fromY = dep.y + ROW_HEIGHT / 2 - 4;
      const toX = barX;
      const toY = y + ROW_HEIGHT / 2 - 4;

      // Arrow color: green if dep done, orange if not
      const depStatus = dep.task.frontmatter.status;
      ctx.strokeStyle = depStatus === 'done' ? '#86efac' : '#fdba74';
      ctx.beginPath();
      ctx.moveTo(fromX, fromY);
      const midX = (fromX + toX) / 2;
      ctx.bezierCurveTo(midX, fromY, midX, toY, toX, toY);
      ctx.stroke();

      // Arrowhead
      ctx.fillStyle = ctx.strokeStyle;
      ctx.beginPath();
      ctx.moveTo(toX, toY);
      ctx.lineTo(toX - 6, toY - 4);
      ctx.lineTo(toX - 6, toY + 4);
      ctx.closePath();
      ctx.fill();
    }
  }

  // Draw task bars
  for (let i = 0; i < rows.length; i++) {
    const { task, y, barX, barWidth, progressWidth } = rows[i];
    const fm = task.frontmatter;
    const isHover = hoverRow === i;
    const barY = y + 2;
    const barH = ROW_HEIGHT - 8;

    // Task ID label (left side)
    ctx.fillStyle = isHover ? '#111' : '#374151';
    ctx.font = `${isHover ? 'bold ' : ''}12px monospace`;
    ctx.textBaseline = 'middle';
    const idText = fm.id.replace('TASK-2026-', '#');
    ctx.fillText(idText, 4, y + ROW_HEIGHT / 2 - 4);

    // Title (truncated)
    ctx.font = '11px system-ui';
    ctx.fillStyle = '#6b7280';
    const titleText = fm.title.length > 20 ? fm.title.slice(0, 18) + '...' : fm.title;
    ctx.fillText(titleText, 60, y + ROW_HEIGHT / 2 - 4);

    // Bar background
    ctx.fillStyle = STATUS_BAR_COLORS[fm.status] || '#e5e7eb';
    ctx.beginPath();
    ctx.roundRect(barX, barY, barWidth, barH, 4);
    ctx.fill();

    // Progress overlay
    if (progressWidth > 0) {
      ctx.fillStyle = STATUS_PROGRESS_COLORS[fm.status] || '#3b82f6';
      ctx.beginPath();
      ctx.roundRect(barX, barY, Math.min(progressWidth, barWidth), barH, 4);
      ctx.fill();
    }

    // Hover highlight
    if (isHover) {
      ctx.strokeStyle = '#2563eb';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.roundRect(barX, barY, barWidth, barH, 4);
      ctx.stroke();
    }

    // Overdue indicator
    if (fm.due_date && fm.status !== 'done' && fm.status !== 'cancelled') {
      const dueDate = new Date(fm.due_date + 'T00:00:00Z');
      const now = new Date();
      if (dueDate < now) {
        const dueX = dateToX(dueDate, range);
        ctx.strokeStyle = '#ef4444';
        ctx.lineWidth = 2;
        ctx.setLineDash([4, 2]);
        ctx.beginPath();
        ctx.moveTo(dueX, barY - 2);
        ctx.lineTo(dueX, barY + barH + 2);
        ctx.stroke();
        ctx.setLineDash([]);
      }
    }
  }

  // Left margin separator
  ctx.strokeStyle = '#d1d5db';
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(LEFT_MARGIN, 0);
  ctx.lineTo(LEFT_MARGIN, canvasHeight);
  ctx.stroke();
}

/**
 * Hit-test: given mouse coords, return the row index or null.
 */
export function hitTest(x: number, y: number, layout: GanttLayout): number | null {
  if (x < LEFT_MARGIN) return null;
  const rowIdx = Math.floor((y - HEADER_HEIGHT) / ROW_HEIGHT);
  if (rowIdx < 0 || rowIdx >= layout.rows.length) return null;
  return rowIdx;
}

export { dateToX, xToDate, DAY_WIDTH, LEFT_MARGIN, ROW_HEIGHT, HEADER_HEIGHT };
