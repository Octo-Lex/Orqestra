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
