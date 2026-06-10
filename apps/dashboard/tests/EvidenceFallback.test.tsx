/**
 * Evidence fallback and hardening tests — v2.10.1
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/react';
import { ReleaseHistoryPanel } from '../src/components/ReleaseHistoryPanel';
import { TestCountTrendPanel } from '../src/components/TestCountTrendPanel';
import { SecurityBoundaryPanel } from '../src/components/SecurityBoundaryPanel';
import { AutonomyPolicyPanel } from '../src/components/AutonomyPolicyPanel';
import { RuntimeEvidencePanel } from '../src/components/RuntimeEvidencePanel';

afterEach(() => cleanup());

// --- Malformed evidence fallback ---

describe('Evidence Fallback', () => {
  it('ReleaseHistoryPanel handles malformed data', () => {
    render(<ReleaseHistoryPanel releaseHistory={{ releases: null } as any} />);
    // Should not crash, either renders fallback or empty
    expect(document.body.textContent).toBeDefined();
  });

  it('TestCountTrendPanel handles malformed data', () => {
    render(<TestCountTrendPanel testCounts={{ history: null } as any} />);
    expect(screen.getByText(/unavailable/i)).toBeDefined();
  });

  it('SecurityBoundaryPanel handles malformed data', () => {
    render(<SecurityBoundaryPanel securityBoundaries={{ boundaries: null } as any} />);
    expect(screen.getByText(/unavailable/i)).toBeDefined();
  });

  it('AutonomyPolicyPanel handles malformed data', () => {
    render(<AutonomyPolicyPanel autonomyPolicy={{ status: null } as any} />);
    expect(screen.getByText(/unavailable/i)).toBeDefined();
  });

  it('RuntimeEvidencePanel handles malformed data', () => {
    render(<RuntimeEvidencePanel runtimeEvidence={{} as any} />);
    expect(screen.getByText(/unavailable/i)).toBeDefined();
  });

  it('RuntimeEvidencePanel displays structural badge and not-external-beta', () => {
    const evidence = {
      evidence_type: 'structural-runtime-decision-matrix',
      external_beta_user_data: false,
      path_matrix_evaluated: 50,
      paths_allowed: 13,
      paths_rejected: 37,
      rejection_rate: '74%',
      safety_invariants_total: 9,
      safety_invariants_passing: 9,
    };
    render(<RuntimeEvidencePanel runtimeEvidence={evidence} />);
    expect(screen.getByText('structural-runtime-decision-matrix')).toBeDefined();
    expect(screen.getByText('Not external beta data')).toBeDefined();
  });

  it('AutonomyPanel displays max cap 10 and auto_commit false', () => {
    const policy = {
      status: 'docs-only pilot',
      allowed_paths: ['docs/**'],
      confidence_threshold_docs: 0.80,
      confidence_threshold_readme: 0.90,
      max_session_cap: 10,
      default_cap: 5,
      auto_commit: false,
    };
    render(<AutonomyPolicyPanel autonomyPolicy={policy} />);
    expect(screen.getByText('10')).toBeDefined();
    expect(screen.getByText('Always False')).toBeDefined();
  });

  // --- Security regression ---

  it('no evidence component contains former token literal in source', async () => {
    const fs = await import('fs');
    const path = await import('path');
    const components = [
      'ReleaseHistoryPanel.tsx',
      'TestCountTrendPanel.tsx',
      'SecurityBoundaryPanel.tsx',
      'AutonomyPolicyPanel.tsx',
      'RuntimeEvidencePanel.tsx',
      'DataFreshnessPanel.tsx',
    ];
    for (const comp of components) {
      const source = fs.readFileSync(
        path.resolve(__dirname, `../src/components/${comp}`),
        'utf-8'
      );
      expect(source, `${comp} should not contain the former token literal`).not.toContain("=== 'master-secret'");
    }
  });

  // --- Accessibility ---

  it('view switcher buttons have accessible attributes', async () => {
    const fs = await import('fs');
    const path = await import('path');
    const source = fs.readFileSync(
      path.resolve(__dirname, '../src/App.tsx'),
      'utf-8'
    );
    expect(source).toContain('role="tablist"');
    expect(source).toContain('role="tab"');
    expect(source).toContain('aria-selected');
    expect(source).toContain('role="tabpanel"');
  });

  // --- Evidence source label ---

  it('DataFreshnessPanel shows Evidence source label', async () => {
    const fs = await import('fs');
    const path = await import('path');
    const source = fs.readFileSync(
      path.resolve(__dirname, '../src/components/DataFreshnessPanel.tsx'),
      'utf-8'
    );
    expect(source).toContain('Evidence source');
    expect(source).not.toContain('v${evidenceGeneratedFrom.source}');
  });
});
