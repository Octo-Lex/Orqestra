# Product Readiness (v2.0.0)

## Overview

Orqestra v2.0.0 is a **governed AI-native development beta** with 447 passing tests, three bounded agents, patch governance, code intelligence, and hybrid native Git operations.

This document describes the current capability state and verification status.

---

## Classification

Orqestra is a **governed AI-native development beta**. Not prototype. Not full product. A governed beta.

The key distinction: all three agent roles exist, but authority remains bounded:

- docs and bugfix can propose (via governance)
- bugfix is symbol-aware and patch-governed
- architect can plan but not mutate
- no agent can auto-commit
- AI-service failure does not fabricate plans
- repository and `.Orqestra` runtime state remain protected

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

## Agent Portfolio

| Agent | Mode | Endpoint | Context | Writes | Patch-Governed |
|-------|------|----------|---------|--------|---------------|
| docs-agent | review-only | `POST /agent/docs` | Agent Context v2 + safe diff context (pilot) | via governance | yes |
| bugfix-agent | review-only, symbol-aware | `POST /agent/bugfix` | Agent Context v2 + symbols + safe diff context (pilot) | via governance | yes |
| architect-agent | read-only planner | `POST /agent/architect` | Agent Context v2 + symbols + ADRs + risks | **no** | no |
| autonomy | disabled | — | — | — | — |

### Review-Only Agent Policy

- Agents propose changes (diagnosis + diff)
- Changes are displayed for human review
- Accept applies the patch through governance (atomic write + audit trail)
- Commits use the normal human-triggered Git flow
- No autonomous commits, no dependency installs, no workflow edits

### Architect Non-Mutating Policy

- Architect produces plans only (no patches, no file writes)
- Architect output has no `before`/`after`/`edits` fields
- Architect plan cannot be passed to `apply_agent_patch_cmd`
- Missing AI service returns error (no fake plans)

---

## Patch Governance (v1.7.0)

All agent file modifications go through `PatchApplicationGuard`:

- **Typed DTOs**: `PatchProposal` with stable `proposal_id`
- **Forbidden paths**: secrets, `.env`, `*.pem`, `*.key`, `.github/workflows`, lock files, binaries
- **Before-content verification**: content must match expected `before` state
- **Atomic writes**: temp-then-rename; failed writes leave original unchanged
- **JSONL audit trail**: every proposal, application, and rejection recorded
- **Server-side policy**: frontend may narrow but never widen allowed paths
- **AgentRunner auto_commit removed**: agents never write files directly

---

## Code Intelligence (v1.8.0)

- **Engine**: tree-sitter 0.24 (pure Rust crate, zero Tauri/git-bridge dependency)
- **Languages**: Rust, TypeScript
- **Symbol extraction**: functions, structs, enums, traits, impls, modules, interfaces, classes
- **File exclusion**: binary, secret, generated, >256 KiB
- **Parse error threshold**: ERROR/MISSING ratio >30% = ParseError
- **Deterministic ordering**: line_start → line_end → kind → name → parent
- **Agent integration**: bugfix-agent receives symbols; docs-agent disabled by default
- **Architect integration**: affected symbols in plan output

---

## Git Provider System (v1.6.0)

### GitProvider Enum

```rust
pub enum GitProvider {
    Gix,           // Pure gix native
    GixHybrid,     // gix + CLI fallback (e.g., tree-from-index via git write-tree)
    GitCliFallback, // CLI fallback for unsupported operations
    DeterministicHeuristic, // Deterministic file kind/risk classification
    NotImplemented, // Operation not yet migrated
}
```

### Per-Operation Provider Report

13 operations tracked with provider, native status, fallback availability, and read-only flag.

### Response Wrappers

`RecentCommitsResult` and `DiffStatResult` carry provider information even on empty results.

### Non-Mutating Diagnostics

`build_provider_report()` only executes read-only operations. Mutating ops reported from static registry only.

---

## Semantic Commit Preparation (v1.3.0)

- Deterministic heuristics (no AI dependency)
- Confidence scoring (0.0–1.0)
- Commit grouping by file area
- Content-free agent context (paths + risk flags only)
- Proposal-only — never auto-commits

---

## Agent Context v2 (v1.4.0)

- Schema-versioned (`schema_version` field)
- Content-free (no file contents in context)
- Explicit `ContentPolicy` with all content excluded
- `review_only: true`, `auto_commit: false`, `auto_apply: false`
- Graceful degradation: context failure does not block agent execution
- Forbidden-field scan scoped to `git_context` object keys only

---

## Safe Diff Context (v1.5.0)

- Opt-in via `ORQESTRA_SAFE_DIFF_CONTEXT` environment variable
- `SafeDiffContext` DTO with policy caps
- Eligibility gate (only review-only agents)
- Bounded diff hunk extraction
- No `diff`/`raw_diff`/`patch` keys in DTO — uses `safe_diff_context`/`hunks`/`lines`

---

## First-Run Environment Checks (v2.0.0)

10 non-mutating probes:

| # | Check | Source | Mutating? |
|---|-------|--------|-----------|
| 1 | Git available | `git --version` probe | No |
| 2 | Repository selectable | project_root exists | No |
| 3 | Roadmap valid | `_index.md` parseable (bounded read, 4 KiB) | No |
| 4 | AI service reachable | `/health` ping (optional/degraded) | No |
| 5 | Credential provider available | keyring probe (read-only) | No |
| 6 | Dashboard export visible | file existence check | No |
| 7 | Agent endpoints available | `/health` ping (optional/degraded) | No |
| 8 | Patch governance enabled | audit dir existence check | No |
| 9 | Code intelligence enabled | bounded probe on test string | No |
| 10 | Git provider resolved | single read-only provider report | No |

AI service and agent endpoint unavailability → `optional`/`degraded` status (not setup failure).

---

## Diagnostics Bundle (v2.0.0)

13 diagnostic files, all secret-redacted, non-mutating:

| File | Content |
|------|---------|
| `app.json` | App version, platform |
| `readiness.json` | Environment readiness report |
| `project-validation.json` | Project validation result |
| `git-provider.json` | Per-operation provider diagnostics |
| `credential-status.json` | Credential provider availability |
| `agent-matrix.json` | Agent mode, endpoint, availability |
| `patch-governance.json` | Policy version, audit entries |
| `code-intel.json` | Languages supported, parse probe status |
| `roadmap-status.json` | Parse status, task count |
| `recent-errors.json` | Recent command errors |
| `system.txt` | OS, arch, git version |
| `ai-health.json` | AI service health check |
| `dashboard-status.json` | Dashboard data freshness |

**No secrets, no source bodies, no raw diffs, no private file contents.** Machine-checked by redaction tests.

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
| Rust workspace | 447 passing | Yes |
| Desktop TypeScript build | Clean | Yes |
| Manifest validation | Pass | Yes |
| Windows release build | NSIS installer | Yes |
| Linux CI build | AppImage | Yes |
| macOS CI build | Universal binary | Yes |
| First-run probes | 12 tests (non-mutating) | Yes |
| Redaction verification | 8 tests (machine-checked) | Yes |

---

## Documentation Doctrine

> Documentation is advisory. Code on disk is authoritative. Release claims must be backed by tests, artifacts, manifests, or verified implementation.
