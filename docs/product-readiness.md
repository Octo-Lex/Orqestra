# Product Readiness (v1.5.0)

## Overview

Orqestra v1.5.0 is a public beta with 328 passing tests, review-only agents, hybrid native Git operations, deterministic semantic commit preparation, and an opt-in safe diff context pilot.

This document describes the current capability state and verification status.

---

## Credential Security

- **Provider:** OS keychain (platform-backed)
  - Windows: Windows Credential Manager (verified)
  - macOS: Keychain (implemented, not smoke-tested)
  - Linux: Secret Service / libsecret (fallback to session-only in headless environments)
- **Fallback:** Session-only (in-memory, never persisted)
- **Legacy migration:** Verified (XOR vault removed in v1.1.0, migrated to OS keychain)
- **Token masking:** Verified (PATs never appear in logs, errors, or UI)
- **Security level:** platform-backed (not "production-grade" — no hardware security module)

---

## Real Agent Paths

| Agent | Mode | Endpoint | Context | Status |
|-------|------|----------|---------|--------|
| docs-agent | review-only | `POST /agent/docs` | Agent Context v2 + safe diff context (pilot) | Real AI service |
| bugfix-agent | review-only | `POST /agent/bugfix` | Agent Context v2 + safe diff context (pilot) | Real AI service |
| architect-agent | — | — | — | Not implemented |

### Review-Only Agent Policy

All agents operate in **review-only** mode:

- Agents propose changes (diagnosis + diff)
- Changes are displayed for human review
- Accept applies the patch but does NOT auto-commit
- Commits use the normal human-triggered Git flow
- No autonomous commits, no dependency installs, no workflow edits

### Agent Context Quality (v1.4.0)

Agent Context v2 provides structured, schema-versioned Git metadata to both agents:

- Branch, HEAD SHA, changed file paths/statuses/risk levels
- Risk summary, diff/stat counts, commit groups, proposal summary
- Explicit content policy (all content excluded)
- `review_only: true`, `auto_commit: false`, `auto_apply: false`
- Graceful degradation: context failure does not block agent execution

### Safe Diff Context Pilot (v1.5.0)

Opt-in pilot for bounded diff excerpts:

- **Disabled by default** — enabled only via `ORQESTRA_SAFE_DIFF_CONTEXT` env var
- Only normal-risk, text files under 256 KiB included
- Secret-risk, binary, large, symlink, and workflow-risk files excluded
- Caps: max 5 files, max 80 lines/hunk, max 120 lines/file, max 250 total lines
- Provider: `git-cli-fallback`
- Fields named `safe_diff_context`, `hunks`, `lines` — no `diff`, `raw_diff`, or `patch` keys

---

## Native Git Operations (v1.2.0)

Read-only native Git layer using hybrid gix + CLI fallback:

| Operation | Provider | Native? |
|-----------|----------|---------|
| HEAD SHA read | gix | Yes |
| Branch name read | gix | Yes |
| Recent commit metadata | gix | Yes |
| Commit creation | gix (tree-from-index via CLI) | Partial |
| Repository snapshot | gix hybrid | Partial |
| Changed file summary | gix hybrid | Partial |
| Diff/stat | CLI fallback | No |
| Staging | CLI fallback | No |

### Key Properties

- **Scope:** read-only — no push/pull/merge
- **Providers:** `gix`, `gix-hybrid`, `git-cli-fallback` — explicitly labeled
- **Fallback required:** true — CLI fallback always available
- **Blocking:** false — native operations never block normal Git flow
- **Secret-safe:** true — secret-risk paths detected by path pattern, never read

### Risk Classification (v1.2.1)

Changed file risk classification is path-only (never reads contents):

| Risk | Detection | Examples |
|------|-----------|----------|
| `secret` | Path pattern | `.env`, `*.pem`, `*_rsa`, `credentials.*` |
| `workflow` | Path pattern | `.github/workflows/**`, `.github/actions/**` |
| `binary` | Bounded 8 KiB sampling | Non-text bytes in first 8192 bytes |
| `large` | Size threshold | Files > 10 MiB |
| `unknown` | Symlink detection | Never classified as `normal` |
| `normal` | Default | Everything else |

---

## Semantic Commit Preparation (v1.3.0)

Deterministic, proposal-only commit preparation — no AI dependency:

- **Mode:** proposal-only — never stages, commits, pushes, or pulls
- **Provider:** deterministic-heuristic (path-based scope/type extraction)
- **Scope extraction:** git, desktop, dashboard, docs, ci, build, release
- **Change type inference:** test, docs, ci, build, feat, refactor, chore
- **Confidence scoring:** 1.0 single-scope → 0.3 minimum
- **Commit grouping:** scope grouping + risk isolation
- **Agent context:** content-free — paths, statuses, risk flags only, no file contents
- **Diff body pilot:** disabled by default, bounded 256 KiB, text+normal risk only

### Manifest Enforcement

The validator structurally enforces:

```
mode == "proposal-only"
native_commit_execution == false
autonomous_commit == false
stages_files == false
writes_repository == false
requires_review == true
```

---

## Knowledge Graph

- **Triple store:** Content-addressed, commit-indexed
- **Commit indexer:** Maps commits to graph entries
- **Vector/embedding search:** Implemented in Python AI service (`all-MiniLM-L6-v2` + cosine similarity via `/query_history` endpoint)
- **Natural-language query history:** UI exists, backed by triple store

---

## CRDT Sync

- **Engine:** Loro per-file document model
- **Local merge:** Two-peer offline merge verified
- **Cloud relay:** Not implemented — Cloudflare Durable Object relay is backlog
- **UI:** SyncPanel shows CRDT documents and merge status

---

## Shockwave Merge

- **Status:** Mock/prototype
- **Data:** Uses fixture data, not real merge conflict resolution
- **UI:** Visual merge conflict display exists but is not connected to real merge operations

---

## Dashboard

- **URL:** [orqestra.pages.dev](https://orqestra.pages.dev)
- **Deploy:** CI-driven on master push via Cloudflare Pages
- **Data source:** Generated JSON from roadmap indexer
- **Freshness:** Version, commit SHA, and generation timestamp in footer
- **Limitation:** Static snapshot — not real-time synchronized with repository changes

---

## Structured Errors

9 error codes provide actionable recovery information:

| Code | Title |
|------|-------|
| `REPO_OPEN_FAILED` | Could not open repository |
| `ROADMAP_PARSE_FAILED` | Roadmap could not be loaded |
| `DASHBOARD_FETCH_FAILED` | Dashboard data could not be fetched or generated |
| `GIT_OPERATION_FAILED` | Git operation (pull, push, commit) failed |
| `CREDENTIAL_OPERATION_FAILED` | Credential save, load, or delete failed |
| `AI_SERVICE_UNREACHABLE` | AI service not running or unreachable |
| `AI_KEY_MISSING` | AI API key not configured |
| `AGENT_PROPOSAL_FAILED` | AI agent could not generate a proposal |
| `LINUX_RUNTIME_CAVEAT` | Linux AppImage runtime limitation |

---

## Verification Status

| Area | Tests | Verified |
|------|-------|----------|
| Rust workspace | 328 passing | Yes |
| Desktop TypeScript build | Clean | Yes |
| Manifest validation | Pass | Yes |
| Windows release build | 4.7 MB NSIS | Yes |
| Linux CI build | AppImage | Yes |
| macOS CI build | Universal binary | Yes |

---

## Documentation Doctrine

> Documentation is advisory. Code on disk is authoritative. Release claims must be backed by tests, artifacts, manifests, or verified implementation.
