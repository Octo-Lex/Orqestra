/**
 * ConfidenceGate — prevents low-confidence agent commits.
 *
 * Spec: §4.5 — thresholds are per-workspace with global defaults.
 * Breaking changes NEVER auto-commit regardless of confidence.
 */

export interface ConfidenceGateConfig {
  auto_commit: number;  // ≥ this → commit immediately, notify async
  propose: number;      // ≥ this → stage, show diff, require human approval
  flag: number;         // ≥ this → log concern, assign to human
  breaking_change_override: 'always_propose' | 'always_flag' | 'respect_thresholds';
}

export const DEFAULT_GATE: ConfidenceGateConfig = {
  auto_commit: 0.90,
  propose: 0.70,
  flag: 0.50,
  breaking_change_override: 'always_propose',
};

export type GateAction =
  | { type: 'auto_commit'; notify: 'async' }
  | { type: 'propose'; ui: 'diff_review_modal'; reason?: string }
  | { type: 'flag'; assignee: 'human_fallback'; reason?: string }
  | { type: 'abort'; alert: 'immediate' };

export class ConfidenceGate {
  private config: ConfidenceGateConfig;

  constructor(config: Partial<ConfidenceGateConfig> = {}) {
    this.config = { ...DEFAULT_GATE, ...config };
  }

  resolve(confidence: number, hasBreakingChange: boolean): GateAction {
    // Breaking change override
    if (hasBreakingChange && this.config.breaking_change_override === 'always_propose') {
      return { type: 'propose', ui: 'diff_review_modal', reason: 'breaking_change' };
    }
    if (hasBreakingChange && this.config.breaking_change_override === 'always_flag') {
      return { type: 'flag', assignee: 'human_fallback', reason: 'breaking_change' };
    }

    // Normal threshold resolution
    if (confidence >= this.config.auto_commit) {
      return { type: 'auto_commit', notify: 'async' };
    }
    if (confidence >= this.config.propose) {
      return { type: 'propose', ui: 'diff_review_modal' };
    }
    if (confidence >= this.config.flag) {
      return { type: 'flag', assignee: 'human_fallback' };
    }
    return { type: 'abort', alert: 'immediate' };
  }
}
