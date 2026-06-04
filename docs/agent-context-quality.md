# Agent Context Quality

## What Changed in v1.4.0

v1.4.0 improves what agents **know**, not what agents are **allowed to do**.

Docs-agent and bugfix-agent now receive structured Agent Context v2 metadata from the semantic commit preparation layer stabilized in v1.3.x.

## Agent Context v2

Agent Context v2 is a schema-versioned, content-policy-aware DTO that provides rich repository metadata without file contents.

**Schema version:** `agent-context-v2`

**What agents receive:**
- Branch and HEAD SHA
- Changed file paths, statuses, file kinds, and risk levels
- Risk summary counts (normal, secret, workflow, binary, large, unknown)
- Diff/stat counts (files changed, insertions, deletions)
- Commit group suggestions (scope, change type, risk, suggested title)
- Semantic proposal summary (title, scope, change type, risk level, confidence)
- Recent commit subjects
- Provider identifier
- Explicit content policy

**What agents do NOT receive:**
- File contents from Git context
- Raw diffs or patches
- Secret values or tokens
- Binary data
- Large file contents
- Symlink target contents

## Review-Only Policy

Both agents remain review-only:

| Constraint | Value |
|-----------|-------|
| review_only | true |
| auto_commit | false |
| auto_apply | false |
| stages_files | false |
| writes_repository | false |
| native_commit_execution | false |
| autonomous_actions | false |

## Docs-Agent Context

Docs-agent receives Agent Context v2 focused on documentation-relevant changes:

- Branch, HEAD SHA
- Changed file paths and statuses
- Risk summary
- Commit groups (especially docs-scoped groups)
- Semantic proposal summary
- Recent commit subjects
- Content policy

Request payload includes `git_context_status` for auditability and explicit `constraints` with `review_only: true`.

## Bugfix-Agent Context

Bugfix-agent receives Agent Context v2 focused on source changes:

- Branch, HEAD SHA
- Allowed file paths (user-selected)
- Changed file paths, statuses, and risk flags
- Risk summary
- Commit groups
- Semantic proposal summary
- Recent commit subjects
- Diff/stat counts
- Content policy

Context does not expand `allowed_paths` or `max_files_changed`.

## Diagnostics UI

The Agent Context Panel displays what context was sent to agents:

- Schema version
- Changed file count and paths (repo-relative only)
- Risk summary
- Commit group count and titles
- Semantic proposal title and confidence
- Provider
- Content policy (all exclusions listed)
- Last context build time
- Agent mode: review-only

The Agent Diagnostics Panel shows agent constraints:

- review_only: true
- auto_commit: false
- auto_apply: false
- Allowed paths
- Max files changed

Neither panel displays file contents, raw diffs, tokens, or secret values.

## Security and Secret Safety

- Git context is content-free by construction
- The `ContentPolicy` struct explicitly declares all exclusions
- Forbidden-field scan (path-aware) verifies no content keys appear in serialized payloads
- `secret_count` and `secret_contents_excluded` are safe metadata keys, not content
- `raw_diffs: false` in content policy is a safe declaration, not content
- Secret-risk files appear only as paths with risk flags, never with contents
- Binary, large, and symlink files are metadata-only

## Evidence and Limitations

- Evidence records payload structure, not subjective AI quality
- Before/after comparison is at the DTO structure level
- No claim says agents are autonomous
- No claim says agents can commit
- Agent output quality is structural unless real AI invocation produces verifiable results

## Backlog

- Safe diff-body context pilot (opt-in, bounded)
- Tree-sitter/AST context exploration
- Agent output quality evaluations with fixed fixtures
- Native commit write feasibility study
