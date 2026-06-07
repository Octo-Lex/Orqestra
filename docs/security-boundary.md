# Security Boundary (v2.5.1+)

## Overview

v2.5.1 is a **release trust repair wave**. It closes six immediate security defects before any future autonomy or authority expansion.

## Trust Boundary

```
Cloudflare Worker:
  Master secret (ORQESTRA_SYNC_MASTER) lives here ONLY.
  Token generation via POST /token/generate.
  HMAC-SHA256 verification.

Desktop:
  Stores only workspace-scoped tokens.
  No master secret.
  No admin token generation.
  TokenManager::new(None) in desktop mode.
```

## Defects Closed

| # | Defect | Fix |
|---|--------|-----|
| 1 | Non-cryptographic HMAC (djb2) | HMAC-SHA256 via `crypto.subtle.sign` |
| 2 | Hardcoded `default-master-token` | Removed; `TokenManager::new(None)` |
| 3 | Live credentials in `.env` | Rotated; stored in provider secret stores |
| 4 | CSP `null` | Restrictive CSP with no wildcards |
| 5 | `DefaultHasher` 16-char checksum | Real SHA-256 (64-char) |
| 6 | Predictable PAT temp file | UUID temp dir, `create_new`, RAII cleanup |

## Token Format

v2: `ork_v2_{scope}_{workspace_id}_{timestamp_hex}_{hmac_sha256_hex}`

Legacy v1 tokens rejected with `UNSUPPORTED_TOKEN_VERSION`.

## HMAC Verification

- Uses `crypto.subtle.sign('HMAC', key, payload)` with SHA-256
- Constant-time comparison via XOR accumulator (`timingSafeEqualHex`)
- 64-char hex HMAC output

## Patch Checksum

- SHA-256 via `sha2::Sha256` crate
- 64-char hex output
- Legacy 16-char checksums rejected with `LEGACY_CHECKSUM_FORMAT`

## CSP

```json
{
  "default-src": "'self'",
  "script-src": "'self'",
  "style-src": "'self' 'unsafe-inline'",
  "connect-src": "'self' https://api.github.com https://ampify.ampify-cloud.workers.dev wss://ampify.ampify-cloud.workers.dev http://localhost:8787 http://localhost:1420",
  "object-src": "'none'"
}
```

No wildcards. No `unsafe-eval`.

## PAT Handling (Interim)

- Unique temp directory per operation (`uuid::Uuid::new_v4()`)
- `create_new(true)` — never overwrites existing file
- RAII cleanup via `impl Drop`
- Owner-only permissions on Unix
- **Interim** — v2.5.2+ should use credential helper or in-memory flow

## Secret Scanning

- `gitleaks` CI step in `.github/workflows/secret-scan.yml`
- Current-tree scanning blocks PRs with detected secrets
- `.env` files gitignored and never committed

## Manifest Gates

`security_stabilization` section in `release-manifest.json` prevents regression:
- `sync_relay_hmac: "hmac-sha256"`
- `desktop_master_secret_compiled_in: false`
- `tauri_csp_enabled: true`
- `patch_checksum: "sha256"`
- `askpass_predictable_path: false`
- `constant_time_hmac_compare: true`
- `token_version_v2: true`

## Test Coverage

- 17 Rust security boundary tests
- 12 Worker HMAC-SHA256 auth tests
- Total: 568 Rust + 24 Worker = 592 tests
