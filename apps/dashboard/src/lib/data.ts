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
  coherence?: CoherenceMetadata;
  release?: {
    version: string;
    source_commit: string;
    generated_at: string;
    generated_by: string;
  };
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
  evidence?: EvidenceSection;
  sprints: Sprint[];
  tasks: Task[];
};

export type EvidenceSection = {
  schema_version: number;
  generated_from: {
    source: string;
    commit: string;
    generated_at: string;
  };
  release_history: ReleaseHistoryEvidence;
  test_counts: TestCountEvidence;
  security_boundaries: SecurityBoundariesEvidence;
  autonomy_policy: AutonomyPolicyEvidence;
  runtime_evidence: RuntimeEvidence;
  external_beta_evidence?: ExternalBetaEvidence;
};

export type ExternalBetaEvidence = {
  schema_version: number;
  status: string;
  external_beta_user_data: boolean;
  intake_mechanism?: string;
  automatic_upload?: boolean;
  consent_required?: boolean;
  redaction_required?: boolean;
};

export type ReleaseHistoryEvidence = {
  schema_version: number;
  releases: Record<string, ReleaseEntry>;
};

export type ReleaseEntry = {
  date: string;
  type: string;
  label: string;
};

export type TestCountEvidence = {
  schema_version: number;
  history: TestCountEntry[];
};

export type TestCountEntry = {
  version: string;
  rust: number;
  worker: number;
  dashboard: number;
  total: number;
};

export type SecurityBoundariesEvidence = {
  schema_version: number;
  provenance?: string;
  boundaries: Record<string, SecurityBoundaryEntry>;
};

export type SecurityBoundaryEntry = {
  algorithm?: string;
  status?: string;
  location?: string;
  version?: string;
  description?: string;
  accepted_tokens?: string[];
  scopes?: string[];
  rejected?: string[];
};

export type AutonomyPolicyEvidence = {
  schema_version: number;
  status: string;
  allowed_paths: string[];
  excluded_paths?: string[];
  confidence_threshold_docs: number;
  confidence_threshold_readme: number;
  max_session_cap: number;
  default_cap: number;
  auto_commit: boolean;
  audit_schema_version?: number;
  rejection_reasons?: number;
  disallowed_operations?: string[];
};

export type RuntimeEvidence = {
  schema_version: number;
  evidence_type: string;
  external_beta_user_data: boolean;
  path_matrix_evaluated: number;
  paths_allowed: number;
  paths_rejected: number;
  rejection_rate: string;
  safety_invariants_total?: number;
  safety_invariants_passing?: number;
  disclaimer?: string;
};

export type CoherenceMetadata = {
  roadmap_state_hash: string;
  relay_snapshot_hash?: string;
  export_state: 'local-only' | 'relay-metadata-present' | 'unknown';
  task_count: number;
  index_version?: number;
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
