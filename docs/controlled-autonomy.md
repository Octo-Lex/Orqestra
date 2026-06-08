# Controlled Low-Risk Autonomy Pilot (v2.6.0)

## Overview

v2.6.0 introduces the first autonomous action in Orqestra: **docs-only auto-apply**.

**Win condition:** A docs-only proposal can be automatically applied under explicit user opt-in, while every higher-risk path fails closed and no commit is created.

## Autonomy Settings

```rust
AutonomySettings {
    enabled: false,                         // Disabled by default
    allowed_agent: "docs",                  // Only docs-agent
    allowed_operation: "auto-apply",        // No auto-commit
    auto_commit: false,                     // Always false
    docs_safe_paths: ["docs/", "README.md"],// Pilot allowlist
    max_patch_bytes: 32768,                 // 32KB
    min_confidence_docs: 0.80,             // docs/** threshold
    min_confidence_readme: 0.90,           // README.md stricter
    max_auto_apply_per_session: 5,         // Rate limit
}
```

## 12-Gate Decision Chain

| Gate | Check | Source |
|------|-------|--------|
| 1 | `settings.enabled == true` | Persisted |
| 2 | `agent == Docs` | Server-side |
| 3 | `operation == "auto-apply"` | Persisted |
| 4 | `auto_commit == false` | Persisted |
| 5 | No path traversal | Computed |
| 6 | Path in docs-safe allowlist | Persisted |
| 7 | `blocks_auto_apply == false` | Operational risk |
| 8 | Not forbidden | Patch guard |
| 9 | `patch_size <= max` | Server-computed |
| 10 | `confidence >= threshold` | Caller-supplied |
| 11 | `before_checksum` matches | Server-computed |
| 12 | `session_count < cap` | Process-scoped |

## Allowlist

**Allowed:**
- `docs/**` — documentation files
- `README.md` — project readme (stricter threshold)

**Explicitly excluded (narrower than server policy):**
- `CHANGELOG.md` — release-sensitive
- `roadmap/**` — task-sensitive
- `src/**`, `lib/**`, `apps/**`, `crates/**`, `services/**` — source code
- `.github/**` — workflows
- `Cargo.toml`, `Cargo.lock`, `package.json` — dependencies
- `release-manifest.json`, `wrangler.toml`, `tauri.conf.json` — config
- `.env*`, `.Orqestra/**` — secrets/internal
- Binary files — `.png`, `.exe`, `.zip`, etc.

## Frontend Trust Boundary

```
Frontend may:
  - Request auto-apply
  - Supply proposal and confidence
  - Request settings changes (validated server-side)

Frontend may NOT:
  - Define autonomy policy
  - Supply patch size
  - Widen allowlist
  - Enable auto-commit
  - Override server-side decisions
```

## Decision Outcomes

| Decision | Action |
|----------|--------|
| `Allowed` | Apply through PatchApplicationGuard, increment session counter |
| `Rejected(reason)` | No write, record audit with reason code |
| `RequiresReview` | No write, record audit, route to human |

`RequiresReview` **never** writes files.

## Audit Records

```json
{
  "timestamp": "2026-06-08T00:00:00Z",
  "proposal_id": "prop-abc",
  "agent": "docs",
  "path_class": "docs",
  "policy_decision": "allowed",
  "reason_codes": [],
  "before_checksum": "sha256...",
  "after_checksum": "sha256...",
  "applied": true,
  "auto_commit": false,
  "policy_version": 1
}
```

No source bodies, tokens, or raw paths in audit.

## Per-Session Cap

Default: 5 auto-applies per session. After cap exceeded, patches route to `RequiresReview`.

## Commands

| Command | Action |
|---------|--------|
| `set_autonomy_settings_cmd` | Enable/disable with server-side validation |
| `get_autonomy_settings_cmd` | Read current settings |
| `auto_apply_patch_cmd` | Attempt auto-apply with full gate chain |

## Test Coverage

- 33 auto_apply unit tests (decision engine, path classification, audit)
- 17 onboarding persistence tests
- Total: 50 new tests in v2.5.3 + v2.6.0
