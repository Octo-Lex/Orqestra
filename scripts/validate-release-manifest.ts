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
  'built-but-unverified', 'bundle-produced-unverified', 'runtime-evidence-wsl2',
  'runtime-blocked', 'smoke-failed', 'smoke-blocked', 'PENDING_SMOKE',
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
  if (value === null || value === undefined || value === '' || value === 'PENDING_CI') return;
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

  // runtime-blocked: runtime attempted but failed
  if (plat.status === 'runtime-blocked') {
    if (!plat.runtime_attempted || plat.runtime_attempted !== true) {
      fail(`platforms.${key} with status "runtime-blocked" must have runtime_attempted: true`);
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

// Limitations
if (!Array.isArray(manifest.limitations)) fail('Missing "limitations" array');

console.log(`PASS: ${path} validates successfully`);
console.log(`  Release: ${release.name}`);
console.log(`  Channel: ${release.channel}`);
console.log(`  Artifacts: ${manifest.artifacts.length}`);
console.log(`  Platforms: ${Object.keys(platforms).join(', ')}`);
