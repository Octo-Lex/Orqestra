# Native Git Operations

## What Changed in v1.2.0

v1.2.0 expands Orqestra's native Git layer from a status-only pilot into a broader read-only operations layer. Five new operations are available through the `git-bridge` crate:

1. **Repository Snapshot** — composite view of branch, HEAD, status, and changed files
2. **Branch and HEAD Metadata** — commit SHA, message, author, timestamp, detached detection
3. **Changed File Summary** — per-file status with secret/workflow/binary risk flags
4. **Recent Commit Metadata** — bounded commit history with author and parent info
5. **Diff/Stat Read** — per-file change statistics without file contents

## Read-Only Scope

All v1.2.0 native Git operations are **read-only**. They never modify the repository, stage files, create commits, or interact with remotes.

Write and network operations remain on the existing CLI-backed human-triggered Git flow.

## Provider Modes

| Provider | Meaning |
|----------|---------|
| `gix` | Fully native — operation completed entirely through the gix library |
| `gix-hybrid` | Branch/HEAD via gix, counts/file-status via CLI |
| `git-cli-fallback` | Native path failed; CLI returned the result |
| `unavailable` | Neither path could produce a safe result |

Every operation reports its provider in the response DTO.

## CLI Fallback

Every v1.2.0 native Git operation has a CLI fallback path. If the native gix path fails (corrupt repo, missing gix support, edge case), the operation falls back to `git` CLI invocation. The `fallback_used` flag is always set.

Fallback behavior:
- **Never blocks UI** — fallback is transparent and non-fatal
- **Always reported** — `provider` and `fallback_used` fields in every DTO
- **Parity tested** — core states are verified to match CLI output

## Repository Snapshot

The `git_repository_snapshot_cmd` returns a composite DTO:

```json
{
  "repo_root": "/path/to/repo",
  "branch": "master",
  "head": { "sha": "...", "short_sha": "abc123", "message": "...", ... },
  "dirty": true,
  "staged_count": 1,
  "unstaged_count": 2,
  "untracked_count": 3,
  "changed_files": [ { "path": "...", "status": "modified", "risk": "normal" } ],
  "provider": "gix-hybrid",
  "fallback_used": false,
  "parity_status": "match",
  "latency_ms": 42,
  "diagnostics": []
}
```

## Changed File Summary

Changed files include two orthogonal classifications:

- **`file_kind`**: `text` | `binary` | `large` | `unknown`
  - Detected by sampling first 8 KiB for null bytes
  - Files > 10 MiB are classified as `large` by metadata
  - Secret-risk files are never opened for kind detection (`unknown`)

- **`risk`**: `normal` | `secret` | `workflow` | `binary` | `large` | `unknown`
  - `.env`, `.env.*`, `*.pem`, `*.key`, `id_rsa`, `id_ed25519` → `secret`
  - `.github/workflows/**` → `workflow`
  - Path-based classification only — no content inspection for risk

### Secret Safety

- Secret-risk paths are flagged by filename pattern only
- Binary detection never reads secret-risk files
- Symlinks are never followed during classification
- File content is never included in any DTO

## Recent Commit Reads

`git_recent_commits_cmd` returns bounded commit metadata:

- Default limit: 10 commits
- Hard maximum: 100 commits
- No diff body by default
- No remote calls
- No credential access
- Uses gix native traversal with CLI fallback

## Diff/Stat Pilot

`git_diff_stat_cmd` provides per-file change statistics:

```json
{
  "files_changed": 4,
  "insertions": 120,
  "deletions": 31,
  "files": [ { "path": "...", "insertions": 12, "deletions": 2, "binary": false, "risk": "normal" } ],
  "provider": "git-cli-fallback",
  "fallback_used": true,
  "parity_status": "not-tested"
}
```

For v1.2.0, diff/stat is CLI-backed. This is explicitly labeled in the `provider` field. The operation is:
- Read-only
- Structured
- Secret-safe (risk flags by path only)
- Provider-labeled
- Non-blocking

## What Is Still CLI-Backed

The following operations remain on CLI and are not migrated:

- `git commit`
- `git push`
- `git pull`
- `git merge` and conflict resolution
- Credentialed remote operations
- Semantic commit pipeline

## What Is Backlog

Post-v1.2.0 backlog items:

1. Native diff body reads with secret-safe guards
2. Native semantic commit preparation
3. Native commit write pilot (only after read-only layer stabilizes)
4. Native push/pull feasibility study
5. DEB packaging for Linux
6. Windows code signing
7. macOS public artifact

## Troubleshooting

### "Provider: git-cli-fallback" on all operations

This means gix could not open the repository. Common causes:
- Not a git repository
- Corrupt `.git` directory
- Unsupported git repository format

### "Detached HEAD" warning

The HEAD commit is checked out directly rather than on a branch. This is informational and does not affect read operations.

### "fallback_used: true" on snapshot

Some operations fell back to CLI. Check the `diagnostics` array for specific fallback reasons.

### Parity mismatch

If `parity_status` is not `match`, the native and CLI outputs differ. This is a warning, not a failure. The CLI output is authoritative.
