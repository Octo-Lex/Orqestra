# Patch Governance (v1.7.0+)

## Overview

Patch governance ensures that all agent file modifications go through a validated, atomic, audited pathway. No agent can write files directly — all changes must be proposed, reviewed, and applied through the `PatchApplicationGuard`.

## Architecture

```
Agent proposes → PatchProposal DTO → Human reviews → apply_agent_patch_cmd → Atomic write + Audit trail
                                                 → reject_agent_patch_cmd → Rejection audit trail
```

## PatchApplicationGuard

The guard enforces:

- **Typed DTOs**: `PatchProposal` with `proposal_id`, `path`, `before`, `after`, checksums
- **Forbidden paths**: secrets, `.env`, `*.pem`, `*.key`, `.github/workflows`, lock files, binaries
- **Before-content verification**: content must match expected `before` state
- **Atomic writes**: temp-then-rename; failed writes leave original unchanged
- **JSONL audit trail**: every proposal, application, and rejection recorded

## Proposal Lifecycle

```
proposed → applied | rejected | apply_failed
```

- `proposed` — agent generates `PatchProposal` with stable `proposal_id`
- `applied` — file written atomically, audit entry recorded
- `rejected` — no file change, rejection reason recorded in audit
- `apply_failed` — write failed (atomic rollback), error recorded

**"Accepted" is UI state, not audit status.** The durable audit outcomes are `proposed`, `rejected`, `apply_failed`, and `applied`.

## Agent Types

```rust
pub enum AgentType {
    Docs,
    Bugfix,
}
```

Architect agent is not in this enum — it cannot propose patches.

## Forbidden Paths

| Category | Pattern | Reason |
|----------|---------|--------|
| Secrets | `.env`, `.env.*`, `secrets.*` | Credential leakage risk |
| Crypto keys | `*.pem`, `*.key`, `*.pub` | Private key exposure |
| CI/CD | `.github/workflows/*` | Supply chain risk |
| Lock files | `package-lock.json`, `Cargo.lock` | Dependency integrity |
| Binaries | `*.exe`, `*.dll`, `*.so`, `*.dylib` | Binary injection |

## Server-Side Path Policy

The server enforces allowed paths. The frontend may narrow but never widen:

- **docs-agent**: `README.md`, `docs/*`, `roadmap/*`, `CHANGELOG.md`
- **bugfix-agent**: source code files (excluding forbidden paths)

## Atomic Writes

All file writes use temp-then-rename:

1. Write to `.Orqestra/tmp/{proposal_id}.tmp`
2. Verify write succeeded
3. Rename tmp → target path (atomic on most filesystems)
4. If rename fails, tmp file cleaned up; original unchanged

## Audit Trail

Format: JSONL at `.Orqestra/audit/patch_audit.jsonl`

Each entry contains:
- `proposal_id` — stable correlation ID
- `agent_type` — docs or bugfix
- `timestamp` — ISO 8601
- `outcome` — proposed / applied / rejected / apply_failed
- `path` — target file path
- `reason` — human-readable context

## Configuration

Patch governance is always enabled in v1.7.0+. There is no configuration to disable it.

Manifest section: `product_readiness.patch_governance` with 16 validator gates.

## Security Properties

- Agents never write files directly (AgentRunner auto_commit removed)
- Missing AI service returns error — no fake proposals generated
- `.Orqestra` runtime state is protected during all operations
- Architect agent output has no patch-shaped fields and cannot be passed to `apply_agent_patch_cmd`
