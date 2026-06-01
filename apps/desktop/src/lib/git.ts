import { invoke } from '@tauri-apps/api/core';
import { load, type Store } from '@tauri-apps/plugin-store';

export interface GitResult {
  success: boolean;
  stdout: string;
  stderr: string;
}

// ---------------------------------------------------------------------------
// PAT persistence — store is ONLY used for cross-session survival.
// Within a session, the PAT lives in React state and is passed directly.
// ---------------------------------------------------------------------------

const STORE_FILE = 'credentials.json';
const PAT_KEY = 'github_pat';

let _store: Store | null = null;

async function getStore(): Promise<Store> {
  if (!_store) {
    _store = await load(STORE_FILE, { autoSave: false, defaults: {} });
  }
  return _store;
}

/// Persist PAT to disk for next app launch.
export async function persistPat(pat: string): Promise<void> {
  const store = await getStore();
  await store.set(PAT_KEY, pat);
  await store.save();
}

/// Load PAT from disk (used on app startup to pre-fill state).
export async function loadPersistedPat(): Promise<string | null> {
  try {
    const store = await getStore();
    const val = await store.get<string>(PAT_KEY);
    return val ?? null;
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Git commands — PAT always passed from caller (React state)
// ---------------------------------------------------------------------------

export async function gitPullRoadmap(projectRoot: string, pat: string): Promise<GitResult> {
  return invoke<GitResult>('git_pull_roadmap', { projectRoot, pat });
}

export async function gitPushRoadmap(projectRoot: string, pat: string): Promise<GitResult> {
  return invoke<GitResult>('git_push_roadmap', { projectRoot, pat });
}
