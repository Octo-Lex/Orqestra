import { invoke } from '@tauri-apps/api/core';

export interface GitResult {
  success: boolean;
  stdout: string;
  stderr: string;
}

// ---------------------------------------------------------------------------
// PAT handling — credentials are managed exclusively by the Rust backend
// via the OS keychain. The frontend never persists PATs to disk.
//
// v2.14.1: Removed credentials.json plaintext PAT storage.
// Rust/keychain is the only credential persistence authority.
// Old credentials.json files are no longer read or written.
// ---------------------------------------------------------------------------

interface TokenStatus {
  exists: boolean;
  provider: string;
  label: string;
  last_updated: string | null;
}

/// Check whether a PAT is stored in the OS keychain.
export async function hasStoredPat(): Promise<boolean> {
  try {
    const status = await invoke<TokenStatus>('get_github_token_status_cmd');
    return status.exists;
  } catch {
    return false;
  }
}

/// Store a PAT in the OS keychain (never written to disk by frontend).
export async function storePat(pat: string): Promise<void> {
  await invoke('save_github_token_cmd', { token: pat });
}

/// Clear the stored PAT from the OS keychain.
export async function clearStoredPat(): Promise<void> {
  await invoke('delete_github_token_cmd');
}

// ---------------------------------------------------------------------------
// Git commands — PAT managed internally by Rust via OS keychain
// ---------------------------------------------------------------------------

export async function gitPullRoadmap(projectRoot: string): Promise<GitResult> {
  return invoke<GitResult>('git_pull_roadmap', { projectRoot, pat: '' });
}

export async function gitPushRoadmap(projectRoot: string): Promise<GitResult> {
  return invoke<GitResult>('git_push_roadmap', { projectRoot, pat: '' });
}
