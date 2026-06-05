# Operational Risk Classification (v2.4.0+)

## Overview

Path-based operational risk classifier for high-leverage files that affect build, release, supply chain, CI, and execution environment. Not a scanner: no file content parsing, no registry lookups.

## Architecture

```
crates/git-bridge/src/operational_risk.rs
```

Classification uses path pattern matching only. Deterministic: same path always produces same risks.

## Categories (13)

| Category | Severity | Human Review | Blocks Auto | Example |
|---|---|---|---|---|
| CredentialOrSecretConfig | Critical | ✅ | **reject outright** | .env, *.pem, *.key |
| ReleaseManifest | Critical | ✅ | ✅ | release-manifest.json |
| CiWorkflow | High | ✅ | ✅ | .github/workflows/* |
| DependencyLockfile | High | ✅ | ✅ | Cargo.lock, pnpm-lock.yaml |
| DependencyManifest | Medium | ✅ | ❌ | Cargo.toml, package.json |
| CloudflareConfig | Medium | ✅ | ❌ | wrangler.toml |
| TauriConfig | Medium | ✅ | ❌ | tauri.conf.json |
| RepoPolicyConfig | Medium | ✅ | ❌ | CODEOWNERS, dependabot.yml |
| ToolchainConfig | Medium | ✅ | ❌ | rust-toolchain.toml, .cargo/config.toml |
| ContainerConfig | Medium | ✅ | ❌ | Dockerfile, docker-compose.yml |
| BuildConfig | Low | ❌ | ❌ | tsconfig.json, vite.config.* |
| PackageManagerConfig | Low | ❌ | ❌ | .npmrc, .yarnrc |
| UnknownSensitiveConfig | Low/Medium | if sensitive dir | ❌ | Other config files |

## Enforcement Semantics

- **CredentialOrSecretConfig**: reject write outright, no human override
- **Critical (non-credential)**: blocks future auto-apply; human may apply through explicit governance confirmation
- **High**: blocks future auto-apply; requires human review
- **Medium**: requires human review flag; no auto-apply block
- **Low/Info**: informational only

**`blocks_auto_apply` means future auto-apply is forbidden, not that human apply is impossible.**

## Multi-Risk Per File

`classify_path()` returns `Vec<OperationalRisk>`. A single path may match multiple categories. Highest severity determines enforcement.

## UnknownSensitiveConfig Escalation

Unrecognized config files (`.yml`, `.toml`, `.json`, etc.) under sensitive directories are escalated:

- `.github/`, `.cloudflare/`, `.vscode/`, `scripts/`, `deploy/`, `infra/`
- Escalated to Medium severity with `requires_human_review: true`

## Reason Codes (13)

Stable governance contract. Test-covered.

| Code | Meaning |
|---|---|
| `RISK_DEPENDENCY_VERSION_CHANGE` | Dependency version change |
| `RISK_LOCKFILE_MODIFIED` | Lockfile modified |
| `RISK_CI_WORKFLOW_MODIFIED` | CI workflow modified |
| `RISK_CLOUDFLARE_CONFIG_MODIFIED` | Cloudflare config modified |
| `RISK_TAURI_CONFIG_MODIFIED` | Tauri config modified |
| `RISK_RELEASE_MANIFEST_MODIFIED` | Release manifest modified |
| `RISK_CREDENTIAL_OR_SECRET_PROXIMITY` | Credential/secret file touched |
| `RISK_BUILD_CONFIG_MODIFIED` | Build config modified |
| `RISK_REPO_POLICY_MODIFIED` | Repo policy modified |
| `RISK_TOOLCHAIN_MODIFIED` | Toolchain config modified |
| `RISK_PACKAGE_MANAGER_CONFIG_MODIFIED` | Package manager config modified |
| `RISK_CONTAINER_CONFIG_MODIFIED` | Container config modified |
| `RISK_UNKNOWN_SENSITIVE_CONFIG` | Unknown sensitive config |

## Integration

### Patch Governance

- Credential paths → rejected outright
- `blocks_auto_apply: true` → flagged for explicit human confirmation
- Audit trail includes reason code

### Agent Context

- `AgentContextV2.operational_risks` — metadata only, no file contents
- `ArchitectRiskSummary` — bounded summary for planning

### Diagnostics

- `operational-risk.json` (17th bundle file)
- Paths hashed (SHA-256), no raw paths
- No file contents or secret values

## Test Coverage

27 operational risk tests verify:
- All 13 categories classified correctly
- Multi-risk per file
- Highest severity enforcement
- Credential reject outright
- blocks_auto_apply ≠ human rejection
- UnknownSensitiveConfig escalation in sensitive directories
- Deterministic classification
- Reason-code stability (all 13 asserted)
- No secret contents in DTOs
- Non-config files produce no risks
