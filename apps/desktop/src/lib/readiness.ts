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
