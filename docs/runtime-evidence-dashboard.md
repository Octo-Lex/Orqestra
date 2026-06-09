# Runtime Evidence Dashboard & Cap Expansion (v2.9.0)

## Overview

v2.9.0 uses the clean structural runtime evidence from v2.8.1 to expand pilot ergonomics and add an evidence dashboard. No semantic autonomy expansion.

**Principle:** Evidence-informed ergonomics, not authority expansion.

## What Changed

### Session Cap Expansion

```rust
DEFAULT_MAX_AUTO_APPLY_PER_SESSION = 5    // unchanged
MIN_AUTO_APPLY_PER_SESSION = 1            // unchanged
MAX_AUTO_APPLY_PER_SESSION = 15           // raised from 10
```

### Evidence Dashboard

Two new commands:

| Command | Purpose |
|---------|---------|
| `get_evidence_dashboard_cmd` | Pre-built 19-path evidence matrix with safety invariants |
| `evaluate_path_matrix_cmd` | Evaluate custom path set through decision engine |

### Evidence DTOs

```rust
PathMatrixEvidence {
    total_tested, allowed_count, rejected_count,
    requires_review_count, rejection_rate,
    records: Vec<PathDecisionRecord>,
    safety_invariants: SafetyInvariantsResult,
    top_rejection_reasons: Vec<(String, usize)>,
}

PathDecisionRecord {
    path, path_class, normalized_path,
    is_readme, confidence, decision, reason, has_traversal,
}

SafetyInvariantsResult {
    no_source_files_touched, no_workflow_files_touched,
    no_secret_files_touched, no_dep_files_touched,
    auto_commit_always_false, changelog_rejected,
    roadmap_rejected, traversal_rejected, wrong_agent_rejected,
}
```

## What Did NOT Change

| Policy | Status |
|--------|--------|
| Allowed paths | `docs/` + `README.md` — unchanged |
| docs/** threshold | 0.80 — unchanged |
| README threshold | 0.90 — unchanged |
| auto_commit | false — always |
| CHANGELOG.md | excluded |
| roadmap/** | excluded |
| Source files | excluded |
| Link-fix special case | not added |
| Agent types | docs only |

## Runtime Evidence (v2.8.1)

50-path structural decision matrix:
- 13 allowed (26%), 37 rejected (74%), 0 requires-review
- 9/9 safety invariants pass
- 74% rejection rate is correct (narrow allowlist working as designed)

## Future

After external beta runtime evidence:
- If clean → v2.10.0: Docs link-fix autonomy (separately gated)
- If friction → v2.10.0: Policy refinement
