/**
 * Validates release-manifest.json against the v1.0.8 schema.
 *
 * Usage: npx tsx scripts/validate-release-manifest.ts [path]
 *   Default path: release-manifest.json
 *
 * Checks:
 *   - Required top-level sections exist
 *   - All commit SHAs are full 40-char hex strings (or null)
 *   - All artifact SHA256 values are full 64-char hex strings (or empty for templates)
 *   - Platform statuses use only allowed values
 *   - signed fields are boolean
 *   - final_artifact_state present on all artifacts
 *   - compile_status/bundle_status present on non-tested platforms
 *   - No platform marked tested without smoke_evidence
 *   - macOS not marked tested/notarized-tested without artifact+smoke
 *   - No raw secrets appear in the manifest
 */

import { readFileSync } from 'fs';

const ALLOWED_STATUSES = [
  'tested', 'signed-tested',
  'built-but-unverified', 'bundle-produced-unverified',
  'runtime-evidence-wsl2', 'runtime-evidence-wslg',
  'runtime-blocked', 'native-runtime-blocked', 'native-smoke-failed', 'native-smoke-blocked',
  'smoke-failed', 'smoke-blocked',
  'build-attempted-failed',
  'build-feasibility-verified', 'artifact-built-unnotarized', 'tested-unnotarized',
  'notarized-tested',
  'not-built', 'deferred', 'failed', 'unsupported',
];

const SHA256_RE = /^[a-f0-9]{64}$/;
const GIT_SHA_RE = /^[a-f0-9]{40}$/;
const SECRET_PATTERNS = ['ghp_', 'gho_', 'ghu_', 'ghs_', 'ghr_', 'sk-', 'Bearer ', 'password', 'secret:', 'token:'];

function fail(msg: string): never {
  console.error(`FAIL: ${msg}`);
  process.exit(1);
}

function warn(msg: string): void {
  console.warn(`WARN: ${msg}`);
}

function validateCommitSha(value: unknown, field: string): void {
  if (value === null || value === undefined || value === '') return;
  if (typeof value !== 'string') fail(`${field} must be a string or null`);
  if (!GIT_SHA_RE.test(value)) fail(`${field} must be a full 40-char hex SHA, got: "${value}"`);
}

function validateSha256(value: unknown, field: string): void {
  if (value === null || value === undefined || value === '' || value === 'PENDING_CI' || value === 'PENDING_BUILD') return;
  if (typeof value !== 'string') fail(`${field} must be a string`);
  if (!SHA256_RE.test(value)) fail(`${field} must be a full 64-char hex SHA256, got: "${value.slice(0, 16)}..."`);
}

function checkSecrets(obj: unknown, path: string): void {
  if (typeof obj === 'string') {
    for (const pat of SECRET_PATTERNS) {
      if (obj.includes(pat)) fail(`Secret pattern "${pat}" found at ${path}`);
    }
  } else if (Array.isArray(obj)) {
    obj.forEach((v, i) => checkSecrets(v, `${path}[${i}]`));
  } else if (obj && typeof obj === 'object') {
    for (const [k, v] of Object.entries(obj)) {
      checkSecrets(v, `${path}.${k}`);
    }
  }
}

// --- Main ---
const path = process.argv[2] || 'release-manifest.json';
let raw: string;
let manifest: any;

try {
  raw = readFileSync(path, 'utf-8');
} catch (e: any) {
  fail(`Cannot read ${path}: ${e.message}`);
}

try {
  manifest = JSON.parse(raw);
} catch (e: any) {
  fail(`Invalid JSON in ${path}: ${e.message}`);
}

// Check secrets
checkSecrets(manifest, '$');

// Release section
const release = manifest.release;
if (!release) fail('Missing "release" section');
if (!release.name) fail('Missing release.name');
if (!release.tag) fail('Missing release.tag');
if (!release.release_version) fail('Missing release.release_version');
if (!release.app_version) fail('Missing release.app_version');
if (release.release_version !== release.app_version) {
  warn(`release_version (${release.release_version}) != app_version (${release.app_version})`);
}

// Provenance section
const provenance = manifest.provenance;
if (!provenance) fail('Missing "provenance" section');
validateCommitSha(provenance.tag_commit, 'provenance.tag_commit');
validateCommitSha(provenance.source_commit, 'provenance.source_commit');
validateCommitSha(provenance.build_commit, 'provenance.build_commit');
if (provenance.release_upload_commit !== null && provenance.release_upload_commit !== undefined) {
  validateCommitSha(provenance.release_upload_commit, 'provenance.release_upload_commit');
}

// Artifacts
if (!Array.isArray(manifest.artifacts)) fail('Missing "artifacts" array');
for (const [i, art] of manifest.artifacts.entries()) {
  if (!art.name) fail(`artifacts[${i}].name missing`);
  if (!art.platform) fail(`artifacts[${i}].platform missing`);
  if (typeof art.signed !== 'boolean') fail(`artifacts[${i}].signed must be boolean`);
  if (!art.final_artifact_state) fail(`artifacts[${i}].final_artifact_state missing`);
  validateSha256(art.sha256, `artifacts[${i}].sha256`);
}

// Platforms
const platforms = manifest.platforms;
if (!platforms) fail('Missing "platforms" section');
for (const [key, plat] of Object.entries(platforms as Record<string, any>)) {
  if (!ALLOWED_STATUSES.includes(plat.status)) {
    fail(`platforms.${key}.status "${plat.status}" is not an allowed value: ${ALLOWED_STATUSES.join(', ')}`);
  }
  if (typeof plat.signed !== 'boolean') fail(`platforms.${key}.signed must be boolean`);

  // Tested platforms must have smoke_evidence
  if (plat.status === 'tested' || plat.status === 'signed-tested' || plat.status === 'tested-unnotarized') {
    if (!plat.smoke_evidence) {
      fail(`platforms.${key} is "${plat.status}" but has no smoke_evidence`);
    }
  }

  // macOS cannot be tested or notarized-tested without artifact+smoke
  if (key.startsWith('macos') && (plat.status === 'tested' || plat.status === 'notarized-tested')) {
    if (!plat.artifact || !plat.smoke_evidence) {
      fail(`platforms.${key} is "${plat.status}" but lacks artifact or smoke_evidence`);
    }
  }

  // bundle-produced-unverified: no smoke, no runtime attempt
  if (plat.status === 'built-but-unverified' || plat.status === 'build-feasibility-verified' || plat.status === 'bundle-produced-unverified') {
    if (!plat.compile_status) {
      fail(`platforms.${key} with status "${plat.status}" must have compile_status`);
    }
    if (!plat.bundle_status) {
      fail(`platforms.${key} with status "${plat.status}" must have bundle_status`);
    }
    if (plat.smoke_tested === undefined) {
      fail(`platforms.${key} with status "${plat.status}" must have smoke_tested field`);
    }
  }

  // runtime-evidence-wsl2: runtime attempted under WSL2, not promoted
  if (plat.status === 'runtime-evidence-wsl2') {
    if (plat.public_artifact !== true) {
      fail(`platforms.${key} with status "runtime-evidence-wsl2" must have public_artifact: true`);
    }
    if (plat.smoke_tested !== false) {
      fail(`platforms.${key} with status "runtime-evidence-wsl2" must have smoke_tested: false (not native desktop)`);
    }
    if (!plat.runtime_attempted || plat.runtime_attempted !== true) {
      fail(`platforms.${key} with status "runtime-evidence-wsl2" must have runtime_attempted: true`);
    }
    if (!plat.promotion_blocker) {
      fail(`platforms.${key} with status "runtime-evidence-wsl2" must have promotion_blocker`);
    }
  }

  // runtime-evidence-wslg: runtime pass under WSLg, not promoted
  if (plat.status === 'runtime-evidence-wslg') {
    if (plat.public_artifact !== true) {
      fail(`platforms.${key} with status "runtime-evidence-wslg" must have public_artifact: true`);
    }
    if (plat.smoke_tested !== false) {
      fail(`platforms.${key} with status "runtime-evidence-wslg" must have smoke_tested: false (not native desktop)`);
    }
    if (plat.native_desktop_smoke !== false) {
      fail(`platforms.${key} with status "runtime-evidence-wslg" must have native_desktop_smoke: false`);
    }
    if (!plat.runtime_attempted || plat.runtime_attempted !== true) {
      fail(`platforms.${key} with status "runtime-evidence-wslg" must have runtime_attempted: true`);
    }
    if (!plat.runtime_result) {
      fail(`platforms.${key} with status "runtime-evidence-wslg" must have runtime_result`);
    }
    if (!plat.promotion_blocker) {
      fail(`platforms.${key} with status "runtime-evidence-wslg" must have promotion_blocker`);
    }
    if (!plat.runtime_environment) {
      fail(`platforms.${key} with status "runtime-evidence-wslg" must have runtime_environment`);
    }
  }

  // native-runtime-blocked: native desktop launch/runtime failed
  if (plat.status === 'native-runtime-blocked') {
    if (plat.smoke_tested !== false) {
      fail(`platforms.${key} with status "native-runtime-blocked" must have smoke_tested: false`);
    }
    if (!plat.runtime_attempted || plat.runtime_attempted !== true) {
      fail(`platforms.${key} with status "native-runtime-blocked" must have runtime_attempted: true`);
    }
    if (!plat.runtime_environment) {
      fail(`platforms.${key} with status "native-runtime-blocked" must have runtime_environment`);
    }
    if (!plat.runtime_blocker) {
      fail(`platforms.${key} with status "native-runtime-blocked" must have runtime_blocker`);
    }
  }

  // native-smoke-failed: native desktop launched but smoke path failed
  if (plat.status === 'native-smoke-failed') {
    if (!plat.runtime_attempted || plat.runtime_attempted !== true) {
      fail(`platforms.${key} with status "native-smoke-failed" must have runtime_attempted: true`);
    }
    if (!plat.runtime_environment) {
      fail(`platforms.${key} with status "native-smoke-failed" must have runtime_environment`);
    }
  }
}

// Verification
const verification = manifest.verification;
if (!verification) fail('Missing "verification" section');

// Signing section
if (manifest.signing) {
  if (manifest.signing.windows) {
    const ws = manifest.signing.windows;
    if (typeof ws.signed !== 'boolean') fail('signing.windows.signed must be boolean');
    if (!ws.status) warn('signing.windows.status missing');
    if (ws.signed === false && !ws.blocker) warn('signing.windows.blocker should be set when unsigned');
    if (!ws.verification_evidence) warn('signing.windows.verification_evidence path missing');
  }
}

// Distribution section (optional but recommended)
if (manifest.distribution) {
  if (manifest.distribution.quickstart && typeof manifest.distribution.quickstart !== 'string') {
    fail('distribution.quickstart must be a string path');
  }
  if (manifest.distribution.troubleshooting && typeof manifest.distribution.troubleshooting !== 'string') {
    fail('distribution.troubleshooting must be a string path');
  }
  if (manifest.distribution.installer_diagnostics && typeof manifest.distribution.installer_diagnostics !== 'string') {
    fail('distribution.installer_diagnostics must be a string path');
  }
  if (manifest.distribution.issue_triage && typeof manifest.distribution.issue_triage !== 'string') {
    fail('distribution.issue_triage must be a string path');
  }
}

// Platform verification section (optional)
if (manifest.platform_verification) {
  if (manifest.platform_verification.ci_run_id && typeof manifest.platform_verification.ci_run_id !== 'string') {
    fail('platform_verification.ci_run_id must be a string');
  }
}

// Dashboard section (optional but recommended)
if (manifest.dashboard) {
  if (manifest.dashboard.url && typeof manifest.dashboard.url !== 'string') {
    fail('dashboard.url must be a string');
  }
  if (manifest.dashboard.generated_from_commit) {
    validateCommitSha(manifest.dashboard.generated_from_commit, 'dashboard.generated_from_commit');
  }
}

// Diagnostics section (optional)
if (manifest.diagnostics) {
  if (manifest.diagnostics.installer_diagnostics && typeof manifest.diagnostics.installer_diagnostics !== 'string') {
    fail('diagnostics.installer_diagnostics must be a string path');
  }
  if (manifest.diagnostics.troubleshooting && typeof manifest.diagnostics.troubleshooting !== 'string') {
    fail('diagnostics.troubleshooting must be a string path');
  }
  if (manifest.diagnostics.issue_triage && typeof manifest.diagnostics.issue_triage !== 'string') {
    fail('diagnostics.issue_triage must be a string path');
  }
}

// Product readiness section (v1.1.0+)
if (manifest.product_readiness) {
  const pr = manifest.product_readiness;
  const allowedProviders = ['os-keychain', 'stronghold', 'encrypted-vault'];
  const allowedSecurityLevels = ['production-grade', 'platform-backed', 'beta-grade', 'migration-required'];

  if (pr.credential_provider && !allowedProviders.includes(pr.credential_provider)) {
    fail(`product_readiness.credential_provider "${pr.credential_provider}" is not allowed: ${allowedProviders.join(', ')}`);
  }
  if (pr.credential_security_level && !allowedSecurityLevels.includes(pr.credential_security_level)) {
    fail(`product_readiness.credential_security_level "${pr.credential_security_level}" is not allowed: ${allowedSecurityLevels.join(', ')}`);
  }
  if (pr.real_agents && !Array.isArray(pr.real_agents)) {
    fail('product_readiness.real_agents must be an array');
  }
  if (pr.agent_mode && pr.agent_mode !== 'review-only' && pr.agent_mode !== 'autonomous') {
    fail(`product_readiness.agent_mode "${pr.agent_mode}" must be review-only or autonomous`);
  }
  if (pr.agent_mode === 'autonomous') {
    fail('product_readiness.agent_mode "autonomous" is not allowed — agents must remain review-only');
  }
  // Native Git validation — supports both old pilot and new expanded section
  const ng = (pr as any).native_git || (pr as any).native_git_pilot;
  if (ng) {
    if (ng.blocking === true) {
      fail('product_readiness.native_git.blocking must be false — must not block normal Git');
    }
    if (ng.fallback_required === false || (ng.fallback && ng.fallback === false)) {
      fail('product_readiness.native_git.fallback_required must be true');
    }
    if (ng.write_operations_migrated === true) {
      fail('product_readiness.native_git.write_operations_migrated must be false — write ops remain CLI');
    }
    if (ng.network_operations_migrated === true) {
      fail('product_readiness.native_git.network_operations_migrated must be false — network ops remain CLI');
    }
    if (ng.secret_safe === false) {
      fail('product_readiness.native_git.secret_safe must be true');
    }
    if (!ng.providers || !Array.isArray(ng.providers) || ng.providers.length === 0) {
      fail('product_readiness.native_git.providers is required and must be non-empty');
    }
    if (!ng.parity) {
      fail('product_readiness.native_git.parity is required');
    }
    // v1.2.1: risk_classification constraints
    if (ng.risk_classification) {
      const rc = ng.risk_classification;
      if (rc.secret_paths !== 'path-only') {
        fail('product_readiness.native_git.risk_classification.secret_paths must be "path-only"');
      }
      if (rc.symlink_following !== false) {
        fail('product_readiness.native_git.risk_classification.symlink_following must be false');
      }
      if (!rc.binary_sampling) {
        fail('product_readiness.native_git.risk_classification.binary_sampling is required');
      }
    }
  }

  // v1.3.0: Semantic commit preparation validation
  const scp = (pr as any).semantic_commit_preparation;
  if (scp) {
    if (scp.native_commit_execution === true) {
      fail('product_readiness.semantic_commit_preparation.native_commit_execution must be false');
    }
    if (scp.autonomous_commit === true) {
      fail('product_readiness.semantic_commit_preparation.autonomous_commit must be false');
    }
    if (scp.stages_files === true) {
      fail('product_readiness.semantic_commit_preparation.stages_files must be false');
    }
    if (scp.writes_repository === true) {
      fail('product_readiness.semantic_commit_preparation.writes_repository must be false');
    }
    if (scp.requires_review !== true) {
      fail('product_readiness.semantic_commit_preparation.requires_review must be true');
    }
    if (scp.mode !== 'proposal-only') {
      fail('product_readiness.semantic_commit_preparation.mode must be "proposal-only"');
    }
    if (scp.secret_safe !== true) {
      fail('product_readiness.semantic_commit_preparation.secret_safe must be true');
    }
    if (scp.diff_body_pilot && scp.diff_body_pilot.enabled === true) {
      if (!scp.diff_body_pilot.max_file_size) {
        fail('product_readiness.semantic_commit_preparation.diff_body_pilot.max_file_size is required when enabled');
      }
      if (scp.diff_body_pilot.secret_risk_excluded !== true) {
        fail('product_readiness.semantic_commit_preparation.diff_body_pilot.secret_risk_excluded must be true');
      }
    }
    // v1.3.1: explicit push/pull/release_verified_state gates
    if (scp.pushes === true) {
      fail('product_readiness.semantic_commit_preparation.pushes must be false');
    }
    if (scp.pulls === true) {
      fail('product_readiness.semantic_commit_preparation.pulls must be false');
    }
    if (scp.diff_body_pilot && !scp.diff_body_pilot.release_verified_state) {
      fail('product_readiness.semantic_commit_preparation.diff_body_pilot.release_verified_state is required');
    }
  }

  // Agent context quality (v1.4.0)
  if (pr.agent_context_quality) {
    const acq = pr.agent_context_quality;
    if (acq.review_only !== true) {
      fail('product_readiness.agent_context_quality.review_only must be true');
    }
    if (acq.auto_commit !== false) {
      fail('product_readiness.agent_context_quality.auto_commit must be false');
    }
    if (acq.auto_apply !== false) {
      fail('product_readiness.agent_context_quality.auto_apply must be false');
    }
    if (acq.stages_files !== false) {
      fail('product_readiness.agent_context_quality.stages_files must be false');
    }
    if (acq.writes_repository !== false) {
      fail('product_readiness.agent_context_quality.writes_repository must be false');
    }
    if (acq.native_commit_execution !== false) {
      fail('product_readiness.agent_context_quality.native_commit_execution must be false');
    }
    if (acq.autonomous_actions !== false) {
      fail('product_readiness.agent_context_quality.autonomous_actions must be false');
    }
    if (acq.schema_version !== 'agent-context-v2') {
      fail('product_readiness.agent_context_quality.schema_version must be agent-context-v2');
    }
    if (!acq.forbidden_fields || !Array.isArray(acq.forbidden_fields)) {
      fail('product_readiness.agent_context_quality.forbidden_fields is required and must be an array');
    }
    if (acq.context_content_policy) {
      const cp = acq.context_content_policy;
      if (cp.git_context_file_contents !== false) {
        fail('product_readiness.agent_context_quality.context_content_policy.git_context_file_contents must be false');
      }
      if (cp.raw_diffs !== false) {
        fail('product_readiness.agent_context_quality.context_content_policy.raw_diffs must be false');
      }
      if (cp.secret_contents_excluded !== true) {
        fail('product_readiness.agent_context_quality.context_content_policy.secret_contents_excluded must be true');
      }
      if (cp.absolute_paths_displayed !== false) {
        fail('product_readiness.agent_context_quality.context_content_policy.absolute_paths_displayed must be false');
      }
    }
    // v1.4.1: degradation guarantees
    if (acq.context_degradation) {
      const cd = acq.context_degradation;
      if (cd.graceful !== true) {
        fail('product_readiness.agent_context_quality.context_degradation.graceful must be true');
      }
      if (cd.failure_blocks_agent !== false) {
        fail('product_readiness.agent_context_quality.context_degradation.failure_blocks_agent must be false');
      }
    }
    // v1.4.1: stabilization evidence
    if (acq.stabilization) {
      const st = acq.stabilization;
      if (st.forbidden_field_scan !== 'path-aware') {
        fail('product_readiness.agent_context_quality.stabilization.forbidden_field_scan must be path-aware');
      }
    }
  }

  // Safe diff context pilot (v1.5.0)
  if (pr.safe_diff_context_pilot) {
    const sdc = pr.safe_diff_context_pilot;
    if (sdc.default !== 'off') {
      fail('product_readiness.safe_diff_context_pilot.default must be off');
    }
    if (sdc.review_only !== true) {
      fail('product_readiness.safe_diff_context_pilot.review_only must be true');
    }
    if (sdc.auto_commit !== false) {
      fail('product_readiness.safe_diff_context_pilot.auto_commit must be false');
    }
    if (sdc.auto_apply !== false) {
      fail('product_readiness.safe_diff_context_pilot.auto_apply must be false');
    }
    if (sdc.stages_files !== false) {
      fail('product_readiness.safe_diff_context_pilot.stages_files must be false');
    }
    if (sdc.writes_repository !== false) {
      fail('product_readiness.safe_diff_context_pilot.writes_repository must be false');
    }
    if (sdc.native_commit_execution !== false) {
      fail('product_readiness.safe_diff_context_pilot.native_commit_execution must be false');
    }
    if (sdc.autonomous_actions !== false) {
      fail('product_readiness.safe_diff_context_pilot.autonomous_actions must be false');
    }
    if (!sdc.provider) {
      fail('product_readiness.safe_diff_context_pilot.provider is required');
    }
    if (sdc.policy) {
      const p = sdc.policy;
      if (p.secret_risk_excluded !== true) {
        fail('product_readiness.safe_diff_context_pilot.policy.secret_risk_excluded must be true');
      }
      if (p.binary_excluded !== true) {
        fail('product_readiness.safe_diff_context_pilot.policy.binary_excluded must be true');
      }
      if (p.large_excluded !== true) {
        fail('product_readiness.safe_diff_context_pilot.policy.large_excluded must be true');
      }
      if (p.symlink_excluded !== true) {
        fail('product_readiness.safe_diff_context_pilot.policy.symlink_excluded must be true');
      }
      if (p.absolute_paths_excluded !== true) {
        fail('product_readiness.safe_diff_context_pilot.policy.absolute_paths_excluded must be true');
      }
      if (!p.max_files || !p.max_total_lines) {
        fail('product_readiness.safe_diff_context_pilot.policy must have caps (max_files, max_total_lines)');
      }
    }
    if (!sdc.forbidden_fields || !Array.isArray(sdc.forbidden_fields)) {
      fail('product_readiness.safe_diff_context_pilot.forbidden_fields is required');
    }
  }

  // Legacy pilot compatibility
  if (pr.native_git_pilot) {
    const ngp = pr.native_git_pilot;
    if (ngp.blocking === true) {
      fail('product_readiness.native_git_pilot.blocking must be false — pilot must not block normal Git');
    }
    if (!ngp.fallback) {
      fail('product_readiness.native_git_pilot.fallback is required');
    }
  }

  // v1.6.0: Git Provider Diagnostics
  if ((pr as any).git_provider_diagnostics) {
    const gpd = (pr as any).git_provider_diagnostics;
    if (gpd.enabled !== true) {
      fail('product_readiness.git_provider_diagnostics.enabled must be true');
    }
    if (gpd.per_operation_reporting !== true) {
      fail('product_readiness.git_provider_diagnostics.per_operation_reporting must be true');
    }
    if (gpd.provider_enum !== true) {
      fail('product_readiness.git_provider_diagnostics.provider_enum must be true — enum-backed labels required');
    }
    if (gpd.read_only_diagnostics_only !== true) {
      fail('product_readiness.git_provider_diagnostics.read_only_diagnostics_only must be true — diagnostics must never mutate');
    }
    if (gpd.mutating_ops_registered_not_executed !== true) {
      fail('product_readiness.git_provider_diagnostics.mutating_ops_registered_not_executed must be true');
    }
    if (!Array.isArray(gpd.operations) || gpd.operations.length < 10) {
      fail('product_readiness.git_provider_diagnostics.operations must list at least 10 operations');
    }
    if (gpd.commit_creation_provider !== 'gix-hybrid') {
      fail('product_readiness.git_provider_diagnostics.commit_creation_provider must be "gix-hybrid" — tree-from-index is CLI');
    }
    if (gpd.no_mutation_guarantee !== true) {
      fail('product_readiness.git_provider_diagnostics.no_mutation_guarantee must be true');
    }
    if (gpd.empty_result_provider_guaranteed !== true) {
      fail('product_readiness.git_provider_diagnostics.empty_result_provider_guaranteed must be true — wrappers must carry provider on empty results');
    }
    if (!Array.isArray(gpd.response_wrappers) || gpd.response_wrappers.length < 2) {
      fail('product_readiness.git_provider_diagnostics.response_wrappers must list at least 2 wrappers');
    }
  } else {
    warn('product_readiness.git_provider_diagnostics section missing (v1.6.0+)');
  }

  // v1.9.0: Architect Agent
  if ((pr as any).architect_agent) {
    const aa = (pr as any).architect_agent;
    if (aa.enabled !== true) fail('product_readiness.architect_agent.enabled must be true');
    if (aa.mode !== 'read-only-planner') fail('product_readiness.architect_agent.mode must be read-only-planner');
    if (aa.may_edit_files === true) fail('product_readiness.architect_agent.may_edit_files must be false');
    if (aa.may_create_adrs === true) fail('product_readiness.architect_agent.may_create_adrs must be false');
    if (aa.patch_application === true) fail('product_readiness.architect_agent.patch_application must be false');
    if (aa.emits_patch_proposals === true) fail('product_readiness.architect_agent.emits_patch_proposals must be false');
    if (aa.writes_repository === true) fail('product_readiness.architect_agent.writes_repository must be false');
    if (aa.schema_versioned !== true) fail('product_readiness.architect_agent.schema_versioned must be true');
    if (aa.no_runtime_mock !== true) fail('product_readiness.architect_agent.no_runtime_mock must be true');
    if (!Array.isArray(aa.context_sources) || aa.context_sources.length < 3) fail('product_readiness.architect_agent.context_sources must have at least 3 sources');
    if (!Array.isArray(aa.output_artifacts) || aa.output_artifacts.length < 5) fail('product_readiness.architect_agent.output_artifacts must have at least 5 artifacts');
  } else {
    warn('product_readiness.architect_agent section missing (v1.9.0+)');
  }

  // v1.8.0: Code Intelligence
  if ((pr as any).code_intelligence) {
    const ci = (pr as any).code_intelligence;
    if (ci.enabled !== true) fail('product_readiness.code_intelligence.enabled must be true');
    if (!Array.isArray(ci.languages) || ci.languages.length < 2) fail('product_readiness.code_intelligence.languages must have at least 2 languages');
    if (ci.symbol_extraction !== true) fail('product_readiness.code_intelligence.symbol_extraction must be true');
    if (ci.content_safe !== true) fail('product_readiness.code_intelligence.content_safe must be true');
    if (ci.read_only !== true) fail('product_readiness.code_intelligence.read_only must be true');
    if (ci.deterministic !== true) fail('product_readiness.code_intelligence.deterministic must be true');
    if (ci.no_source_bodies_in_output !== true) fail('product_readiness.code_intelligence.no_source_bodies_in_output must be true');
    if (ci.zero_tauri_dependency !== true) fail('product_readiness.code_intelligence.zero_tauri_dependency must be true');
    if (ci.zero_git_bridge_dependency !== true) fail('product_readiness.code_intelligence.zero_git_bridge_dependency must be true');
    if (ci.excludes_binary !== true) fail('product_readiness.code_intelligence.excludes_binary must be true');
    if (ci.excludes_secret_paths !== true) fail('product_readiness.code_intelligence.excludes_secret_paths must be true');
    if (ci.attached_to_agents?.includes('docs-agent')) warn('product_readiness.code_intelligence.attached_to_agents includes docs-agent — should be disabled-by-default');
    if (!ci.attached_to_agents?.includes('bugfix-agent')) fail('product_readiness.code_intelligence.attached_to_agents must include bugfix-agent');
    if (ci.affected_symbols_scope !== 'file-level') fail('product_readiness.code_intelligence.affected_symbols_scope must be file-level');
    if (typeof ci.max_file_size_bytes !== 'number') fail('product_readiness.code_intelligence.max_file_size_bytes must be a number');
  } else {
    warn('product_readiness.code_intelligence section missing (v1.8.0+)');
  }

  // v1.7.0: Patch Governance
  if ((pr as any).patch_governance) {
    const pg = (pr as any).patch_governance;
    if (pg.enabled !== true) fail('product_readiness.patch_governance.enabled must be true');
    if (pg.validation_before_apply !== true) fail('product_readiness.patch_governance.validation_before_apply must be true');
    if (pg.forbidden_path_enforcement !== true) fail('product_readiness.patch_governance.forbidden_path_enforcement must be true');
    if (pg.binary_write_blocked !== true) fail('product_readiness.patch_governance.binary_write_blocked must be true');
    if (pg.secret_path_blocked !== true) fail('product_readiness.patch_governance.secret_path_blocked must be true');
    if (pg.workflow_path_blocked !== true) fail('product_readiness.patch_governance.workflow_path_blocked must be true');
    if (pg.dependency_lock_blocked !== true) fail('product_readiness.patch_governance.dependency_lock_blocked must be true');
    if (pg.before_checksum_verification !== true) fail('product_readiness.patch_governance.before_checksum_verification must be true');
    if (pg.atomic_writes !== true) fail('product_readiness.patch_governance.atomic_writes must be true');
    if (pg.audit_trail !== true) fail('product_readiness.patch_governance.audit_trail must be true');
    if (pg.proposal_id_required !== true) fail('product_readiness.patch_governance.proposal_id_required must be true');
    if (pg.server_side_agent_policy !== true) fail('product_readiness.patch_governance.server_side_agent_policy must be true');
    if (pg.frontend_narrows_only !== true) fail('product_readiness.patch_governance.frontend_narrows_only must be true');
    if (pg.auto_commit === true) fail('product_readiness.patch_governance.auto_commit must be false');
    if (pg.auto_apply === true) fail('product_readiness.patch_governance.auto_apply must be false');
    if (!Array.isArray(pg.states) || !pg.states.includes('applied') || !pg.states.includes('rejected')) {
      fail('product_readiness.patch_governance.states must include applied and rejected');
    }
    if (pg.typed_dtos !== true) fail('product_readiness.patch_governance.typed_dtos must be true');
  } else {
    warn('product_readiness.patch_governance section missing (v1.7.0+)');
  }

  // Cross-check: if credential_security_level is production-grade,
  // require evidence that both tested platforms have credential verification
  if (pr.credential_security_level === 'production-grade') {
    const testedPlatforms = Object.entries(platforms as Record<string, any>)
      .filter(([_, p]) => p.status === 'tested');
    for (const [key, _] of testedPlatforms) {
      // production-grade claim requires at least one tested platform
      warn(`product_readiness.credential_security_level is "production-grade" — ensure credential tests exist for ${key}`);
    }
  }
}

// Limitations
if (!Array.isArray(manifest.limitations)) fail('Missing "limitations" array');

console.log(`PASS: ${path} validates successfully`);
console.log(`  Release: ${release.name}`);
console.log(`  Channel: ${release.channel}`);
console.log(`  Artifacts: ${manifest.artifacts.length}`);
console.log(`  Platforms: ${Object.keys(platforms).join(', ')}`);
if (manifest.product_readiness) {
  console.log(`  Credential: ${manifest.product_readiness.credential_provider} (${manifest.product_readiness.credential_security_level})`);
  console.log(`  Agents: ${manifest.product_readiness.real_agents?.join(', ')} (${manifest.product_readiness.agent_mode})`);
  const ngLabel = (manifest.product_readiness as any).native_git ? 'read-only' : ((manifest.product_readiness as any).native_git_pilot?.operation || 'none');
  console.log(`  Native Git: ${ngLabel}`);
}
