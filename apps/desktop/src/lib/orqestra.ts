import { invoke } from '@tauri-apps/api/core';

export type TaskStatus =
  | 'backlog' | 'ready' | 'in-progress'
  | 'in-review' | 'done' | 'cancelled';

export type Priority = 'Critical' | 'High' | 'Medium' | 'Low';

export interface TaskFrontmatter {
  id: string;
  title: string;
  status: TaskStatus;
  priority: Priority;
  sprint: string | null;
  epic: string | null;
  assignee: string | null;
  progress: number;
  dependencies: string[];
  blocks: string[];
  labels: string[];
  time_estimate: number | null;
  time_logged: number | null;
  due_date: string | null;
  start_date: string | null;
  created: string;
  updated: string;
}

export interface Task {
  frontmatter: TaskFrontmatter;
  body: {
    context: string | null;
    acceptance_criteria: Array<{ text: string; completed: boolean }>;
    agent_notes: string | null;
    raw: string;
  };
  source_path: string;
}

export interface IndexRoadmapResult {
  tasks: Task[];
  warnings: string[];
}

export interface CommandError {
  code: string;
  message: string;
}

export async function indexRoadmap(projectRoot: string): Promise<IndexRoadmapResult> {
  return invoke<IndexRoadmapResult>('index_roadmap_cmd', { projectRoot });
}

export async function getTask(projectRoot: string, taskId: string): Promise<Task | null> {
  return invoke<Task | null>('get_task', { projectRoot, taskId });
}
