# Semantic Commit Preparation

## What Changed in v1.3.0

v1.3.0 introduces **proposal-only** semantic commit preparation. It uses the stabilized read-only Git layer (repository snapshots, changed-file summaries, diff/stat, recent commits) to generate structured commit proposals.

**It does not stage files, create commits, push, pull, or perform any autonomous Git operation.**

## Proposal-Only Mode

All semantic commit preparation output is a **proposal**. The user must review and manually apply it through the existing human-triggered commit flow.

The proposal DTO always includes:
- `write_operations: false`
- `requires_review: true`
- `provider: "deterministic-heuristic"`

## Inputs Used

The proposal builder composes these read-only operations:

| Input | Source | v1.2.x Stabilized |
|-------|--------|--------------------|
| Repository snapshot | `repository_snapshot()` | ✅ v1.2.0 |
| Changed-file summary | `parse_changed_files()` | ✅ v1.2.0, hardened v1.2.1 |
| Diff/stat | `diff_stat()` | ✅ v1.2.0, hardened v1.2.1 |
| Recent commits | `recent_commits()` | ✅ v1.2.0 |

## Deterministic Heuristics

The proposal builder uses **path-based heuristics only**. No file content is read by default.

### Scope Extraction

| Path Pattern | Scope |
|-------------|-------|
| `crates/git-bridge/**` | `git` |
| `apps/desktop/**` | `desktop` |
| `apps/dashboard/**` | `dashboard` |
| `docs/**` | `docs` |
| `.github/**` | `ci` |
| `scripts/**` | `build` |
| `roadmap/**`, `demo/**` | `release` |
| Root build files (`Cargo.toml`, etc.) | `build` |

### Change Type Heuristics

| Condition | Type |
|-----------|------|
| All test files | `test` |
| All doc files | `docs` |
| Any workflow files | `ci` |
| All build config files | `build` |
| New source files in `crates/` or `apps/` | `feat` |
| Modified source files | `refactor` |
| Default | `chore` |

### Confidence Scoring

- **1.0**: Single scope, single type, no risk files
- **0.8**: Multiple files, single scope
- **0.6**: Multiple scopes
- **0.4**: Risk files present
- **Minimum**: 0.3

## Commit Grouping Suggestions

Files are grouped by:
1. Scope (extracted from path)
2. Risk isolation (secret-risk, workflow-risk, unknown-risk in separate groups)
3. Type consistency

Groups are **suggestions only** — the user decides how to commit.

## Secret-Safe Behavior

- Secret-risk files are flagged by path only — **contents are never read**
- Binary, large, and symlink files are excluded from analysis
- Agent context is **content-free** — only paths, statuses, and risk flags

## Optional Diff Body Pilot

Disabled by default. Controlled by `SEMANTIC_PREP_DIFF_BODY_ENABLED=true` env var.

When enabled, reads small safe text diffs only if:
- `file_kind = "text"`
- `risk = "normal"`
- File size ≤ 256 KiB
- Not a symlink
- Not secret-risk
- Not workflow-risk (by default)

The proposal builder works identically with or without the diff body pilot.

## Agent Context Integration

Docs-agent and bugfix-agent now receive safe Git context:

```json
{
  "branch": "master",
  "head_short_sha": "abc123",
  "changed_file_paths": ["README.md"],
  "changed_file_statuses": ["modified"],
  "risk_flags": [],
  "diff_stat": { "files_changed": 1, "insertions": 10, "deletions": 2 },
  "recent_commit_subjects": ["feat(git): add snapshot"],
  "risk_summary": { "secret_count": 0, "workflow_count": 0, ... }
}
```

**No file contents are included.** Only metadata.

## What Still Uses the Existing Git Flow

- `git commit` — human-triggered
- `git push` — human-triggered
- `git pull` — human-triggered
- `git merge` — human-triggered
- All network operations — human-triggered
- Semantic commit execution — existing `semantic_commit()` → CLI pipeline

## What Is Not Implemented

- Native commit execution
- Autonomous commits
- Auto-staging files
- AI-assisted commit message generation (backlog)
- Full diff body reading (pilot, disabled)

## Troubleshooting

### "No changes to commit" proposal

The working tree is clean. Stage or modify files and refresh.

### Low confidence score

Multiple scopes or risk files detected. Review the proposal carefully.

### "Review Required" badge

All proposals require review. This is by design — v1.3.0 never creates commits.
