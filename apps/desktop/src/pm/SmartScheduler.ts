/**
 * SmartScheduler — dependency-aware date propagation.
 *
 * Given a list of tasks with dependencies, computes adjusted dates
 * when a blocker moves. Uses topological sort on the dependency DAG
 * to propagate start_date forward.
 *
 * Phase 2: pure TypeScript, no Rust dependency.
 */

import type { Task } from '../lib/orqestra';

/** Weekend check: Saturday=6, Sunday=0 */
function isWeekend(d: Date): boolean {
  const day = d.getDay();
  return day === 0 || day === 6;
}

/** Add N working days to a date (skips weekends) */
function addWorkingDays(start: Date, days: number): Date {
  const result = new Date(start);
  let remaining = days;
  while (remaining > 0) {
    result.setDate(result.getDate() + 1);
    if (!isWeekend(result)) remaining--;
  }
  return result;
}

/** Parse date string (YYYY-MM-DD or ISO) to Date at midnight UTC */
function parseDate(s: string | null): Date | null {
  if (!s) return null;
  const d = new Date(s + 'T00:00:00Z');
  return isNaN(d.getTime()) ? null : d;
}

/** Format Date to YYYY-MM-DD */
function formatDate(d: Date): string {
  return d.toISOString().split('T')[0];
}

/** Convert time_estimate (minutes) to working days (8h = 1 day) */
function minutesToWorkDays(minutes: number | null): number {
  if (minutes === null || minutes <= 0) return 1; // default 1 day
  return Math.max(1, Math.ceil(minutes / 480));
}

export interface ScheduledTask {
  task: Task;
  computedStart: Date;
  computedEnd: Date;
  isOverdue: boolean;
}

/**
 * Topological sort of tasks by dependency DAG.
 * Tasks with no dependencies come first.
 */
export function topoSort(tasks: Task[]): Task[] {
  const idMap = new Map(tasks.map(t => [t.frontmatter.id, t]));
  const visited = new Set<string>();
  const visiting = new Set<string>();
  const result: Task[] = [];

  function visit(id: string): void {
    if (visited.has(id)) return;
    if (visiting.has(id)) return; // cycle — skip
    visiting.add(id);

    const task = idMap.get(id);
    if (task) {
      for (const dep of task.frontmatter.dependencies) {
        visit(dep);
      }
      result.push(task);
    }
    visiting.delete(id);
    visited.add(id);
  }

  for (const t of tasks) {
    visit(t.frontmatter.id);
  }

  return result;
}

/**
 * Auto-schedule: propagate dates through the dependency DAG.
 *
 * Rules:
 * 1. Tasks with no dependencies keep their start_date (or today if null)
 * 2. A dependent task's start_date = max(dependency end_date + 1)
 * 3. Duration = time_estimate converted to working days (default 1 day)
 * 4. Weekends are skipped
 *
 * Returns tasks with adjusted start/end dates, preserving original data
 * for tasks that don't need adjustment.
 */
export function autoSchedule(tasks: Task[]): ScheduledTask[] {
  const sorted = topoSort(tasks);
  const endDates = new Map<string, Date>();

  return sorted.map(task => {
    const fm = task.frontmatter;

    // Base start: the latest end_date of all dependencies
    let start = parseDate(fm.start_date) ?? new Date();

    for (const depId of fm.dependencies) {
      const depEnd = endDates.get(depId);
      if (depEnd) {
        const nextWorkDay = addWorkingDays(depEnd, 1);
        if (nextWorkDay > start) {
          start = nextWorkDay;
        }
      }
    }

    // Duration from estimate
    const workDays = minutesToWorkDays(fm.time_estimate);
    const end = addWorkingDays(start, workDays - 1); // inclusive

    endDates.set(fm.id, end);

    const dueDate = parseDate(fm.due_date);
    const isOverdue = dueDate !== null && end > dueDate;

    return { task, computedStart: start, computedEnd: end, isOverdue };
  });
}

/**
 * Propagate a single blocker's date change through the DAG.
 * Returns only tasks whose dates changed.
 */
export function propagateBlockerMove(
  tasks: Task[],
  blockerId: string,
  newEndDate: Date,
): Map<string, { oldStart: string | null; newStart: string }> {
  const changes = new Map<string, { oldStart: string | null; newStart: string }>();
  const scheduled = autoSchedule(tasks);

  // Build the new end dates map
  const newEndDates = new Map<string, Date>();
  newEndDates.set(blockerId, newEndDate);

  // Re-propagate from the blocker forward
  for (const st of scheduled) {
    const fm = st.task.frontmatter;
    if (fm.dependencies.includes(blockerId) || [...changes.keys()].some(c => fm.dependencies.includes(c))) {
      let newStart = newEndDate;
      for (const depId of fm.dependencies) {
        const depEnd = newEndDates.get(depId);
        if (depEnd) {
          const next = addWorkingDays(depEnd, 1);
          if (next > newStart) newStart = next;
        }
      }

      const workDays = minutesToWorkDays(fm.time_estimate);
      const newEnd = addWorkingDays(newStart, workDays - 1);
      newEndDates.set(fm.id, newEnd);

      const oldStart = fm.start_date;
      const newStartStr = formatDate(newStart);
      if (oldStart !== newStartStr) {
        changes.set(fm.id, { oldStart, newStart: newStartStr });
      }
    }
  }

  return changes;
}

/**
 * Compute the overall date range for all tasks.
 * Returns [minDate, maxDate] spanning all task dates.
 */
export function computeDateRange(
  tasks: Task[],
  extraWeeks: number = 1,
): { start: Date; end: Date } {
  const now = new Date();
  let minDate = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  let maxDate = new Date(minDate);
  maxDate.setDate(maxDate.getDate() + 28); // default 4-week view

  for (const task of tasks) {
    const s = parseDate(task.frontmatter.start_date);
    const d = parseDate(task.frontmatter.due_date);
    if (s && s < minDate) minDate = new Date(s);
    if (d && d > maxDate) maxDate = new Date(d);
  }

  // Add padding
  minDate.setDate(minDate.getDate() - 3);
  maxDate.setDate(maxDate.getDate() + 7 * extraWeeks);

  return { start: minDate, end: maxDate };
}
