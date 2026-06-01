import { invoke } from '@tauri-apps/api/core';

export interface SemanticCommitResult {
  hash: string;
  stub_path: string;
}

export interface BackfillResult {
  confidence: number;
  intent_summary: string;
  reasoning_trace_id: string;
}

export async function semanticCommit(
  projectRoot: string,
  message: string,
  taskIds: string[],
): Promise<SemanticCommitResult> {
  return invoke<SemanticCommitResult>('semantic_commit_cmd', {
    projectRoot,
    message,
    taskIds,
  });
}

export async function backfill(
  projectRoot: string,
  commitHash: string,
  aiServiceUrl: string,
): Promise<BackfillResult> {
  return invoke<BackfillResult>('backfill_cmd', {
    projectRoot,
    commitHash,
    aiServiceUrl,
  });
}
