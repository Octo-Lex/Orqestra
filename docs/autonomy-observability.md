# Autonomy Observability & Pilot Evaluation (v2.7.0)

## Overview

v2.7.0 makes the docs-only autonomy pilot fully observable. Every decision is durably audited, summaries distinguish session from persisted metrics, and pilot evaluation is evidence-based.

**Win condition:** Every autonomy decision is durably auditable, summaries are accurate after restart, diagnostics are redacted, and pilot evaluation can be based on evidence rather than anecdotes.

## Audit Persistence

```
.Orqestra/agents/docs/auto-apply-audit.jsonl
```

- Append-only JSONL, one record per line
- Never rewrites prior lines
- Malformed lines skipped and counted, never fatal
- Includes `audit_schema_version` for future migration clarity

## Audit Record Schema (v1)

```json
{
  "audit_schema_version": 1,
  "timestamp": "2026-06-09T00:00:00Z",
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

## Metrics

### Session Metrics (in-memory, reset on restart)

```rust
AutonomyMetrics {
    total_decisions, allowed_count, rejected_count,
    requires_review_count, rejection_reasons,
    path_classes_allowed, path_classes_rejected,
    manual_commits_after_auto_apply,
}
```

### Audit-Derived Metrics (from persisted JSONL)

Computed on demand from `auto-apply-audit.jsonl`. Survives restart.

Summary command returns both:
```rust
AutonomySummary {
    session_metrics,    // since app start
    audit_metrics,      // from persisted records
    audit_record_count,
    malformed_audit_lines,
    recent_decisions,   // last 20 for local UI
    safety_report,
}
```

## Pilot Safety Report

```rust
PilotSafetyReport {
    report_timestamp,
    pilot_duration,          // enabled_at to now
    total_auto_applied,
    total_rejected,
    total_requires_review,
    rejection_rate,          // rejected / total
    top_rejection_reasons,   // top 5
    no_secrets_in_audit,     // verified by scan
    no_auto_commits,         // always true
    no_source_files_touched, // verified by path_class
    audit_completeness,      // records / decisions
    session_cap_hit_count,
    manual_follow_up_rate,   // commits / auto-applies
}
```

## Commands

| Command | Returns | Scope |
|---------|---------|-------|
| `get_autonomy_summary_cmd` | Full summary with both metrics + safety report | Local UI |
| `export_autonomy_audit_cmd` | All persisted records + malformed count | Local |
| `get_autonomy_diagnostics_cmd` | Aggregate counts, hashed IDs, safety report | Diagnostics |
| `record_manual_commit_after_auto_apply_cmd` | Void (gated by known applied proposal_id) | Tracking |

## Diagnostic Boundaries

| Context | Raw proposal IDs | Recent decisions | Source bodies |
|---------|-----------------|-----------------|---------------|
| Local UI | ✅ | ✅ (last 20) | ❌ |
| Diagnostics | ❌ (hashed) | ❌ (aggregate only) | ❌ |

## Redaction Scan

Recursive scan of JSON values (not keys) for secret-shaped patterns:
- `ork_v2_`, `ghp_`, `gho_`, `Bearer `, `-----BEGIN `

Field names like `no_secrets_in_audit` do not trigger false positives.

## Manual Follow-up Tracking

`record_manual_commit_after_auto_apply_cmd(proposal_id)`:
- Only accepts proposal IDs that exist in the audit as `applied: true`
- Unknown or non-applied IDs are rejected
- Increments `manual_commits_after_auto_apply` in session metrics

## Test Coverage

- 28 new observability tests
- Audit persistence, malformed line handling, metrics computation
- Redaction scan (clean, catches secrets, no false positives)
- Pilot safety report generation
- Manual follow-up validation
