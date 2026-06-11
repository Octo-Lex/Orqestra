/**
 * Evidence panel tests — v2.10.0
 *
 * Tests evidence rendering, missing-evidence fallback, and security regression.
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/react';
import { ReleaseHistoryPanel } from '../src/components/ReleaseHistoryPanel';
import { TestCountTrendPanel } from '../src/components/TestCountTrendPanel';
import { SecurityBoundaryPanel } from '../src/components/SecurityBoundaryPanel';
import { AutonomyPolicyPanel } from '../src/components/AutonomyPolicyPanel';
import { RuntimeEvidencePanel } from '../src/components/RuntimeEvidencePanel';
import { DataFreshnessPanel } from '../src/components/DataFreshnessPanel';

afterEach(() => cleanup());

const mockReleaseHistory = {
  releases: {
    '2.9.1': { date: '2026-06-10', type: 'security-patch', label: 'Dashboard Token Boundary' },
    '2.9.0': { date: '2026-06-09', type: 'feature', label: 'Evidence Dashboard' },
  },
};

const mockTestCounts = {
  history: [
    { version: '2.9.1', rust: 442, worker: 24, dashboard: 12, total: 478 },
    { version: '2.9.0', rust: 442, worker: 24, dashboard: 0, total: 466 },
  ],
};

const mockSecurityBoundaries = {
  provenance: 'Test provenance',
  boundaries: {
    relay_auth: { algorithm: 'HMAC-SHA256', description: 'Test auth' },
    content_security_policy: { status: 'restrictive', description: 'Test CSP' },
    patch_checksum: { algorithm: 'SHA-256', description: 'Test checksums' },
    credential_storage: { location: 'OS keychain', description: 'Test storage' },
    token_format: { version: 'v2', description: 'Test tokens' },
    master_secret: { location: 'Worker only', description: 'Test secret' },
    dashboard_authority: { status: 'read-only evidence surface', description: 'Test authority' },
  },
};

const mockAutonomyPolicy = {
  status: 'docs-only pilot',
  allowed_paths: ['docs/**', 'README.md'],
  excluded_paths: ['CHANGELOG.md', 'roadmap/**'],
  confidence_threshold_docs: 0.80,
  confidence_threshold_readme: 0.90,
  max_session_cap: 10,
  default_cap: 5,
  auto_commit: false,
  audit_schema_version: 1,
  rejection_reasons: 17,
  disallowed_operations: ['source-code auto-apply'],
};

const mockRuntimeEvidence = {
  evidence_type: 'structural-runtime-decision-matrix',
  external_beta_user_data: false,
  path_matrix_evaluated: 50,
  paths_allowed: 13,
  paths_rejected: 37,
  rejection_rate: '74%',
  safety_invariants_total: 9,
  safety_invariants_passing: 9,
  disclaimer: 'Test disclaimer',
};

describe('Evidence Panels', () => {
  // --- Rendering ---

  it('ReleaseHistoryPanel renders releases', () => {
    render(<ReleaseHistoryPanel releaseHistory={mockReleaseHistory} />);
    expect(screen.getByText(/v2\.9\.1/)).toBeDefined();
    expect(screen.getByText(/v2\.9\.0/)).toBeDefined();
    expect(screen.getByText('security-patch')).toBeDefined();
    expect(screen.getByText('feature')).toBeDefined();
  });

  it('TestCountTrendPanel renders history', () => {
    render(<TestCountTrendPanel testCounts={mockTestCounts} />);
    expect(screen.getByText(/478 total/)).toBeDefined();
    expect(screen.getByText(/466 total/)).toBeDefined();
  });

  it('SecurityBoundaryPanel renders boundaries', () => {
    render(<SecurityBoundaryPanel securityBoundaries={mockSecurityBoundaries} />);
    expect(screen.getByText('HMAC-SHA256')).toBeDefined();
    expect(screen.getByText('SHA-256')).toBeDefined();
    expect(screen.getByText('Test provenance')).toBeDefined();
  });

  it('AutonomyPolicyPanel renders policy', () => {
    render(<AutonomyPolicyPanel autonomyPolicy={mockAutonomyPolicy} />);
    expect(screen.getByText('docs-only pilot')).toBeDefined();
    expect(screen.getByText('Always False')).toBeDefined();
  });

  it('RuntimeEvidencePanel renders evidence', () => {
    render(<RuntimeEvidencePanel runtimeEvidence={mockRuntimeEvidence} />);
    expect(screen.getByText('50')).toBeDefined();
    expect(screen.getByText('13')).toBeDefined();
    expect(screen.getByText('37')).toBeDefined();
    expect(screen.getByText('74%')).toBeDefined();
    expect(screen.getByText('Test disclaimer')).toBeDefined();
  });

  it('DataFreshnessPanel renders source info', () => {
    render(
      <DataFreshnessPanel
        generatedAt="2026-06-10T18:00:00Z"
        source={{ repo: 'orqestra', branch: 'master', commit: 'abc123def456' }}
      />
    );
    expect(screen.getByText('Static Export')).toBeDefined();
    expect(screen.getByText(/abc123def456/)).toBeDefined();
  });

  // --- Missing data fallback ---

  it('ReleaseHistoryPanel shows fallback when data missing', () => {
    render(<ReleaseHistoryPanel releaseHistory={{}} />);
    expect(screen.getByText(/unavailable/i)).toBeDefined();
  });

  it('TestCountTrendPanel shows fallback when data missing', () => {
    render(<TestCountTrendPanel testCounts={null} />);
    expect(screen.getByText(/unavailable/i)).toBeDefined();
  });

  it('RuntimeEvidencePanel shows structural badge', () => {
    render(<RuntimeEvidencePanel runtimeEvidence={mockRuntimeEvidence} />);
    expect(screen.getByText('structural-runtime-decision-matrix')).toBeDefined();
    expect(screen.getByText('Not external beta data')).toBeDefined();
  });

  // --- Security regression ---

  it('no evidence component contains admin/write/master-secret in source', async () => {
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
      expect(source, `${comp} should not contain master-secret`).not.toContain("=== 'master-secret'");
      expect(source, `${comp} should not contain admin scope`).not.toContain("'admin'");
    }
  });

  // --- v2.11.0: evidence entries ---

  it('release history includes v2.11.0 entry', () => {
    render(<ReleaseHistoryPanel releaseHistory={{
      ...mockReleaseHistory,
      releases: {
        ...mockReleaseHistory.releases,
        '2.11.0': { date: '2026-06-11', type: 'productization', label: 'Self-Serve Beta Readiness' },
        '2.10.1': { date: '2026-06-11', type: 'hardening', label: 'Evidence Surface Hardening' },
        '2.10.0': { date: '2026-06-10', type: 'feature', label: 'Public Evidence Surface' },
      }
    }} />);
    expect(screen.getByText('Self-Serve Beta Readiness')).toBeDefined();
  });

  it('release history shows productization type', () => {
    render(<ReleaseHistoryPanel releaseHistory={{
      ...mockReleaseHistory,
      releases: {
        '2.11.0': { date: '2026-06-11', type: 'productization', label: 'Self-Serve Beta Readiness' },
      }
    }} />);
    expect(screen.getByText('productization')).toBeDefined();
  });

  // --- v2.11.0: no new authority language ---

  it('no evidence component introduces live API or write language', async () => {
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
      expect(source, `${comp} should not contain live API fetch`).not.toContain('fetch(');
      expect(source, `${comp} should not contain write scope`).not.toContain("'write'");
    }
  });
});
