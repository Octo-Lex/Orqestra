/**
 * PublicGantt — Canvas-based Gantt chart for stakeholders.
 * Read-only rendering of tasks on a timeline.
 */
import React, { useRef, useEffect } from 'react';
import { TASKS, STATUS_COLORS, type Task } from '../lib/data';

const ROW_HEIGHT = 40;
const HEADER_HEIGHT = 60;
const LABEL_WIDTH = 200;
const DAY_WIDTH = 40;

function parseDate(s: string): Date {
  return new Date(s + 'T00:00:00Z');
}

function daysBetween(a: Date, b: Date): number {
  return Math.round((b.getTime() - a.getTime()) / (1000 * 60 * 60 * 24));
}

export const PublicGantt: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const tasks = TASKS.filter(t => t.start_date && t.end_date);
    if (tasks.length === 0) return;

    // Compute date range
    const allStarts = tasks.map(t => parseDate(t.start_date!));
    const allEnds = tasks.map(t => parseDate(t.end_date!));
    const minDate = new Date(Math.min(...allStarts.map(d => d.getTime())));
    const maxDate = new Date(Math.max(...allEnds.map(d => d.getTime())));

    // Add padding
    minDate.setDate(minDate.getDate() - 3);
    maxDate.setDate(maxDate.getDate() + 3);

    const totalDays = daysBetween(minDate, maxDate);
    const width = LABEL_WIDTH + totalDays * DAY_WIDTH;
    const height = HEADER_HEIGHT + tasks.length * ROW_HEIGHT + 20;

    canvas.width = width;
    canvas.height = height;

    const ctx = canvas.getContext('2d')!;
    ctx.clearRect(0, 0, width, height);

    // Background
    ctx.fillStyle = '#0a0a1a';
    ctx.fillRect(0, 0, width, height);

    // Date headers
    ctx.fillStyle = '#475569';
    ctx.font = '11px system-ui';
    ctx.textAlign = 'center';
    for (let d = 0; d <= totalDays; d += 2) {
      const date = new Date(minDate);
      date.setDate(date.getDate() + d);
      const x = LABEL_WIDTH + d * DAY_WIDTH;
      const label = `${date.getMonth() + 1}/${date.getDate()}`;
      ctx.fillText(label, x, 20);

      // Grid line
      ctx.strokeStyle = '#1e293b';
      ctx.lineWidth = 0.5;
      ctx.beginPath();
      ctx.moveTo(x, HEADER_HEIGHT);
      ctx.lineTo(x, height);
      ctx.stroke();
    }

    // Task bars
    tasks.forEach((task, i) => {
      const y = HEADER_HEIGHT + i * ROW_HEIGHT;

      // Row background (alternating)
      if (i % 2 === 0) {
        ctx.fillStyle = '#0f172a';
        ctx.fillRect(0, y, width, ROW_HEIGHT);
      }

      // Label
      ctx.fillStyle = '#e2e8f0';
      ctx.font = '12px system-ui';
      ctx.textAlign = 'left';
      ctx.fillText(task.id, 8, y + ROW_HEIGHT / 2 + 4);

      // Bar
      const start = parseDate(task.start_date!);
      const end = parseDate(task.end_date!);
      const startDay = daysBetween(minDate, start);
      const duration = daysBetween(start, end);

      const barX = LABEL_WIDTH + startDay * DAY_WIDTH;
      const barWidth = Math.max(duration * DAY_WIDTH, DAY_WIDTH);
      const barY = y + 10;
      const barHeight = ROW_HEIGHT - 20;

      ctx.fillStyle = STATUS_COLORS[task.status];
      ctx.globalAlpha = 0.8;
      ctx.beginPath();
      ctx.roundRect(barX, barY, barWidth, barHeight, 4);
      ctx.fill();
      ctx.globalAlpha = 1;

      // Bar text
      ctx.fillStyle = '#ffffff';
      ctx.font = '10px system-ui';
      ctx.textAlign = 'left';
      ctx.fillText(task.title.substring(0, 20), barX + 6, barY + barHeight / 2 + 3);
    });

    // Header separator
    ctx.strokeStyle = '#334155';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, HEADER_HEIGHT);
    ctx.lineTo(width, HEADER_HEIGHT);
    ctx.stroke();

  }, []);

  return (
    <div>
      <h3 style={{ marginTop: 0, marginBottom: 12 }}>Gantt Timeline</h3>
      <div style={{ overflowX: 'auto', border: '1px solid #1e293b', borderRadius: 8 }}>
        <canvas ref={canvasRef} />
      </div>
    </div>
  );
};
