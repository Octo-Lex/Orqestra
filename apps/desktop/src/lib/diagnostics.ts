import { invoke } from '@tauri-apps/api/core';

export interface DiagnosticBundleResult {
  path: string;
  created_at: string;
  files: DiagnosticBundleFile[];
  redaction_summary: RedactionSummary;
}

export interface DiagnosticBundleFile {
  name: string;
  description: string;
  bytes: number;
}

export interface RedactionSummary {
  rules_applied: string[];
  redacted_value_count: number;
  contains_raw_secrets: boolean;
}

export interface RecoveryAdvice {
  code: string;
  title: string;
  description: string;
  action_label: string;
  action_kind: string;
  action_payload?: string;
}

export async function exportDiagnostics(projectRoot?: string): Promise<DiagnosticBundleResult> {
  return invoke<DiagnosticBundleResult>('export_diagnostics_cmd', { projectRoot: projectRoot || null });
}

export async function getRecoveryAdvice(code: string): Promise<RecoveryAdvice> {
  return invoke<RecoveryAdvice>('get_recovery_advice_cmd', { code });
}
