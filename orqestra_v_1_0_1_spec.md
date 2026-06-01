# Orqestra — v1.0.1 Operational Hardening Specification

**Version:** 1.0.1  
**Date:** 2026-06-01  
**Status:** Draft — implementation-ready hardening release  
**Release Theme:** Truthful Release Candidate  
**Base:** v1.0.0 tag, after completion of Phases 0–6

---

## 1. Executive Summary

Orqestra v1.0.1 is an operational hardening release. It does not introduce a new architectural phase. Instead, it makes the visible product claims of v1.0.0 true, deployable, secure, and externally demonstrable.

The v1.0.0 codebase established the core architecture: Markdown-native project management, Tauri desktop, Rust core crates, semantic commits, knowledge graph, vector search, CRDT sync, multi-agent workspace UI, a dashboard scaffold, and self-hosting roadmap files. However, several pieces remain either mock-mode, dev-mode, hardcoded, undeployed, or declared-but-unused.

v1.0.1 closes those gaps by focusing on four production-readiness outcomes:

1. The public dashboard uses real roadmap data instead of hardcoded mock data.
2. The desktop app produces a real Tauri production build.
3. GitHub credentials are stored securely through Stronghold instead of JSON storage.
4. At least one agent execution path invokes the real AI service and produces a reviewable diff.

The guiding principle for this release is simple: **do not add more surface area until the existing surface area is honest.**

---

## 2. Release Goals

### 2.1 Primary Goals

| Goal | Description | Outcome |
|---|---|---|
| Real dashboard data | Replace hardcoded dashboard data with generated JSON from `roadmap/` | Dashboard reflects repository truth |
| Cloudflare Pages deployment | Deploy the static dashboard to a public Cloudflare Pages URL | Stakeholders can view live progress |
| Production desktop build | Generate a working packaged Tauri app | Desktop is no longer dev-server-only |
| Stronghold credentials | Store GitHub PATs through encrypted Stronghold storage | No credentials in plaintext JSON |
| Real docs-agent execution | Replace one mock agent path with a real AI-service invocation | Agent flow becomes demonstrable end-to-end |

### 2.2 Non-Goals

The following are explicitly out of scope for v1.0.1 unless required to unblock the primary goals:

- Full Cloudflare Durable Object CRDT relay
- Full edge worker semantic query API
- Complete gix migration for all Git operations
- Full tree-sitter AST analysis
- Full ML-Master exploration loop
- Full multi-agent autonomous execution
- Automatic agent commits without human review
- Marketplace, plugin system, billing, or hosted SaaS functionality

---

## 3. Current Baseline

### 3.1 Completed in v1.0.0

The v1.0.0 baseline includes:

- Markdown roadmap indexing
- Dependency graph CLI
- Tauri desktop scaffold
- Git sync UI
- AI service integration stubs and verified local endpoints
- Semantic commit pipeline with optimistic stubs
- ConfidenceGate
- Gantt, Kanban, scheduler, and time tracking UI
- Multi-agent workspace UI and routing
- Graph store and vector search
- Query history
- Semantic diff
- Shockwave merge UI
- Loro CRDT engine and two-peer merge verification
- Sync panel
- Public dashboard scaffold
- GitHub Actions workflow for Orqestra agents
- Self-hosting roadmap files
- README and changelog updates

### 3.2 Known Gaps to Close

| Gap | Current State | v1.0.1 Required State |
|---|---|---|
| Dashboard data | Hardcoded mock data in dashboard source | Generated from real `roadmap/` files |
| Dashboard deployment | Static build exists but not deployed | Deployed through Cloudflare Pages |
| Tauri production build | Dev mode verified only | `npm run tauri build` succeeds |
| GitHub PAT storage | Stored through Tauri store JSON | Stored through Stronghold |
| Agent execution | Routing exists, execution is mock-mode | Docs agent invokes real AI service |
| Release artifacts | No packaged desktop artifact | Build artifact attached to release |
| Release truthfulness | README may imply capabilities not fully wired | README reflects exact v1.0.1 behavior |

---

## 4. Architectural Positioning

v1.0.1 preserves the v0.5.1 / v1.0.0 architectural decisions.

### 4.1 Tauri In-Process Boundary Remains Primary

The desktop renderer continues to call Rust through Tauri `invoke()` commands. The pure core crates remain free of Tauri dependencies.

```text
React / TypeScript Renderer
  → Tauri invoke() commands
    → Tauri Rust command layer
      → Pure Rust core crates
```

The future sidecar/gRPC path remains valid but is not implemented in v1.0.1.

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

### 4.3 AI Must Not Block Git Commits

The semantic commit invariant remains unchanged:

```text
Git commit first
  → semantic stub immediately
    → AI backfill asynchronously
      → graph and embedding update later
```

v1.0.1 may improve visibility and reliability of this path, but it must not make AI inference a blocking dependency for standard Git commits.

---

## 5. Workstream A — Real Dashboard Data

### 5.1 Problem

The dashboard currently builds, but it uses hardcoded mock data. This undermines the product claim that the dashboard reflects the repository’s roadmap.

### 5.2 Required Behavior

The dashboard must consume a generated JSON artifact derived from real Markdown files in `roadmap/`.

Pipeline:

```text
roadmap/*.md
  → md-indexer
  → generated JSON artifact
  → dashboard static build
  → Cloudflare Pages deployment
```

### 5.3 Data Artifact

Create a generated file:

```text
apps/dashboard/public/orqestra-roadmap.json
```

Recommended shape:

```json
{
  "generated_at": "2026-06-01T00:00:00Z",
  "source": {
    "repo": "orqestra",
    "branch": "master",
    "commit": "<git-sha>"
  },
  "summary": {
    "total_tasks": 13,
    "done": 7,
    "backlog": 6,
    "in_progress": 0,
    "blocked": 0
  },
  "sprints": [
    {
      "id": "Sprint 16",
      "title": "Operational Hardening",
      "start_date": "2026-06-01",
      "end_date": "2026-06-14",
      "tasks": ["TASK-2026-060", "TASK-2026-061", "TASK-2026-062"]
    }
  ],
  "tasks": [
    {
      "id": "TASK-2026-061",
      "title": "Stronghold credentials",
      "status": "backlog",
      "priority": "High",
      "sprint": "Sprint 16",
      "epic": "Operational Hardening",
      "assignee": "agent-architect",
      "progress": 0,
      "start_date": null,
      "due_date": null,
      "dependencies": [],
      "blocks": [],
      "labels": ["security", "desktop"]
    }
  ]
}
```

### 5.4 Export Implementation Options

Preferred option:

```bash
orqestra roadmap export --format=json --out apps/dashboard/public/orqestra-roadmap.json
```

Fallback option:

```bash
cargo run -p md-indexer -- export \
  --roadmap roadmap \
  --out apps/dashboard/public/orqestra-roadmap.json
```

If the existing CLI only supports dependency graph output, extend it without breaking the existing command:

```bash
orqestra deps --format=dot
```

### 5.5 Dashboard Code Changes

Replace:

```text
apps/dashboard/src/lib/data.ts hardcoded data
```

With:

```text
fetch('/orqestra-roadmap.json')
```

Dashboard behavior:

- Show loading state while JSON loads.
- Show parse/load error state if the JSON is missing or malformed.
- Render Gantt and Kanban from the generated JSON.
- Display `generated_at` and source commit hash in the footer.

### 5.6 CI Workflow

Add or update:

```text
.github/workflows/dashboard.yml
```

Required steps:

```text
checkout
setup Rust
setup Node
cargo test -p md-indexer
npm ci
export roadmap JSON
npm run build -w apps/dashboard
upload dashboard artifact
optional: deploy to Cloudflare Pages when credentials exist
```

### 5.7 Acceptance Criteria

- `apps/dashboard/src/lib/data.ts` no longer contains primary mock roadmap data.
- `orqestra-roadmap.json` is generated from `roadmap/`.
- Dashboard renders real tasks from the repository.
- Dashboard build fails if JSON generation fails.
- Dashboard displays generation timestamp and source commit.
- Cloudflare deployment command succeeds when `CLOUDFLARE_API_TOKEN` is present.

---

## 6. Workstream B — Cloudflare Pages Deployment

### 6.1 Problem

The dashboard is buildable but not live. v1.0.1 must make it externally visible.

### 6.2 Required Behavior

The dashboard must be deployable with:

```bash
wrangler pages deploy apps/dashboard/dist
```

The repository must include clear deployment configuration.

### 6.3 Required Files

```text
apps/dashboard/wrangler.toml
.github/workflows/dashboard.yml
```

Example `wrangler.toml`:

```toml
name = "orqestra-dashboard"
pages_build_output_dir = "dist"
compatibility_date = "2026-06-01"
```

### 6.4 Deployment Modes

| Mode | Trigger | Behavior |
|---|---|---|
| Local manual | Developer runs Wrangler command | Deploys local `dist/` |
| CI preview | Pull request | Builds and uploads artifact; preview deploy optional |
| CI production | Tag or master push | Deploys to production if token exists |

### 6.5 Acceptance Criteria

- `npm run build -w apps/dashboard` succeeds.
- `wrangler pages deploy apps/dashboard/dist` succeeds when authenticated.
- README includes the live dashboard URL after first deployment.
- Dashboard displays real generated roadmap data after deployment.

---

## 7. Workstream C — Tauri Production Build

### 7.1 Problem

The desktop app has been verified in development mode, but no production Tauri bundle has been generated.

### 7.2 Required Command

```bash
cd apps/desktop
npm run tauri build
```

### 7.3 Required Behavior

The packaged desktop app must:

- Launch without a Vite dev server.
- Open the project picker.
- Index a selected repository’s `roadmap/` directory.
- Render the task table.
- Render Gantt and Kanban views.
- Preserve command error handling through stable JSON DTOs.
- Not expose internal Rust errors directly to TypeScript.

### 7.4 Common Failure Areas to Verify

| Area | Check |
|---|---|
| Asset paths | Built frontend assets load correctly inside Tauri |
| Permissions | Dialog plugin permissions are valid for Tauri 2 |
| Command registration | All invoked commands are registered in Rust |
| Path handling | Project paths work outside dev mode |
| Plugin config | Dialog/store/Stronghold plugins initialize correctly |
| Environment assumptions | No code depends on Vite-only environment variables |

### 7.5 Release Artifact

Attach the generated desktop bundle to the v1.0.1 GitHub release.

Expected artifact examples:

```text
macOS: .dmg or .app.tar.gz
Windows: .msi or .exe
Linux: .AppImage or .deb
```

At least one platform artifact is required for v1.0.1. Full cross-platform release automation may remain backlog.

### 7.6 Acceptance Criteria

- `npm run tauri build` succeeds on at least one target platform.
- The packaged app opens successfully.
- The app can index a real Orqestra repo after installation.
- Build instructions are documented in README.
- Release artifact is attached to the v1.0.1 release.

---

## 8. Workstream D — Stronghold Credential Storage

### 8.1 Problem

The desktop currently declares Stronghold but still stores GitHub PATs through Tauri store JSON. This is not acceptable for a release that claims secure credential handling.

### 8.2 Required Behavior

GitHub PATs must be stored, retrieved, and deleted through Stronghold-backed encrypted storage.

The app must not persist PATs in plaintext JSON.

### 8.3 Commands

Add Tauri commands:

```rust
save_github_token_cmd(project_root: String, token: String) -> Result<(), CommandError>
get_github_token_status_cmd(project_root: String) -> Result<TokenStatus, CommandError>
delete_github_token_cmd(project_root: String) -> Result<(), CommandError>
```

Do not expose a command that returns the raw token to TypeScript unless absolutely necessary. Prefer Rust-side Git operations that retrieve the token internally.

### 8.4 Token Status DTO

```json
{
  "exists": true,
  "provider": "stronghold",
  "label": "GitHub PAT",
  "last_updated": "2026-06-01T00:00:00Z"
}
```

### 8.5 Migration

If a PAT already exists in legacy Tauri store JSON:

1. Read the legacy value once.
2. Save it to Stronghold.
3. Verify Stronghold write success.
4. Delete the legacy value.
5. Emit a migration audit event.

If migration fails, preserve the legacy token and show a warning. Do not delete the only copy of a credential unless the Stronghold write is verified.

### 8.6 UI Changes

The Git sync panel must display credential state:

```text
No GitHub credential stored
Credential stored securely
Credential migration required
Credential migration failed
```

Required actions:

- Save token
- Replace token
- Delete token
- Test GitHub connection

### 8.7 Security Rules

- Never log the raw PAT.
- Never include the raw PAT in UI state snapshots.
- Never write the raw PAT to `.Orqestra/`.
- Never commit credential files.
- Mask any accidental token-like value in error messages.

### 8.8 Acceptance Criteria

- No PAT appears in Tauri store JSON after save or migration.
- Push/pull still works after app restart.
- Deleting credentials removes access.
- Credential status survives app restart.
- Logs do not include raw token values.

---

## 9. Workstream E — Real Docs-Agent Execution

### 9.1 Problem

The multi-agent UI and routing exist, but agent execution is still mock-mode. v1.0.1 must make one low-risk execution path real.

### 9.2 Selected Agent

Use the documentation agent first.

Rationale:

- Low blast radius
- Easy human review
- Output can be constrained to Markdown files
- Good test of task context, AI invocation, diff generation, and semantic commit flow

### 9.3 Required Flow

```text
User selects docs task
  → Agent router chooses docs workspace
  → Desktop gathers task context and relevant files
  → run_agent_cmd invokes real AI service
  → AI service returns proposed file edits
  → Desktop creates a reviewable diff
  → ConfidenceGate forces propose mode
  → User accepts or rejects
  → Accepted change creates normal Git commit
  → Semantic stub is written
  → AI backfill updates semantic metadata asynchronously
```

### 9.4 AI Service Contract

Add or finalize an endpoint:

```http
POST /agent/docs
```

Request:

```json
{
  "task": {
    "id": "TASK-2026-XXX",
    "title": "Update README",
    "body": "...",
    "labels": ["docs"]
  },
  "context_files": [
    {
      "path": "README.md",
      "content": "..."
    }
  ],
  "constraints": {
    "allowed_paths": ["README.md", "docs/**", "roadmap/**"],
    "max_files_changed": 3,
    "auto_commit": false
  }
}
```

Response:

```json
{
  "summary": "Updated README to document dashboard deployment.",
  "confidence": 0.82,
  "has_breaking_change": false,
  "edits": [
    {
      "path": "README.md",
      "before": "...",
      "after": "..."
    }
  ],
  "notes": [
    "No code files changed."
  ]
}
```

### 9.5 ConfidenceGate Policy

For v1.0.1, docs-agent output must not auto-commit.

Temporary config:

```yaml
confidence_gate:
  auto_commit: 1.01
  propose: 0.00
  flag: 0.00
  breaking_change_override: always_propose
```

This forces all successful docs-agent outputs into review mode.

### 9.6 File Scope

The docs agent may edit only:

```text
README.md
docs/**
roadmap/**
CHANGELOG.md
```

It must not edit:

```text
src/**
crates/**
apps/**
services/**
.github/**
Cargo.toml
package.json
pyproject.toml
```

### 9.7 Acceptance Criteria

- A docs-labeled task routes to the docs agent.
- The agent execution path calls the real AI service.
- The returned edit creates a visible diff.
- The user can accept or reject the proposed change.
- Accepted changes create a standard Git commit.
- Semantic stub creation still works.
- The UI clearly labels the action as human-approved, not autonomous.

---

## 10. Testing Requirements

### 10.1 Existing Tests Must Remain Green

Required baseline:

```bash
cargo test --workspace
npm run build -w apps/desktop
npm run build -w apps/dashboard
```

If existing E2E tests are available:

```bash
npm run test:e2e
```

### 10.2 New Tests

| Workstream | Test |
|---|---|
| Dashboard data | Exporter test verifies real roadmap files produce expected JSON |
| Dashboard UI | Dashboard renders generated JSON and shows generation timestamp |
| Desktop build | Production build command runs in CI or release workflow |
| Stronghold | Save/status/delete credential command tests or smoke tests |
| Agent execution | Docs-agent returns edit, diff is reviewable, no auto-commit occurs |
| Security | Token masking test for logs and errors |

### 10.3 Manual Smoke Test

Before tagging v1.0.1:

1. Clone repo fresh.
2. Run all Rust tests.
3. Build dashboard.
4. Generate roadmap JSON.
5. Run dashboard locally and verify real tasks appear.
6. Build Tauri production app.
7. Launch packaged app.
8. Open repo through project picker.
9. Save GitHub PAT.
10. Restart app and verify credential status.
11. Run docs-agent on a docs task.
12. Review and accept a Markdown diff.
13. Verify Git commit and semantic stub.
14. Deploy dashboard to Cloudflare Pages.
15. Update README with dashboard URL.

---

## 11. CI/CD Requirements

### 11.1 Dashboard Workflow

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
      - run: cargo run -p orqestra -- roadmap export --format=json --out apps/dashboard/public/orqestra-roadmap.json
      - run: npm run build -w apps/dashboard
      - uses: actions/upload-artifact@v4
        with:
          name: dashboard-dist
          path: apps/dashboard/dist
```

The exact CLI command may differ depending on the implemented package name. The workflow must be updated to match the real command before merge.

### 11.2 Desktop Release Workflow

A full cross-platform matrix is optional for v1.0.1. At minimum, document the local release command and verify one platform.

Future workflow:

```yaml
name: Desktop Release

on:
  push:
    tags:
      - "v*"

jobs:
  build-desktop:
    strategy:
      matrix:
        os: [macos-latest, windows-latest, ubuntu-latest]
```

For v1.0.1, one successful artifact is sufficient.

---

## 12. Documentation Updates

### 12.1 README

README must state the exact v1.0.1 status:

- What works locally
- What is deployed
- What remains backlog
- How to build desktop production bundle
- How to deploy dashboard
- How credentials are stored
- What agent execution mode is real
- What remains mock-mode

### 12.2 CHANGELOG

Add:

```markdown
## v1.0.1 — Truthful Release Candidate

### Added
- Real roadmap JSON export for dashboard
- Cloudflare Pages deployment path
- Production Tauri build instructions
- Stronghold-backed GitHub credential storage
- Real docs-agent execution path with human-reviewed diffs

### Changed
- Dashboard no longer uses hardcoded roadmap data as primary source
- Agent execution UI distinguishes mock, proposed, and human-approved actions

### Security
- Removed plaintext PAT persistence from Tauri store JSON
- Added token masking rules for logs and UI errors

### Known Limitations
- Full edge worker is still backlog
- Full gix migration is still backlog
- AST/tree-sitter analysis is still backlog
- ML-Master exploration remains incomplete
- Agents do not auto-commit code changes
```

### 12.3 Release Notes

The release notes must not overclaim autonomy. Suggested phrasing:

```text
v1.0.1 makes the v1.0.0 prototype externally demonstrable: the dashboard can now be generated from real roadmap data and deployed, the desktop app can be packaged, GitHub credentials are stored securely, and the documentation agent can produce real AI-generated diffs for human review.
```

---

## 13. Roadmap Task Updates

Create or update the following roadmap tasks.

### TASK-2026-066 — Dashboard Roadmap JSON Export

```yaml
---
pm-task: true
id: TASK-2026-066
title: "Generate dashboard data from real roadmap files"
type: Task
status: backlog
priority: Critical
sprint: "Sprint 16"
epic: "Operational Hardening"
assignee: "agent-architect"
labels:
  - dashboard
  - md-indexer
  - release
---
```

Acceptance criteria:

- CLI exports `roadmap/` to dashboard JSON.
- Dashboard consumes generated JSON.
- Mock data is removed or demoted to test fixtures.

### TASK-2026-067 — Deploy Dashboard to Cloudflare Pages

```yaml
---
pm-task: true
id: TASK-2026-067
title: "Deploy public dashboard to Cloudflare Pages"
type: Task
status: backlog
priority: High
sprint: "Sprint 16"
epic: "Operational Hardening"
assignee: "agent-devops"
labels:
  - dashboard
  - cloudflare
  - ci
---
```

Acceptance criteria:

- Wrangler deployment succeeds.
- Live dashboard URL is documented.
- Deployment uses generated roadmap JSON.

### TASK-2026-068 — Produce Tauri Production Build

```yaml
---
pm-task: true
id: TASK-2026-068
title: "Produce production Tauri desktop build"
type: Task
status: backlog
priority: Critical
sprint: "Sprint 16"
epic: "Operational Hardening"
assignee: "agent-desktop"
labels:
  - desktop
  - tauri
  - release
---
```

Acceptance criteria:

- `npm run tauri build` succeeds.
- Packaged app opens and indexes roadmap.
- Release artifact is attached to GitHub release.

### TASK-2026-069 — Wire Stronghold Credential Storage

```yaml
---
pm-task: true
id: TASK-2026-069
title: "Store GitHub PATs through Stronghold"
type: Task
status: backlog
priority: Critical
sprint: "Sprint 16"
epic: "Operational Hardening"
assignee: "agent-security"
labels:
  - security
  - credentials
  - desktop
---
```

Acceptance criteria:

- PATs are no longer persisted in plaintext JSON.
- Credential migration is safe.
- Push/pull works after restart.

### TASK-2026-070 — Implement Real Docs-Agent Execution

```yaml
---
pm-task: true
id: TASK-2026-070
title: "Replace docs-agent mock execution with real AI-service call"
type: Task
status: backlog
priority: High
sprint: "Sprint 16"
epic: "Operational Hardening"
assignee: "agent-docs"
labels:
  - agents
  - ai-service
  - docs
---
```

Acceptance criteria:

- Docs-agent calls the real AI service.
- Returned edits are shown as reviewable diffs.
- No autonomous commit occurs without human approval.

---

## 14. Release Checklist

### 14.1 Code

- [ ] Dashboard JSON export implemented
- [ ] Dashboard consumes generated JSON
- [ ] Dashboard mock data removed or isolated to tests
- [ ] Cloudflare Pages deployment verified
- [ ] Production Tauri build succeeds
- [ ] Stronghold token storage implemented
- [ ] Legacy token migration implemented
- [ ] Docs-agent real execution implemented
- [ ] Human-reviewed diff flow implemented
- [ ] README updated
- [ ] CHANGELOG updated

### 14.2 Tests

- [ ] `cargo test --workspace`
- [ ] `npm run build -w apps/desktop`
- [ ] `npm run build -w apps/dashboard`
- [ ] `npm run tauri build`
- [ ] Dashboard JSON export test
- [ ] Dashboard render test
- [ ] Credential smoke test
- [ ] Docs-agent review flow smoke test

### 14.3 Release

- [ ] Create tag `v1.0.1`
- [ ] Push tag
- [ ] Attach desktop artifact
- [ ] Publish release notes
- [ ] Confirm dashboard URL
- [ ] Verify README links

---

## 15. Success Definition

v1.0.1 is successful when an external reviewer can do the following without relying on mock claims:

1. Open the public dashboard and see real roadmap state.
2. Download or build the desktop app as a production bundle.
3. Open a repository in the packaged desktop app.
4. Store GitHub credentials without plaintext JSON persistence.
5. Run a docs-agent task that invokes the real AI service and produces a human-reviewable diff.
6. Verify that standard tests pass.
7. Read the README and see an accurate distinction between implemented features and backlog items.

The release is not successful if the dashboard remains hardcoded, the app remains dev-mode only, credentials remain in JSON, or the agent path still returns mock responses.

---

## 16. Post-v1.0.1 Backlog

After v1.0.1, continue with the existing Sprint 16–17 items in this recommended order:

1. Complete native `gix` Git operation migration.
2. Add tree-sitter AST code analysis.
3. Implement dependency vulnerability scanning.
4. Expand CI/CD integration.
5. Implement ML-Master exploration loop.
6. Build the Cloudflare edge worker.
7. Add Durable Object CRDT relay.
8. Expand real agent execution beyond docs tasks.
9. Introduce carefully gated autonomous commits for low-risk work.

---

## Appendix A — Suggested Branch Plan

```bash
git checkout master
git pull
git checkout -b hardening/v1.0.1
```

Suggested commit sequence:

```text
feat(dashboard): export roadmap JSON from md-indexer
feat(dashboard): consume generated roadmap data
ci(dashboard): build dashboard from generated roadmap artifact
build(desktop): fix production Tauri bundle
feat(security): store GitHub PATs in Stronghold
feat(agent): invoke real docs-agent AI endpoint
 docs(release): document v1.0.1 truthful release status
```

---

## Appendix B — Minimum Demo Script

A v1.0.1 demo should follow this order:

1. Show the repository `roadmap/` files.
2. Run the roadmap JSON export.
3. Open the dashboard locally and show the same tasks rendered.
4. Open the deployed Cloudflare Pages dashboard.
5. Launch the packaged desktop app.
6. Open the Orqestra repo.
7. Show secure GitHub credential status.
8. Select a docs task.
9. Run the docs agent.
10. Review the proposed diff.
11. Accept the change.
12. Show the resulting Git commit and semantic stub.

---

## Appendix C — Release Integrity Rule

Every public claim in README, release notes, or dashboard copy must be classified as one of:

```text
Implemented and verified
Implemented but local-only
Implemented but mock-mode
Scaffolded but not wired
Backlog
```

No feature may be described as complete unless it has a passing test, a working local demo, or a successful deployment artifact.

This rule is mandatory for v1.0.1 and should remain part of the release process afterward.

