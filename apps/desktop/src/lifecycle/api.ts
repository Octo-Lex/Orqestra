//! Orqestra Development Lifecycle — Frontend API calls

import { invoke } from '@tauri-apps/api/core';
import type { LifecycleState } from './types';

export async function lifecycleInit(projectRoot: string): Promise<any> {
  return invoke('lifecycle_init_cmd', { projectRoot });
}

export async function lifecycleGetState(projectRoot: string): Promise<LifecycleState> {
  return invoke('lifecycle_get_state_cmd', { projectRoot });
}

export async function lifecycleRequestAdvance(
  projectRoot: string,
  featureId: string | null = null,
): Promise<any> {
  return invoke('lifecycle_request_advance_cmd', { projectRoot, featureId });
}

export async function lifecycleApproveGate(
  projectRoot: string,
  featureId: string | null = null,
): Promise<any> {
  return invoke('lifecycle_approve_gate_cmd', { projectRoot, featureId });
}

export async function lifecycleRejectGate(
  projectRoot: string,
  reason: string,
  featureId: string | null = null,
): Promise<any> {
  return invoke('lifecycle_reject_gate_cmd', { projectRoot, featureId, reason });
}

export async function lifecycleRecordArtifact(
  projectRoot: string,
  artifactType: string,
  path: string,
  featureId: string | null = null,
  actor = 'human',
): Promise<any> {
  return invoke('lifecycle_record_artifact_cmd', {
    projectRoot,
    artifactType,
    path,
    featureId,
    actor,
  });
}

export async function lifecycleReadArtifact(
  projectRoot: string,
  path: string,
): Promise<{ ok: boolean; content: string }> {
  return invoke('lifecycle_read_artifact_cmd', { projectRoot, path });
}

export async function lifecycleWriteArtifact(
  projectRoot: string,
  path: string,
  content: string,
  artifactType?: string,
  featureId?: string | null,
  actor = 'human',
): Promise<any> {
  return invoke('lifecycle_write_artifact_cmd', {
    projectRoot,
    path,
    content,
    artifactType: artifactType ?? null,
    featureId: featureId ?? null,
    actor,
  });
}
