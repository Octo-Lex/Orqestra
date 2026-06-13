//! Orqestra Development Lifecycle — TypeScript types (v0.2.0)
//!
//! Mirror of the Rust types for the frontend.

export type LifecycleStageName =
  | 'orient'
  | 'discover'
  | 'define'
  | 'design'
  | 'plan'
  | 'prepare'
  | 'build'
  | 'verify'
  | 'review'
  | 'ship'
  | 'observe'
  | 'learn'
  | 'evolve';

export interface StageInfo {
  name: LifecycleStageName;
  display_name: string;
  index: number;
  is_current: boolean;
  is_implemented: boolean;
  purpose: string;
}

export interface ArtifactRecord {
  artifact_type: string;
  path: string;
  feature_id: string | null;
  created_at: string;
  updated_at: string | null;
}

export type GateStatus = 'pending' | 'requested' | 'approved' | 'rejected';

export interface GateRecord {
  gate: string;
  feature_id: string | null;
  status: GateStatus;
  requested_at: string | null;
  decided_at: string | null;
  decided_by: string | null;
  rejection_reason: string | null;
}

export interface LifecycleState {
  initialized: boolean;
  started: boolean;
  current_stage: LifecycleStageName | null;
  current_stage_name: string | null;
  current_stage_purpose: string | null;
  stages: StageInfo[];
  artifacts: ArtifactRecord[];
  gates: GateRecord[];
  events_count: number;
}

export const STAGE_LABELS: Record<LifecycleStageName, string> = {
  orient: 'Orient',
  discover: 'Discover',
  define: 'Define',
  design: 'Design',
  plan: 'Plan',
  prepare: 'Prepare',
  build: 'Build',
  verify: 'Verify',
  review: 'Review',
  ship: 'Ship',
  observe: 'Observe',
  learn: 'Learn',
  evolve: 'Evolve',
};

export const ALL_STAGES: LifecycleStageName[] = [
  'orient', 'discover', 'define', 'design', 'plan',
  'prepare', 'build', 'verify', 'review', 'ship',
  'observe', 'learn', 'evolve',
];

export const IMPLEMENTED_STAGES: LifecycleStageName[] = [
  'orient', 'discover', 'define', 'design', 'plan',
];
