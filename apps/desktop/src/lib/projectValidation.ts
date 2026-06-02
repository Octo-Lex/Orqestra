import { invoke } from '@tauri-apps/api/core';

export interface ProjectValidationResult {
  project_root: string;
  status: 'valid' | 'repairable' | 'not_orqestra' | 'invalid' | 'inaccessible';
  detected: ProjectDetectedState;
  errors: ProjectValidationIssue[];
  warnings: ProjectValidationIssue[];
  suggested_actions: SuggestedAction[];
}

export interface ProjectDetectedState {
  is_git_repo: boolean;
  has_roadmap_dir: boolean;
  has_index_md: boolean;
  task_count: number;
  malformed_task_count: number;
  has_orqestra_dir: boolean;
  has_orqestra_toml: boolean;
  has_dashboard_json: boolean;
}

export interface ProjectValidationIssue {
  code: string;
  path?: string;
  message: string;
  severity: string;
}

export interface SuggestedAction {
  id: string;
  label: string;
  description: string;
  kind: string;
  safe: boolean;
}

export interface SampleProjectResult {
  path: string;
  created: boolean;
  task_count: number;
}

export async function validateProject(projectRoot: string): Promise<ProjectValidationResult> {
  return invoke<ProjectValidationResult>('validate_project_cmd', { projectRoot });
}

export async function createSampleProject(destination?: string): Promise<SampleProjectResult> {
  return invoke<SampleProjectResult>('create_sample_project_cmd', { destination: destination || null });
}
