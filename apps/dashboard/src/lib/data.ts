/**
 * Dashboard data types.
 * Primary source: /orqestra-roadmap.json (generated from roadmap/ by CLI).
 * Fallback: hardcoded mock data for development without generation.
 */

export type TaskStatus = 'todo' | 'in-progress' | 'review' | 'done' | 'blocked' | 'backlog' | 'ready' | 'in-review' | 'cancelled';

export type Task = {
  id: string;
  title: string;
  status: TaskStatus;
  priority: string;
  assignee?: string | null;
  sprint?: string | null;
  epic?: string | null;
  start_date?: string | null;
  due_date?: string | null;
  progress: number;
  dependencies: string[];
  blocks: string[];
  labels: string[];
};

export type Sprint = {
  id: string;
  title?: string;
  start_date?: string;
  end_date?: string;
  status?: string;
  tasks: string[];
};

export type RoadmapData = {
  generated_at: string;
  source: {
    repo: string;
    branch: string;
    commit: string;
  };
  summary: {
    total_tasks: number;
    done: number;
    backlog: number;
    in_progress: number;
    blocked: number;
    ready: number;
  };
  sprints: Sprint[];
  tasks: Task[];
};

export const STATUS_COLORS: Record<string, string> = {
  'todo': '#6b7280',
  'backlog': '#6b7280',
  'ready': '#8b5cf6',
  'in-progress': '#3b82f6',
  'in-review': '#f59e0b',
  'review': '#f59e0b',
  'done': '#22c55e',
  'blocked': '#ef4444',
  'cancelled': '#64748b',
};

export const PRIORITY_COLORS: Record<string, string> = {
  'Low': '#6b7280',
  'Medium': '#3b82f6',
  'High': '#f59e0b',
  'Critical': '#ef4444',
};

/**
 * Fetch roadmap data from the generated JSON artifact.
 * Falls back to empty state if the JSON is unavailable.
 */
export async function fetchRoadmapData(): Promise<RoadmapData | null> {
  try {
    const resp = await fetch('/orqestra-roadmap.json');
    if (!resp.ok) return null;
    return await resp.json();
  } catch {
    return null;
  }
}
