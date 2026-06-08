# Docs Autonomy Expansion (v2.8.0)

## Overview

v2.8.0 expands the **ergonomics** of the docs-only autonomy pilot, not its authority surface.

**Principle:** Expand pilot usability, not pilot authority.

## What Changed

### Configurable Session Cap

```rust
MIN_AUTO_APPLY_PER_SESSION = 1
DEFAULT_MAX_AUTO_APPLY_PER_SESSION = 5   // unchanged
MAX_AUTO_APPLY_PER_SESSION = 10          // Rust-enforced ceiling
```

- User can configure cap between 1 and 10
- Default remains 5
- Frontend cannot set cap above 10 (clamped server-side)
- Cap changes persisted in `app-state.json` with audit trail

### Cap Audit Trail

When cap changes:
```json
{
  "cap_changed_at": "2026-06-09T00:00:00Z",
  "cap_previous_value": 5
}
```

### RequiresReview Explanation

When cap is exceeded:
```rust
RequiresReviewExplanation {
    reason: "Session auto-apply cap reached",
    session_applied: 5,
    configured_cap: 5,
    remaining: 0,
    reset_behavior: "Cap resets on app restart",
}
```

### Summary Cap Display

```rust
AutonomySummary {
    configured_cap: 5,
    session_cap_remaining: 2,  // 5 - 3 applied
    ...
}
```

## What Did NOT Change

| Policy | Status |
|--------|--------|
| Allowed paths | `docs/` + `README.md` only (unchanged) |
| Allowed agent | `docs` only (unchanged) |
| docs/** threshold | 0.80 (unchanged) |
| README.md threshold | 0.90 (unchanged) |
| auto_commit | false (always) |
| CHANGELOG.md | excluded |
| roadmap/** | excluded |
| Source files | excluded |
| Link-fix special case | not added |

## Commands

| Command | Change |
|---------|--------|
| `set_autonomy_settings_cmd` | Now accepts `max_auto_apply_per_session` |
| `get_autonomy_summary_cmd` | Returns `configured_cap`, `session_cap_remaining` |
| `get_autonomy_diagnostics_cmd` | Returns `configured_cap`, `cap_hit_count` |

## Guardrails

- Frontend cannot widen cap beyond `MAX_AUTO_APPLY_PER_SESSION` (10)
- Cap changes are persisted and audited
- Changing cap does not change allowlist, agent, threshold, or commit policy
- Cap exceeded routes to `RequiresReview` — never writes
- `RequiresReview` explains remaining cap and reset behavior

## Future

After collecting runtime evidence from external beta:

- If evidence clean → v2.9.0: Docs Link-Fix Autonomy (separately gated)
- If evidence shows friction → v2.9.0: Autonomy Policy Refinement
