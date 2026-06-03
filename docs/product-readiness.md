# Product Readiness (v1.1.0)

## What Changed in v1.1.0

v1.1.0 is Orqestra's first product-readiness release after the v1.0.x beta hardening line. It:

- Validates platform-backed credential storage
- Adds a real review-only bugfix-agent path
- Improves first-run guidance and structured error recovery
- Begins a non-blocking native Git status pilot
- Preserves tested Windows and Linux beta platforms

## Credential Security

- **Provider:** OS keychain (platform-backed)
  - Windows: Windows Credential Manager
  - macOS: Keychain (implemented, not smoke-tested)
  - Linux: Secret Service / libsecret
- **Fallback:** Session-only (in-memory, never persisted)
- **Legacy migration:** Verified (XOR vault → OS keychain)
- **Token masking:** Verified (PATs never appear in logs, errors, or UI)
- **Security level:** platform-backed

## Real Agent Paths

| Agent | Mode | Status |
|-------|------|--------|
| docs-agent | review-only | Real AI service (verified in v1.0.x) |
| bugfix-agent | review-only | Real AI service (new in v1.1.0) |

## Review-Only Agent Policy

All agents operate in **review-only** mode:

- Agents propose changes (diagnosis + diff)
- Changes are displayed for human review
- Accept applies the patch but does NOT auto-commit
- Commits use the normal human-triggered Git flow
- No autonomous commits, no dependency installs, no workflow edits

## First-Run Flow

New users see a Getting Started checklist:

1. Open a repository
2. Load roadmap tasks
3. Open live dashboard
4. Try no-key beta mode
5. (Optional) Configure real-AI mode

The guide is dismissible and reopenable. AI mode is clearly labeled as optional.

## Structured Errors

9 error codes provide actionable recovery information:

| Code | Title |
|------|-------|
| `REPO_OPEN_FAILED` | Could not open repository |
| `ROADMAP_PARSE_FAILED` | Roadmap could not be loaded |
| `DASHBOARD_FETCH_FAILED` | Dashboard data could not be loaded |
| `GIT_OPERATION_FAILED` | Git operation failed |
| `CREDENTIAL_OPERATION_FAILED` | Credential operation failed |
| `AI_SERVICE_UNREACHABLE` | AI service not running |
| `AI_KEY_MISSING` | AI API key not configured |
| `AGENT_PROPOSAL_FAILED` | Agent proposal failed |
| `LINUX_RUNTIME_CAVEAT` | Linux runtime limitation |

Each error includes: likely causes, suggested actions, reporting hint, and a secret-safe guarantee.

## Native Git Pilot

- **Operation:** status read-only
- **Provider:** gix 0.84 (branch detection) + git CLI (counts)
- **Fallback:** Always falls back to git CLI if native path fails
- **Blocking:** No — the pilot never blocks normal Git operations

## Remaining Beta Limitations

- Windows installer is unsigned (SmartScreen warnings expected)
- Linux tested on Ubuntu 24.04 only
- macOS has no bundled artifact
- Native Git status is a pilot (CLI fallback always available)
- Agents are review-only, not autonomous

## Platform Status

| Platform | Status |
|----------|--------|
| Windows x64 | tested |
| Linux x64 | tested |
| macOS | build-feasibility-verified |

## What Is Still Backlog

- Full native gix migration for write operations
- Third real agent path (after bugfix-agent stabilizes)
- DEB packaging for Linux
- macOS public artifact path
- Windows code signing
- Tree-sitter/AST analysis
- Cloudflare edge worker
- Durable Object CRDT relay
- ML-Master exploration loop
