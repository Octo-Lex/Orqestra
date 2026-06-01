import { invoke } from '@tauri-apps/api/core';

export interface GitResult {
  success: boolean;
  stdout: string;
  stderr: string;
}

export async function storePat(pat: string): Promise<void> {
  return invoke('store_pat', { pat });
}

export async function hasStoredPat(): Promise<boolean> {
  return invoke<boolean>('has_stored_pat');
}

export async function gitPullRoadmap(projectRoot: string): Promise<GitResult> {
  return invoke<GitResult>('git_pull_roadmap', { projectRoot });
}

export async function gitPushRoadmap(projectRoot: string): Promise<GitResult> {
  return invoke<GitResult>('git_push_roadmap', { projectRoot });
}
