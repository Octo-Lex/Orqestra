# Persistent Onboarding + Project Switching (v2.5.3+)

## Overview

v2.5.3 closes the last user-journey gap before autonomy: onboarding and project context are now durable across restarts.

## Storage

```
{app_data_dir}/app-state.json
```

- Atomic write: write to `.tmp`, then `rename`
- Corrupt file: backed up as `app-state.corrupt.{timestamp}.json`, replaced with default

## Project Identity

```rust
project_id = "proj-" + sha256(canonical_root_path)
```

Stable across moves. Deduped by `project_id`, not by raw path.

## State Structure

```json
{
  "onboarding_completed": true,
  "last_project_root": "/path/to/project",
  "last_project_id": "proj-abc123...",
  "recent_projects": [
    {
      "project_id": "proj-abc123...",
      "root": "/path/to/project",
      "name": "Orqestra",
      "last_opened_at": "2026-06-08T00:00:00Z",
      "last_known_credential_status": "configured",
      "last_known_relay_status": "connected"
    }
  ],
  "last_opened_at": "2026-06-08T00:00:00Z"
}
```

## Status Enums (metadata only)

```rust
CredentialStatus: Unknown | Configured | Missing | Error
RelayConnectionStatus: Unknown | Connected | Disconnected | NeverConnected
```

- `CredentialStatus`: global credential availability, metadata only
- `RelayConnectionStatus`: last known state, recomputed on project open

## Security Boundaries

| Location | Paths | Secrets |
|----------|-------|---------|
| app-state.json | ✅ local paths allowed | ❌ never |
| Diagnostic bundles | ❌ hashed only | ❌ never |
| OS keychain | N/A | ✅ encrypted |

Reset onboarding **never** clears OS-keychain credentials. Credential deletion requires a separate explicit command.

## Project Switching

When switching projects:
1. Disconnect current relay actor (if any)
2. Update `last_project_root` and `last_project_id`
3. Reinitialize sync engine for new project
4. Update recent projects list
5. Persist to disk immediately

## Commands

| Command | Action |
|---------|--------|
| `get_onboarding_state_cmd` | Load state (lazy from disk) |
| `set_onboarding_state_cmd` | Update onboarding + persist |
| `reset_onboarding_cmd` | Clear metadata and/or history (never secrets) |
| `record_project_access_cmd` | Update project status + persist |

## Test Coverage

- 9 onboarding_types unit tests
- 8 persistence integration tests
- Total: 17 new tests
