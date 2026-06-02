import { invoke } from '@tauri-apps/api/core';

export interface ReadinessReport {
  generated_at: string;
  app: AppReadiness;
  project: ProjectReadiness | null;
  local_tools: ToolReadiness[];
  ai: AiReadiness;
  credentials: CredentialReadiness;
  dashboard: DashboardReadiness;
  release_artifacts: ReleaseArtifactReadiness[];
  warnings: ReadinessWarning[];
}

export interface AppReadiness {
  version: string;
  git_sha: string | null;
  tauri_commands_registered: number | null;
  platform: string;
}

export interface ProjectReadiness {
  root: string;
  status: string;
  task_count: number;
}

export interface ToolReadiness {
  tool: string;
  status: string;
  version?: string;
  required_for: string[];
}

/** Spec-aligned AI readiness status (v1.0.4 WS-B). */
export interface AiReadinessStatus {
  available: boolean;
  mode: 'real-ai' | 'degraded' | 'mock' | 'unavailable';
  provider: 'zai' | 'none';
  model: string | null;
  requires_env: 'ZAI_API_KEY' | null;
  message: string;
}

/** Map raw AI readiness from Rust command into spec-aligned DTO. */
export function mapAiReadiness(ai: AiReadiness): AiReadinessStatus {
  const isReal = ai.mode === 'real' && ai.service_status === 'reachable';
  const isDegraded = ai.mode === 'degraded_mock' && ai.service_status === 'reachable';

  const mode: AiReadinessStatus['mode'] = isReal
    ? 'real-ai'
    : isDegraded
      ? 'degraded'
      : ai.service_status === 'unreachable'
        ? 'unavailable'
        : 'mock';

  return {
    available: isReal,
    mode,
    provider: isReal || isDegraded ? 'zai' : 'none',
    model: isReal ? 'glm-5.1' : null,
    requires_env: !isReal ? 'ZAI_API_KEY' : null,
    message: isReal
      ? 'AI service running with real model'
      : isDegraded
        ? 'AI service running in degraded mode (no API key)'
        : 'AI service unavailable',
  };
}

export interface AiReadiness {
  service_status: string;
  health_url: string;
  api_key_status: string;
  mode: string;
  last_error?: string;
}

export interface CredentialReadiness {
  github_token: string;
  provider: string;
  last_error?: string;
}

export interface DashboardReadiness {
  local_json: string;
  live_url_status: string;
  source_commit?: string;
  cloudflare_secrets: string;
}

export interface ReleaseArtifactReadiness {
  platform: string;
  status: string;
  artifact_name?: string;
  limitation?: string;
}

export interface ReadinessWarning {
  code: string;
  severity: string;
  message: string;
  recovery: string;
}

export async function getReadiness(projectRoot?: string): Promise<ReadinessReport> {
  return invoke<ReadinessReport>('get_readiness_cmd', { projectRoot: projectRoot || null });
}
