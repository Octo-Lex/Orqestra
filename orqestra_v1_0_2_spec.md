# Orqestra — v1.0.2 Productization & Trust Hardening Specification

**Version:** 1.0.2  
**Date:** 2026-06-02  
**Status:** Draft — implementation-ready hardening release  
**Release Theme:** Productization & Trust Hardening  
**Base:** v1.0.1 tag, after completion of the “Truthful Release Candidate” hardening release

---

## 1. Executive Summary

Orqestra v1.0.2 is a productization and trust hardening release. It does not introduce a new architectural phase. It converts the v1.0.1 working prototype into something an external developer can install, verify, and trust without reading the source code.

The v1.0.1 baseline proved the core product loop: Markdown roadmap files can be indexed by Rust, rendered by a Tauri desktop app, exported to a public dashboard, represented in a knowledge graph, enriched by semantic commits, synchronized through CRDT primitives, and exercised by one real AI agent path. However, v1.0.1 still has important product gaps:

- Credential storage is encrypted pragmatically, but not cryptographically strong.
- Dashboard data is real, but not guaranteed fresh from CI deployment.
- The desktop app has a production Windows artifact, but not cross-platform release automation.
- Semantic commit operations still depend on shelling out to Git.
- Only the docs-agent execution path is real; bugfix and architect agents remain mock-mode.
- An external user still needs too much project knowledge to verify the product cleanly.

v1.0.2 closes those gaps through five bounded workstreams:

1. Replace the XOR credential vault with Stronghold plus OS-keychain-delegated vault unlocking.
2. Make the public dashboard refresh automatically from CI-generated roadmap JSON.
3. Add cross-platform desktop release automation and artifacts.
4. Replace the semantic commit shell-out path with native `gix` for the commit operation.
5. Replace the bugfix-agent mock with a constrained, user-scoped, human-reviewed AI execution path.

The guiding principle for this release is:

> **Do not add autonomy until installation, credential security, release artifacts, dashboard freshness, and review-only agent execution are trustworthy.**

---

## 2. Release Goals

### 2.1 Primary Goals

| Goal | Description | Outcome |
|---|---|---|
| Real credential security | Replace XOR credential storage with Stronghold-backed secret storage and OS-keychain-delegated vault unlocking | GitHub PATs are protected by a cryptographic vault without requiring a startup password every launch |
| Fresh dashboard CI deployment | Regenerate roadmap JSON in CI and deploy Cloudflare Pages from the generated artifact | Public dashboard reflects the deployed commit, not a stale local snapshot |
| Cross-platform desktop release | Build and attach desktop artifacts for Windows, macOS, and Linux from tagged releases | External users can download a packaged app for their platform |
| Native semantic commit path | Replace shell-outs in the semantic commit path with native `gix` operations | Semantic commits are faster, less fragile, and easier to test |
| Real bugfix-agent review flow | Replace bugfix-agent mock execution with real AI service invocation over user-selected files only | Bugfix agent can propose safe, reviewable diffs without autonomous commits |

### 2.2 Secondary Goals

| Goal | Description | Outcome |
|---|---|---|
| First-run verification path | Document and/or implement a minimal external-reviewer path | A reviewer can verify Orqestra without reverse-engineering the repo |
| Release-claim classification | Preserve the v1.0.1 release-integrity rule | README and release notes distinguish implemented, local-only, mock-mode, scaffolded, and backlog features |
| Security downgrade prevention | Ensure the app never silently falls back to plaintext or XOR credential storage | Failure states are explicit and safe |

### 2.3 Non-Goals

The following are explicitly out of scope for v1.0.2 unless required to unblock one of the primary goals:

- Full Cloudflare Durable Object CRDT relay.
- Full edge worker semantic query API.
- Full `gix` migration for all Git operations.
- Full tree-sitter / AST-based source analysis.
- ML-Master exploration loop completion.
- Architect-agent real execution.
- Automatic bugfix commits.
- Agent-initiated source-file discovery.
- Agent-initiated scope expansion.
- Marketplace, plugin system, billing, or hosted SaaS functionality.
- Code signing and notarization as a hard release blocker. Unsigned artifacts may ship if clearly documented.

---

## 3. Current Baseline

### 3.1 v1.0.1 Verified State

The v1.0.1 baseline includes:

- Rust workspace with 68 passing tests.
- Clean Desktop TypeScript build.
- Clean Dashboard TypeScript build.
- Tauri production build producing a Windows NSIS installer.
- Live Cloudflare Pages dashboard.
- Dashboard consuming generated roadmap JSON rather than primary hardcoded data.
- Markdown roadmap indexing, dependency graph, DOT export, and JSON export.
- Tauri desktop task table, Gantt, Kanban, scheduler, and time tracking UI.
- Git sync through shell-out Git commands.
- Semantic commit pipeline with pending-to-complete backfill.
- Python AI service endpoints for `/extract-intent`, `/embed`, `/query-history`, `/agent/docs`, and a stubbed `/explore`.
- ConfidenceGate and review-oriented agent UI.
- Multi-agent routing with docs, bugfix, and architect workspaces.
- Real docs-agent AI execution with reviewable diffs.
- Knowledge graph, vector search, query history UI, semantic diff UI, and CRDT merge verification.

### 3.2 v1.0.1 Gaps Targeted by v1.0.2

| Gap | v1.0.1 State | v1.0.2 Required State |
|---|---|---|
| Credential encryption | XOR-based encrypted vault with migration path | Stronghold vault, unlocked through OS keychain delegation |
| Dashboard freshness | Real JSON generated locally; live dashboard may show a stale commit | CI regenerates JSON and deploys Pages from the generated artifact |
| Desktop release | Windows x64 NSIS tested only | Windows, macOS, and Linux release artifacts built from tag workflow |
| Git operations | Semantic commit path still shells out | Semantic commit write path uses native `gix`; push/pull may remain shell-out |
| Bugfix agent | Mock execution | Real AI call over user-selected files, review-only |
| AST analysis | Not implemented | Remains backlog; not required for file selection in v1.0.2 |
| Edge worker | Not implemented | Remains backlog |
| ML-Master | Stub markers remain | Remains backlog |

---

## 4. Architectural Positioning

v1.0.2 preserves the v0.5.1 / v1.0.0 / v1.0.1 architecture.

### 4.1 Tauri In-Process Boundary Remains Primary

The desktop renderer continues to call Rust through Tauri `invoke()` commands.

```text
React / TypeScript Renderer
  → Tauri invoke() commands
    → Tauri Rust command layer
      → Pure Rust core crates
```

The future sidecar/gRPC path remains valid, but it is not part of v1.0.2.

### 4.2 Core Crate Purity Remains Mandatory

The following crates must remain pure Rust libraries with zero Tauri dependencies:

- `md-indexer`
- `git-bridge`
- `graph-store`
- `loro-engine`

Tauri-specific code belongs only under:

```text
apps/desktop/src-tauri/
```

Credential UI and command wrappers may live in the Tauri app, but reusable credential abstractions should avoid contaminating the core crates with renderer or desktop-shell concerns.

### 4.3 AI Must Not Block Git Commits

The semantic commit invariant remains unchanged:

```text
Git commit first
  → semantic stub immediately
    → AI backfill asynchronously
      → graph and embedding update later
```

v1.0.2 may replace the commit implementation with native `gix`, but it must not make AI inference a blocking dependency for a standard Git commit.

### 4.4 Credentials Must Stay Behind the Rust Boundary

Raw GitHub PATs and vault unlock secrets must not be exposed to React state or TypeScript except during the short-lived user input event in which a token is submitted.

Preferred boundary:

```text
React input form
  → save_github_token_cmd(token)
    → Rust command layer
      → Stronghold vault
        → OS keychain stores only Stronghold unlock secret
```

After saving, TypeScript receives only a `TokenStatus` DTO. It must not receive the raw token.

### 4.5 Agent Execution Must Remain Human-Reviewed

v1.0.2 expands real agent execution from docs-only to docs plus bugfix, but it does not expand autonomous execution. The bugfix agent may propose diffs only. It must not write files or commit changes until a human approves the proposed diff.

---

## 5. Workstream A — Real Credential Security

### 5.1 Problem

v1.0.1 includes encrypted credential storage, but the encryption is explicitly not production-grade. GitHub PAT storage must be upgraded before Orqestra can be considered trustworthy as a desktop app.

### 5.2 Locked UX and Security Decision

Use **OS keychain delegation** for Stronghold vault unlocking.

Orqestra will use Stronghold as the encrypted vault for GitHub PATs and future desktop secrets. The Stronghold vault unlock secret will be generated by Orqestra and stored in the operating system credential store. The user’s OS login becomes the effective unlock mechanism.

Platform mapping:

| Platform | Unlock secret storage |
|---|---|
| Windows | Windows Credential Manager |
| macOS | Keychain |
| Linux | Secret Service / libsecret where available |

The Stronghold vault stores the GitHub PAT. The OS keychain stores only the Stronghold vault unlock secret.

### 5.3 Rejected Alternatives

The following are explicitly rejected for v1.0.2:

| Alternative | Reason rejected |
|---|---|
| App-startup password prompt | More secure in isolation, but poor UX for a developer desktop app |
| Machine-derived key only | Better than XOR but not user-controllable and still not aligned with normal desktop secret UX |
| Plaintext JSON | Unacceptable credential persistence |
| XOR vault | Not cryptographic-grade |
| React state persistence | Exposes secrets to UI snapshots and developer tooling |
| Silent insecure fallback | Security downgrade must be explicit, not hidden |

### 5.4 Required Behavior

First-run credential flow:

```text
User saves GitHub PAT
  → Rust command checks for Stronghold vault unlock secret in OS keychain
    → if missing, generate high-entropy unlock secret
      → store unlock secret in OS keychain
        → initialize/load Stronghold snapshot with unlock secret
          → write GitHub PAT into Stronghold
            → return TokenStatus only
```

App-startup credential flow:

```text
Desktop app launches
  → no password prompt
    → credential status command checks OS keychain + Stronghold metadata
      → UI shows secure credential state
```

Git operation flow:

```text
User clicks pull/push/test connection
  → Rust command retrieves PAT internally from Stronghold
    → Git operation uses token inside Rust boundary
      → TypeScript receives success/failure DTO only
```

### 5.5 Fallback Behavior

If OS keychain access fails:

- The app must show `Secure credential storage unavailable`.
- The app must not save the PAT to plaintext JSON.
- The app must not save the PAT to XOR storage.
- The app may allow session-only token usage if clearly labeled and never persisted.
- The app must include a diagnostic message that is safe to display and does not include secrets.

### 5.6 Tauri Commands

Add or replace the credential commands under:

```text
apps/desktop/src-tauri/src/commands/credentials.rs
```

Required commands:

```rust
bootstrap_credential_vault_cmd(project_root: String) -> Result<CredentialVaultStatus, CommandError>
save_github_token_cmd(project_root: String, token: String) -> Result<TokenStatus, CommandError>
get_github_token_status_cmd(project_root: String) -> Result<TokenStatus, CommandError>
delete_github_token_cmd(project_root: String) -> Result<TokenStatus, CommandError>
test_github_connection_cmd(project_root: String) -> Result<GitHubConnectionStatus, CommandError>
rotate_vault_unlock_secret_cmd(project_root: String) -> Result<CredentialVaultStatus, CommandError>
```

Do not add a general-purpose command that returns the raw GitHub PAT to TypeScript. If a raw token is temporarily needed for an existing Git shell-out path, retrieve it in Rust and pass it directly to the Git operation without returning it to the renderer.

### 5.7 DTOs

```typescript
export type CredentialProvider =
  | 'stronghold-os-keychain'
  | 'session-only'
  | 'unavailable';

export type CredentialMigrationState =
  | 'not_required'
  | 'required'
  | 'in_progress'
  | 'complete'
  | 'failed';

export interface CredentialVaultStatus {
  available: boolean;
  provider: CredentialProvider;
  vault_exists: boolean;
  unlock_secret_exists: boolean;
  migration_state: CredentialMigrationState;
  last_error: string | null;
}

export interface TokenStatus {
  exists: boolean;
  provider: CredentialProvider;
  label: 'GitHub PAT';
  last_updated: string | null;
  migration_state: CredentialMigrationState;
}

export interface GitHubConnectionStatus {
  ok: boolean;
  username: string | null;
  scopes: string[];
  message: string;
}
```

### 5.8 Secret Naming

Use stable service/account names so credentials survive app restarts and upgrades.

Recommended OS-keychain entry:

```text
service: com.elephantrocklab.orqestra.stronghold
account: default-vault-unlock-secret
```

Recommended Stronghold vault keys:

```text
client: orqestra-desktop
vault: credentials
github_pat_key: github/pat/default
metadata_key: github/pat/default/status
```

### 5.9 Migration from v1.0.1 XOR Vault

If a credential exists in the legacy XOR vault:

1. Detect legacy credential presence.
2. Mark UI state as `Credential migration required`.
3. Read the legacy credential once inside Rust.
4. Bootstrap Stronghold and OS keychain unlock secret.
5. Save the PAT to Stronghold.
6. Verify by reading credential metadata and testing GitHub connection if possible.
7. Delete the legacy credential only after verification succeeds.
8. Emit a migration audit event with no raw secrets.

If migration fails:

- Preserve the legacy credential.
- Mark status as `Credential migration failed`.
- Do not delete the only copy of the credential.
- Do not downgrade to plaintext storage.
- Show a remediation hint.

### 5.10 UI Changes

The Git sync panel must display one of these states:

```text
No GitHub credential stored
Credential stored securely
Credential migration required
Credential migration failed
Secure credential storage unavailable
Session-only credential active
```

Required actions:

- Save token.
- Replace token.
- Delete token.
- Test GitHub connection.
- Migrate legacy credential.
- Use session-only token when secure persistence is unavailable.

### 5.11 Security Rules

- Never log raw PATs.
- Never log Stronghold unlock secrets.
- Never include PATs in UI state snapshots.
- Never write PATs to `.Orqestra/`.
- Never commit credential files.
- Never expose a general raw-token getter to TypeScript.
- Mask token-like substrings in errors before returning them to the UI.
- Treat OS-keychain failure as a blocking persistence error, not as permission to use plaintext.

### 5.12 Acceptance Criteria

- GitHub PATs are stored in Stronghold, not XOR vault or plaintext JSON.
- The Stronghold unlock secret is stored in the OS keychain.
- On Windows, the unlock secret is stored in Windows Credential Manager.
- App restart does not require a user-entered Stronghold password.
- Push/pull/test connection works after app restart.
- Deleting credentials removes GitHub access.
- Legacy XOR credential migration is safe and verified before deletion.
- Secure storage failure never silently downgrades to plaintext or XOR storage.
- Logs and UI errors contain no raw PATs or unlock secrets.

---

## 6. Workstream B — Fresh Dashboard CI Deployment

### 6.1 Problem

v1.0.1 deploys a live dashboard that consumes real generated data, but the dashboard JSON can become stale because it is not guaranteed to be generated and deployed by CI for the latest `master` commit.

### 6.2 Required Behavior

The dashboard must be generated and deployed from CI using the current repository state.

Pipeline:

```text
push to master or release tag
  → checkout repository
    → run md-indexer tests
      → export roadmap JSON from roadmap/
        → build dashboard
          → deploy Cloudflare Pages when credentials exist
            → dashboard footer displays generated_at + source commit
```

### 6.3 Required Artifact

Continue using:

```text
apps/dashboard/public/orqestra-roadmap.json
```

The generated artifact must include:

```json
{
  "generated_at": "2026-06-02T00:00:00Z",
  "source": {
    "repo": "github.com/Elephant-Rock-Lab/Orqestra",
    "branch": "master",
    "commit": "<git-sha>",
    "workflow_run_id": "<github-actions-run-id>"
  },
  "summary": {
    "total_tasks": 0,
    "done": 0,
    "backlog": 0,
    "in_progress": 0,
    "blocked": 0
  },
  "sprints": [],
  "tasks": []
}
```

### 6.4 Dashboard Runtime Requirements

The dashboard must:

- Fetch `/orqestra-roadmap.json`.
- Render loading state while the JSON loads.
- Render a safe error state if the JSON is missing or malformed.
- Display `generated_at` and source commit hash in the footer.
- Display a stale-data warning if the artifact is older than a configured threshold.
- Avoid falling back to primary mock data in production.

Mock data may exist only as test fixtures or Storybook/demo fixtures.

### 6.5 CI Workflow

Add or update:

```text
.github/workflows/dashboard.yml
```

Required behavior:

- Run on `push` to `master`.
- Run on pull requests as build-only.
- Run on manual dispatch.
- Generate roadmap JSON before building dashboard.
- Fail the dashboard build if JSON export fails.
- Upload `apps/dashboard/dist` as an artifact.
- Deploy to Cloudflare Pages only when `CLOUDFLARE_API_TOKEN` and related secrets are present.

Suggested workflow skeleton:

```yaml
name: Dashboard

on:
  push:
    branches: [master]
  pull_request:
  workflow_dispatch:

jobs:
  build-dashboard:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: npm
      - run: cargo test -p md-indexer
      - run: npm ci
      - run: cargo run -p orqestra -- export --format=json --out apps/dashboard/public/orqestra-roadmap.json
      - run: npm run build -w apps/dashboard
      - uses: actions/upload-artifact@v4
        with:
          name: dashboard-dist
          path: apps/dashboard/dist

  deploy-dashboard:
    needs: build-dashboard
    if: github.ref == 'refs/heads/master' && secrets.CLOUDFLARE_API_TOKEN != ''
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          name: dashboard-dist
          path: apps/dashboard/dist
      - run: npx wrangler pages deploy apps/dashboard/dist --project-name orqestra-dashboard
        env:
          CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
          CLOUDFLARE_ACCOUNT_ID: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
```

The exact export command must match the real CLI package name before merge.

### 6.6 Acceptance Criteria

- `orqestra-roadmap.json` is generated in CI from `roadmap/`.
- Dashboard build fails if JSON export fails.
- Dashboard deploys to Cloudflare Pages from CI when credentials are present.
- Dashboard footer shows the source commit and generation timestamp.
- The deployed dashboard source commit matches the deployed workflow run.
- Production dashboard does not use primary hardcoded mock data.
- README documents the dashboard freshness behavior.

---

## 7. Workstream C — Cross-Platform Desktop Release

### 7.1 Problem

v1.0.1 has a verified Windows x64 production build, but a productized desktop release needs repeatable cross-platform artifacts.

### 7.2 Required Behavior

On a version tag, GitHub Actions must build packaged desktop artifacts for:

- Windows x64.
- macOS universal or macOS x64/arm64.
- Linux x64.

At minimum, v1.0.2 must produce one artifact per operating system family.

### 7.3 Expected Artifact Types

| Platform | Expected artifact |
|---|---|
| Windows | `.msi` or `.exe` / NSIS installer |
| macOS | `.dmg`, `.app.tar.gz`, or `.zip` |
| Linux | `.AppImage`, `.deb`, or `.rpm` |

Code signing and notarization are not hard blockers for v1.0.2, but unsigned artifacts must be clearly labeled.

### 7.4 Required Workflow

Add or update:

```text
.github/workflows/desktop-release.yml
```

Suggested skeleton:

```yaml
name: Desktop Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build-desktop:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: windows-latest
            artifact-name: orqestra-windows
          - os: macos-latest
            artifact-name: orqestra-macos
          - os: ubuntu-latest
            artifact-name: orqestra-linux
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: npm
      - run: npm ci
      - run: cargo test --workspace
      - run: npm run build -w apps/desktop
      - run: npm run tauri build -w apps/desktop
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact-name }}
          path: apps/desktop/src-tauri/target/release/bundle/**
```

Linux system dependencies must be installed explicitly if the Tauri build requires them.

### 7.5 Packaged-App Smoke Test

Each platform job should include at least one smoke check after packaging:

- Verify expected bundle directory exists.
- Verify at least one installer artifact exists.
- Verify the frontend assets are embedded.
- Verify the Tauri binary exists.

Full GUI launch automation may remain backlog if platform CI makes it impractical.

### 7.6 README Requirements

README must include:

- Where to download artifacts.
- Which artifacts are unsigned.
- How to build locally per platform.
- Known platform-specific warnings.
- Minimum OS versions if known.

### 7.7 Acceptance Criteria

- Tagged release workflow builds Windows, macOS, and Linux artifacts.
- Artifacts are uploaded to workflow artifacts or attached to the GitHub release.
- At least one packaging smoke check runs per platform.
- README documents artifact status and unsigned-artifact caveats.
- Existing Windows NSIS path remains working.

---

## 8. Workstream D — Native Semantic Commit Path

### 8.1 Problem

The v1.0.1 semantic commit pipeline works, but Git operations still shell out. The immediate product risk is the semantic commit path, because it is central to Orqestra’s identity and performance claims.

### 8.2 Scope

v1.0.2 migrates only the semantic commit write path to native `gix`.

In scope:

- Detect repository state.
- Stage selected files or respect already-staged files, depending on current behavior.
- Create a normal Git commit with `gix`.
- Return commit hash.
- Write semantic stub after commit.
- Preserve AI backfill behavior.
- Preserve graph update behavior.
- Benchmark commit latency.

Out of scope:

- Full push/pull migration.
- Credentialed remote operations through `gix`.
- Merge conflict resolution rewrite.
- Full replacement of all `std::process::Command` Git usage.

### 8.3 Required Behavior

```text
User approves commit or agent diff
  → Rust git-bridge validates repo
    → native gix writes standard Git commit
      → semantic stub written to .Orqestra/graph/commits/{hash}.json
        → UI receives commit hash and pending indexing state
          → AI backfill runs asynchronously
```

The commit must remain a standard Git commit. The semantic layer must remain additive and non-blocking.

### 8.4 API Surface

In `crates/git-bridge`, expose a native path such as:

```rust
pub struct NativeCommitRequest {
    pub project_root: PathBuf,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub task_id: Option<String>,
    pub paths: Vec<PathBuf>,
}

pub struct NativeCommitResult {
    pub hash: String,
    pub parent_hashes: Vec<String>,
    pub semantic_stub_path: PathBuf,
    pub elapsed_ms: u64,
}

pub fn semantic_commit_native(req: NativeCommitRequest) -> Result<NativeCommitResult, GitBridgeError>;
```

The old shell-out implementation may remain temporarily behind a feature flag or fallback path, but tests must exercise the native path.

### 8.5 Error Contract

Git errors crossing into Tauri must be mapped to stable command errors:

| Code | Meaning |
|---|---|
| `GIT_REPO_NOT_FOUND` | Project root is not a Git repository |
| `GIT_NOTHING_TO_COMMIT` | No staged or selected changes exist |
| `GIT_INDEX_ERROR` | Index read/write failed |
| `GIT_COMMIT_ERROR` | Native commit operation failed |
| `SEMANTIC_STUB_ERROR` | Commit succeeded but semantic stub write failed |
| `AI_BACKFILL_PENDING` | Commit succeeded; AI backfill has not completed |

If the Git commit succeeds but semantic stub writing fails, the UI must show a recoverable warning. It must not roll back the Git commit.

### 8.6 Benchmarks

Add a simple benchmark or measured integration test:

- Shell-out semantic commit baseline.
- Native `gix` semantic commit time.
- Stub write time.
- Total synchronous elapsed time.

Target:

```text
Native semantic commit synchronous path: < 100ms for small commits on a typical SSD dev machine
Hard upper warning threshold: 200ms
```

The target is a benchmark goal, not a correctness failure condition for CI.

### 8.7 Acceptance Criteria

- Semantic commit path uses native `gix` for commit creation.
- The resulting commit is a standard Git commit.
- Semantic stub is still written immediately after commit.
- AI backfill remains asynchronous.
- Existing semantic commit tests remain green.
- New tests cover native commit success, empty commit failure, invalid repo failure, and semantic-stub failure behavior.
- Shell-out push/pull may remain in place and documented as backlog.

---

## 9. Workstream E — Real Bugfix-Agent Review Flow

### 9.1 Problem

v1.0.1 includes multi-agent routing and a real docs-agent path, but the bugfix-agent path is still mock-mode. v1.0.2 should make one more agent real without introducing uncontrolled autonomy.

### 9.2 Locked Scope Decision

Use **user-selected file scope**.

The bugfix agent must not infer editable files from task labels, task prose, source scanning, or heuristic discovery. Before execution, the user must select the exact files the bugfix agent may inspect and propose changes for.

Heuristic file discovery belongs to a later AST/tree-sitter workstream.

### 9.3 Required Flow

```text
User selects bug-labeled task
  → Agent router selects bugfix workspace
    → UI asks user to select allowed files
      → run_bugfix_agent_cmd sends task + selected files to AI service
        → AI service returns proposed edits only
          → Desktop renders reviewable diff
            → ConfidenceGate forces propose mode
              → User accepts or rejects
                → On accept, Rust writes files and creates standard Git commit
                  → Semantic stub is written
                    → AI backfill updates semantic metadata asynchronously
```

### 9.4 File Scope Rules

The bugfix agent may:

- Read only user-selected files.
- Propose edits only to user-selected files.
- Return notes about files it wishes it could inspect, but not inspect them.
- Return a failure if selected files are insufficient.

The bugfix agent must not:

- Expand file scope without user approval.
- Edit files not explicitly selected.
- Read arbitrary `src/**` files.
- Modify config, CI, dependency manifests, or lockfiles unless explicitly selected and allowed by policy.
- Commit automatically.

### 9.5 UI Requirements

Add a pre-run file scope step:

```text
Selected task: TASK-2026-XXX
Selected workspace: bugfix
Allowed files:
  [ ] src/foo.ts
  [ ] src/bar.ts
  [ ] tests/foo.test.ts

Run bugfix agent
```

Required UI states:

```text
Select files before running
Running bugfix agent
Proposed diff ready
Rejected by user
Accepted by user
Commit created
Semantic indexing pending
Semantic indexing complete
Agent failed safely
```

### 9.6 AI Service Contract

Add or finalize:

```http
POST /agent/bugfix
```

Request:

```json
{
  "task": {
    "id": "TASK-2026-XXX",
    "title": "Fix failing login redirect",
    "body": "...",
    "labels": ["bug"]
  },
  "allowed_files": [
    {
      "path": "src/auth/redirect.ts",
      "content": "..."
    },
    {
      "path": "tests/auth/redirect.test.ts",
      "content": "..."
    }
  ],
  "constraints": {
    "allowed_paths": ["src/auth/redirect.ts", "tests/auth/redirect.test.ts"],
    "max_files_changed": 2,
    "auto_commit": false,
    "may_request_more_files": true,
    "test_command": "npm test -- redirect"
  }
}
```

Response:

```json
{
  "summary": "Fixed login redirect fallback when next URL is missing.",
  "confidence": 0.78,
  "has_breaking_change": false,
  "edits": [
    {
      "path": "src/auth/redirect.ts",
      "before": "...",
      "after": "..."
    },
    {
      "path": "tests/auth/redirect.test.ts",
      "before": "...",
      "after": "..."
    }
  ],
  "test_plan": {
    "recommended_command": "npm test -- redirect",
    "was_run": false,
    "notes": "Run after applying patch."
  },
  "needs_more_files": false,
  "requested_files": [],
  "notes": [
    "No dependency files changed."
  ]
}
```

If the agent needs more files:

```json
{
  "summary": "Cannot safely propose a fix with the selected files.",
  "confidence": 0.31,
  "has_breaking_change": false,
  "edits": [],
  "needs_more_files": true,
  "requested_files": [
    {
      "path": "src/auth/session.ts",
      "reason": "Redirect behavior depends on session fallback."
    }
  ],
  "notes": [
    "No files were modified."
  ]
}
```

The UI may let the user approve additional files and rerun the agent. The agent itself must not fetch those files automatically.

### 9.7 ConfidenceGate Policy

For v1.0.2, bugfix-agent output must never auto-commit.

Temporary config:

```yaml
confidence_gate:
  auto_commit: 1.01
  propose: 0.00
  flag: 0.00
  breaking_change_override: always_propose
```

All successful bugfix-agent outputs go to human review.

### 9.8 File Write and Commit Policy

Before human approval:

- No file writes.
- No Git staging.
- No Git commit.
- Proposed edits exist only in memory or as review data.

After human approval:

- Rust applies edits.
- Tests may be run if configured.
- Standard Git commit is created.
- Semantic stub is written.
- AI backfill runs asynchronously.
- Commit is labeled as human-approved agent output.

### 9.9 Acceptance Criteria

- Bug-labeled task routes to bugfix workspace.
- User must select allowed files before running the bugfix agent.
- Bugfix-agent request includes only selected files.
- AI service is called through a real `/agent/bugfix` endpoint or equivalent implementation.
- Returned edits are shown as a reviewable diff.
- Agent cannot edit unselected files.
- Agent can request additional files without reading them.
- User can accept or reject proposed diff.
- Accepted diff creates a standard Git commit and semantic stub.
- No autonomous bugfix commit occurs.
- UI clearly labels the result as human-approved.

---

## 10. Testing Requirements

### 10.1 Existing Tests Must Remain Green

Required baseline:

```bash
cargo test --workspace
npm run build -w apps/desktop
npm run build -w apps/dashboard
npm run tauri build -w apps/desktop
```

If E2E tests are available:

```bash
npm run test:e2e
```

### 10.2 New Tests by Workstream

| Workstream | Required tests |
|---|---|
| Credential security | OS-keychain adapter tests with mock store; Stronghold save/status/delete smoke tests; migration test from XOR vault; token masking test |
| Dashboard freshness | CI export test; dashboard renders generated commit metadata; production build fails on missing JSON |
| Desktop release | Matrix workflow packaging smoke checks; artifact existence checks |
| Native semantic commit | Native commit success; empty commit failure; invalid repo failure; semantic stub failure warning; latency measurement |
| Bugfix agent | User-selected file scope enforcement; unselected edit rejection; needs-more-files response; diff review accept/reject; no auto-commit |

### 10.3 Credential Security Test Cases

```text
save token → restart app → status says secure credential exists
save token → inspect Tauri store JSON → no raw token
save token → inspect .Orqestra/ → no raw token
delete token → restart app → status says no credential
legacy XOR token exists → migrate → Stronghold token exists → legacy token deleted
migration write fails → legacy token preserved
OS keychain unavailable → persistence blocked, session-only optional
error contains token-like string → returned error is masked
```

### 10.4 Bugfix-Agent Test Cases

```text
selected files: [A, B] → response edits A only → accepted
selected files: [A, B] → response edits C → rejected before review
selected files: [A] → agent requests B → no automatic read occurs
selected files: [] → run button disabled
agent confidence 0.99 → still propose-only
agent returns breaking change → still propose-only with warning
user rejects diff → no file write, no commit
user accepts diff → file write, standard commit, semantic stub
```

### 10.5 Manual Smoke Test

Before tagging v1.0.2:

1. Clone repo fresh.
2. Run `cargo test --workspace`.
3. Run desktop and dashboard TypeScript builds.
4. Build dashboard from freshly generated roadmap JSON.
5. Confirm dashboard footer shows current commit.
6. Trigger or manually run Cloudflare Pages deployment.
7. Build desktop artifacts for Windows, macOS, and Linux.
8. Launch packaged app on at least one platform.
9. Save GitHub PAT.
10. Restart app and verify secure credential status.
11. Confirm no PAT exists in legacy JSON or `.Orqestra/`.
12. Run GitHub connection test.
13. Create a semantic commit through the native path.
14. Verify semantic stub appears.
15. Run bugfix agent on a bug task with selected files only.
16. Review proposed diff.
17. Reject once and confirm no file write.
18. Run again, accept diff, and confirm standard Git commit plus semantic stub.
19. Publish release artifacts.
20. Confirm README and CHANGELOG classify all claims truthfully.

---

## 11. CI/CD Requirements

### 11.1 Dashboard Workflow

The dashboard workflow must be authoritative for public dashboard freshness.

Required jobs:

- `build-dashboard`
- `deploy-dashboard`

The deploy job must run only for `master` or configured release branches and only when Cloudflare secrets exist.

### 11.2 Desktop Release Workflow

The desktop release workflow must build artifacts from tags.

Required jobs:

- `build-desktop` matrix across Windows, macOS, Linux.
- `upload-artifacts` or GitHub release attachment step.

Optional for v1.0.2:

- Code signing.
- macOS notarization.
- Auto-update feed generation.

### 11.3 Security CI

Add lightweight checks:

```text
ripgrep for token-like test fixtures outside approved test fixture files
unit test for token masking
unit test for credential DTOs never carrying raw token fields
```

Recommended forbidden field names in renderer-facing DTOs:

```text
token
pat
secret
password
unlock_secret
```

Exceptions must be explicit and reviewed.

---

## 12. Documentation Updates

### 12.1 README

README must state the exact v1.0.2 status:

- What works locally.
- What is deployed.
- What remains backlog.
- How to download desktop artifacts.
- Which artifacts are unsigned.
- How to build desktop artifacts locally.
- How dashboard freshness works.
- How credentials are stored.
- How to recover from unavailable secure credential storage.
- Which agent paths are real.
- Which agent paths remain mock-mode.
- What “human-approved agent diff” means.

### 12.2 CHANGELOG

Add:

```markdown
## v1.0.2 — Productization & Trust Hardening

### Added
- Stronghold credential vault unlocked through OS keychain delegation
- CI-generated dashboard data and Cloudflare Pages deployment refresh
- Cross-platform desktop release workflow for Windows, macOS, and Linux
- Native `gix` semantic commit path
- Real bugfix-agent execution path with user-selected file scope and human-reviewed diffs

### Changed
- Credential storage no longer relies on XOR-based persistence
- Dashboard deployment now reflects CI-generated roadmap JSON
- Bugfix-agent output is explicitly review-only and cannot auto-commit
- Release artifacts are produced from tagged builds

### Security
- Stronghold vault unlock secret is stored in the OS credential store
- Raw GitHub PATs are never returned to TypeScript after save
- Insecure credential persistence fallback is disallowed
- Token masking added for logs and UI errors

### Known Limitations
- Push/pull may still shell out to Git
- Full edge worker remains backlog
- Durable Object CRDT relay remains backlog
- AST/tree-sitter analysis remains backlog
- ML-Master exploration remains incomplete
- Architect-agent execution remains mock-mode
- Bugfix-agent cannot discover files automatically
- Agent commits require human approval
```

### 12.3 Release Notes

Suggested release-note language:

```text
v1.0.2 turns Orqestra’s v1.0.1 prototype into a more trustworthy external release. Credentials now use a Stronghold vault unlocked through the OS keychain, the public dashboard is refreshed from CI-generated roadmap data, desktop artifacts are built for Windows, macOS, and Linux, semantic commits begin moving from shell-outs to native gix, and the bugfix agent can now produce human-reviewed diffs over user-selected files.
```

### 12.4 Security Note

README must include a credential storage note:

```text
Orqestra stores GitHub PATs in a Stronghold encrypted vault. The vault unlock secret is generated locally and stored in the OS credential store: Windows Credential Manager, macOS Keychain, or Linux Secret Service/libsecret where available. If secure credential storage is unavailable, Orqestra will not silently persist credentials insecurely.
```

---

## 13. Roadmap Task Updates

Create or update the following roadmap tasks.

### TASK-2026-071 — Stronghold Vault with OS Keychain Unlock

```yaml
---
pm-task: true
id: TASK-2026-071
title: "Replace XOR credentials with Stronghold and OS keychain unlock"
type: Task
status: backlog
priority: Critical
sprint: "Sprint 17"
epic: "Productization & Trust Hardening"
assignee: "agent-security"
labels:
  - security
  - credentials
  - desktop
  - tauri
---
```

Acceptance criteria:

- GitHub PATs are stored in Stronghold.
- Stronghold unlock secret is stored in OS credential store.
- Windows uses Windows Credential Manager.
- App restart does not require user-entered vault password.
- Legacy XOR credential migration is safe.
- No silent insecure fallback exists.
- Token masking tests pass.

### TASK-2026-072 — CI-Generated Dashboard Freshness

```yaml
---
pm-task: true
id: TASK-2026-072
title: "Deploy dashboard from CI-generated roadmap JSON"
type: Task
status: backlog
priority: Critical
sprint: "Sprint 17"
epic: "Productization & Trust Hardening"
assignee: "agent-devops"
labels:
  - dashboard
  - cloudflare
  - ci
  - release
---
```

Acceptance criteria:

- CI exports roadmap JSON from `roadmap/`.
- Dashboard build fails if export fails.
- Cloudflare Pages deploy uses generated artifact.
- Dashboard footer shows source commit and generation timestamp.
- Deployed dashboard source commit matches workflow run.

### TASK-2026-073 — Cross-Platform Desktop Release Workflow

```yaml
---
pm-task: true
id: TASK-2026-073
title: "Build desktop release artifacts for Windows, macOS, and Linux"
type: Task
status: backlog
priority: High
sprint: "Sprint 17"
epic: "Productization & Trust Hardening"
assignee: "agent-desktop"
labels:
  - desktop
  - tauri
  - release
  - ci
---
```

Acceptance criteria:

- Tagged release workflow builds Windows artifact.
- Tagged release workflow builds macOS artifact.
- Tagged release workflow builds Linux artifact.
- Artifacts are uploaded or attached to release.
- README documents unsigned artifact caveats.

### TASK-2026-074 — Native gix Semantic Commit Path

```yaml
---
pm-task: true
id: TASK-2026-074
title: "Replace semantic commit shell-out path with native gix"
type: Task
status: backlog
priority: High
sprint: "Sprint 17"
epic: "Productization & Trust Hardening"
assignee: "agent-rust"
labels:
  - git
  - gix
  - semantic-commit
  - rust
---
```

Acceptance criteria:

- Semantic commit creation uses native `gix`.
- Standard Git commit is produced.
- Semantic stub is written immediately.
- AI backfill remains asynchronous.
- Empty commit and invalid repo errors are stable.
- Latency is benchmarked.

### TASK-2026-075 — Real Bugfix-Agent with User-Selected File Scope

```yaml
---
pm-task: true
id: TASK-2026-075
title: "Replace bugfix-agent mock with real review-only execution"
type: Task
status: backlog
priority: High
sprint: "Sprint 17"
epic: "Productization & Trust Hardening"
assignee: "agent-bugfix"
labels:
  - agents
  - ai-service
  - bugfix
  - safety
---
```

Acceptance criteria:

- User must select allowed files before bugfix agent runs.
- Bugfix-agent request includes only selected files.
- Agent can propose edits only to selected files.
- Agent can request more files but cannot fetch them automatically.
- Returned edits are shown as reviewable diffs.
- No autonomous commit occurs.
- Accepted diff creates standard commit and semantic stub.

---

## 14. Release Checklist

### 14.1 Code

- [ ] Stronghold vault implemented.
- [ ] OS-keychain unlock secret storage implemented.
- [ ] Windows Credential Manager path verified.
- [ ] Legacy XOR migration implemented.
- [ ] Plaintext/XOR fallback removed or disabled.
- [ ] Dashboard JSON generated in CI.
- [ ] Cloudflare Pages deploy uses CI artifact.
- [ ] Desktop release workflow added.
- [ ] Windows artifact generated.
- [ ] macOS artifact generated.
- [ ] Linux artifact generated.
- [ ] Native `gix` semantic commit path implemented.
- [ ] Bugfix-agent real AI endpoint implemented.
- [ ] Bugfix-agent file scope UI implemented.
- [ ] Bugfix-agent review diff flow implemented.
- [ ] README updated.
- [ ] CHANGELOG updated.

### 14.2 Tests

- [ ] `cargo test --workspace`.
- [ ] `npm run build -w apps/desktop`.
- [ ] `npm run build -w apps/dashboard`.
- [ ] `npm run tauri build -w apps/desktop`.
- [ ] Credential migration test.
- [ ] Token masking test.
- [ ] Dashboard freshness test.
- [ ] Native semantic commit tests.
- [ ] Bugfix-agent scope enforcement tests.
- [ ] Bugfix-agent diff accept/reject tests.
- [ ] Release workflow smoke checks.

### 14.3 Security

- [ ] No PAT in Tauri store JSON.
- [ ] No PAT in `.Orqestra/`.
- [ ] No PAT in logs.
- [ ] No Stronghold unlock secret in logs.
- [ ] No raw-token getter exposed to TypeScript.
- [ ] OS-keychain unavailable state tested.
- [ ] Session-only fallback, if present, is non-persistent and clearly labeled.

### 14.4 Release

- [ ] Create tag `v1.0.2`.
- [ ] Push tag.
- [ ] Confirm desktop artifacts for Windows, macOS, and Linux.
- [ ] Confirm dashboard deployed from latest CI-generated JSON.
- [ ] Publish release notes.
- [ ] Verify README links.
- [ ] Verify dashboard source commit.
- [ ] Run minimum demo script.

---

## 15. Success Definition

v1.0.2 is successful when an external reviewer can do the following without relying on mock claims or source-code archaeology:

1. Open the public dashboard and see roadmap data generated by CI from the current repository state.
2. Download desktop artifacts for Windows, macOS, or Linux.
3. Launch a packaged desktop app.
4. Open an Orqestra repository.
5. Store GitHub credentials without plaintext, XOR, or repeated startup password prompts.
6. Restart the app and verify credential status persists securely.
7. Create a semantic commit through the native `gix` path.
8. Run the docs-agent path and see a human-reviewed diff.
9. Run the bugfix-agent path over user-selected files and see a human-reviewed diff.
10. Confirm no agent commits autonomously.
11. Read README and release notes that accurately distinguish implemented features, local-only features, mock-mode features, scaffolded features, and backlog.

The release is not successful if:

- Credentials still rely on XOR storage.
- Stronghold requires a startup password prompt as the default UX.
- The app silently falls back to plaintext credential persistence.
- Dashboard data is still deployed from a stale local artifact.
- Only Windows desktop artifacts exist.
- Semantic commits still rely only on shell-out Git.
- Bugfix-agent can read or edit files not selected by the user.
- Bugfix-agent can auto-commit.
- Public documentation overclaims autonomy or production security.

---

## 16. Post-v1.0.2 Backlog

After v1.0.2, continue in this recommended order:

1. Complete full native `gix` Git operation migration, including push/pull.
2. Add tree-sitter AST code analysis.
3. Add heuristic file discovery with user confirmation.
4. Implement dependency vulnerability scanning.
5. Improve first-run onboarding and recovery UX.
6. Implement ML-Master exploration loop.
7. Build the Cloudflare edge worker.
8. Add Durable Object CRDT relay.
9. Expand real agent execution to architect-agent.
10. Introduce carefully gated autonomous commits for low-risk docs-only or formatting-only work.
11. Add code signing, notarization, and auto-update feeds.

---

## Appendix A — Suggested Branch Plan

```bash
git checkout master
git pull
git checkout -b hardening/v1.0.2
```

Suggested commit sequence:

```text
feat(security): unlock Stronghold vault through OS keychain
feat(security): migrate legacy XOR credentials safely
ci(dashboard): deploy Pages from CI-generated roadmap JSON
ci(desktop): build cross-platform Tauri release artifacts
feat(git): create semantic commits through native gix
feat(agent): run bugfix agent over user-selected files
feat(agent): add bugfix diff review accept-reject flow
docs(release): document v1.0.2 productization status
```

---

## Appendix B — Minimum Demo Script

A v1.0.2 demo should follow this order:

1. Show the repository `roadmap/` files.
2. Show CI-generated `orqestra-roadmap.json`.
3. Open the live dashboard and show footer commit metadata.
4. Open the GitHub Actions dashboard workflow that deployed it.
5. Show desktop release artifacts for Windows, macOS, and Linux.
6. Launch the packaged desktop app.
7. Open the Orqestra repo.
8. Save a GitHub PAT.
9. Restart the app and show secure credential status.
10. Show that no plaintext/XOR token exists.
11. Create a semantic commit through the native path.
12. Show semantic stub pending indexing.
13. Select a bug task.
14. Select allowed files.
15. Run the bugfix agent.
16. Show proposed diff.
17. Reject once and confirm no write occurred.
18. Run again or accept the existing proposal.
19. Accept the diff.
20. Show standard Git commit and semantic stub.
21. Show README status table with accurate claim classifications.

---

## Appendix C — Credential Implementation Sketch

Recommended internal modules:

```text
apps/desktop/src-tauri/src/commands/credentials.rs
apps/desktop/src-tauri/src/security/keychain.rs
apps/desktop/src-tauri/src/security/stronghold_store.rs
apps/desktop/src-tauri/src/security/token_mask.rs
```

Recommended Rust traits:

```rust
pub trait VaultUnlockKeyStore {
    fn get_or_create_unlock_secret(&self) -> Result<Vec<u8>, CredentialError>;
    fn rotate_unlock_secret(&self) -> Result<Vec<u8>, CredentialError>;
    fn delete_unlock_secret(&self) -> Result<(), CredentialError>;
}

pub trait SecretVault {
    fn put_secret(&self, key: &str, value: &[u8]) -> Result<(), CredentialError>;
    fn has_secret(&self, key: &str) -> Result<bool, CredentialError>;
    fn delete_secret(&self, key: &str) -> Result<(), CredentialError>;
    fn with_secret<T>(&self, key: &str, f: impl FnOnce(&[u8]) -> T) -> Result<T, CredentialError>;
}
```

Renderer-facing APIs should expose status and operations, not raw secrets.

---

## Appendix D — Bugfix Agent Policy

The bugfix agent is allowed to reason, propose, and request scope. It is not allowed to assume scope.

Policy:

```text
Allowed:
- Read selected files.
- Propose edits to selected files.
- Explain test plan.
- Ask user for more files.
- Return no-op when confidence is low.

Forbidden:
- Read unselected files.
- Edit unselected files.
- Stage changes before approval.
- Commit before approval.
- Modify dependency manifests unless selected and approved.
- Modify CI unless selected and approved.
- Modify credential/security code unless selected and approved with warning.
```

---

## Appendix E — Release Integrity Rule

Every public claim in README, release notes, dashboard copy, or demo script must be classified as one of:

```text
Implemented and verified
Implemented but local-only
Implemented but mock-mode
Scaffolded but not wired
Backlog
```

No feature may be described as complete unless it has one of:

- Passing automated test.
- Working local demo.
- Successful deployment artifact.
- Successful packaged artifact.

For security claims, the bar is higher: a feature may not be described as secure unless the insecure fallback paths are explicitly tested or removed.

This rule remains mandatory for v1.0.2 and should remain part of the release process afterward.
