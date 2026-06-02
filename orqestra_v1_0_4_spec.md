# Orqestra — v1.0.4 Release Candidate Completion Specification

**Version:** 1.0.4  
**Date:** 2026-06-02  
**Status:** Draft — implementation-ready release-candidate completion  
**Release Theme:** Verified Beta Artifact  
**Base:** v1.0.3 tag at `e5833ce`, plus the post-v1.0.3 Cloudflare Pages deployment fix

---

## 1. Executive Summary

Orqestra v1.0.4 is a release-candidate completion pass. It does not introduce a new product phase and it does not expand the architectural surface area. The purpose of v1.0.4 is to make the implemented v1.0.3 user-ready beta credible as a downloadable, demoable, externally reviewable release.

v1.0.3 delivered the user-facing beta layer: onboarding, readiness checks, project validation, sample project generation, diagnostics export, recovery cards, user documentation, and a deterministic demo script. After v1.0.3, the Cloudflare dashboard deployment blocker was resolved: `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` are configured, the dashboard workflow was fixed with an explicit `accountId`, and both build and deploy jobs now pass.

The remaining problem is release integrity. The source tree is ahead of the shipped desktop binary, real AI demos still depend on `ZAI_API_KEY`, macOS/Linux artifacts are not yet clearly packaged or labeled, and unsigned artifact status must be explicit. v1.0.4 closes those gaps.

**Guiding principle:** v1.0.4 is successful only if a reviewer can download or build the current release artifact, open Orqestra, run the documented demo path, verify the live dashboard, and understand exactly which features are real, degraded, unsigned, or backlog.

---

## 2. Current Baseline

### 2.1 v1.0.3 Implemented State

The v1.0.3 codebase includes:

- 131 source files
- 17,011 total lines of code
- 48 Tauri commands
- 141 passing Rust tests
- 7/7 HTTP endpoint tests
- 10/10 CDP UI checks
- First-run onboarding wizard
- Environment readiness checks
- Project validation
- Sample project generation
- Diagnostics bundle export with redaction
- Recovery cards
- User guide, first-run guide, setup checks, diagnostics guide, release-artifact guide, and demo script

### 2.2 Resolved Since v1.0.3 Status Report

The Cloudflare dashboard deployment blocker is resolved.

- `CLOUDFLARE_API_TOKEN` is configured as a GitHub Actions repository secret.
- `CLOUDFLARE_ACCOUNT_ID` is configured as a GitHub Actions repository secret.
- The dashboard workflow includes an explicit `accountId` parameter for `wrangler-action`.
- The explicit `accountId` avoids the `/memberships` lookup path that failed with Cloudflare error `9106`.
- `build-dashboard` passes.
- `deploy-dashboard` passes.
- `orqestra.pages.dev` returns 200 OK and is freshly deployed.

### 2.3 Remaining Release Blockers

| Blocker | Current State | v1.0.4 Required State |
|---|---|---|
| Desktop artifact freshness | Existing Windows binary/NSIS installer are from v1.0.2 | Fresh v1.0.4 desktop artifact built from current source |
| Real AI demo | `ZAI_API_KEY` absent means mocks/expected auth failures | Real docs-agent and bugfix-agent demo path verified when key exists; degraded state clearly handled when absent |
| Dashboard deploy | Recently fixed manually | CI fix codified, documented, and protected by regression check |
| macOS/Linux bundles | CI/build support exists but local validation is limited | Artifacts either built and labeled, or explicitly marked unavailable/unverified |
| Unsigned binaries | Code signing not done | Unsigned beta status documented in release notes, UI docs, and artifact manifest |
| Demo integrity | Demo script exists | Demo script passes end-to-end against the release artifact, not only dev mode |

---

## 3. Release Goals

### 3.1 Primary Goals

| Goal | Description | Outcome |
|---|---|---|
| Fresh desktop release artifact | Rebuild desktop installer/binary from v1.0.4 source | Artifact matches source and tag |
| Real AI readiness | Make real AI path demoable when `ZAI_API_KEY` is set | Docs/bugfix agents can be demonstrated without mock ambiguity |
| Cloudflare deployment permanence | Preserve and document the fixed dashboard CI path | Dashboard stays auto-deployable from CI |
| Artifact labeling | Generate clear platform support metadata | Reviewers know what is tested, untested, unsigned, or unavailable |
| Demo-gate release | Run deterministic demo script against release artifact | Release evidence is reproducible |
| Documentation truthfulness | Update README, CHANGELOG, release notes, and artifact guide | No overclaims about signing, AI, agents, or platform coverage |

### 3.2 Non-Goals

The following remain out of scope for v1.0.4 unless required to unblock the primary goals:

- Full native `gix` migration for every remaining Git shell-out
- AST/tree-sitter code analysis
- Dependency vulnerability scanning
- Real architect-agent implementation
- Full ML-Master exploration loop
- Cloudflare Durable Object CRDT relay
- Edge Worker semantic query API
- Code signing and notarization
- Autonomous agent commits
- Hosted SaaS functionality

---

## 4. Architectural Positioning

v1.0.4 preserves all previous architecture decisions.

### 4.1 Tauri In-Process Boundary Remains Primary

The renderer continues to call Rust through Tauri `invoke()` commands. Rust core crates remain independent of Tauri. The Tauri command layer owns desktop integration.

```text
React / TypeScript renderer
  → Tauri invoke() command layer
    → Pure Rust core crates
      → Python AI service / Git / graph store / CRDT engine
```

### 4.2 AI Remains Review-Only for Agents

Docs-agent and bugfix-agent output remains review-only. A successful AI call may produce a proposed diff, but no agent may auto-commit without human approval.

### 4.3 Dashboard Is CI-Deployed, Not Manually Staged

The dashboard must be generated from roadmap JSON during CI and deployed through Cloudflare Pages using the repository secrets:

```text
CLOUDFLARE_API_TOKEN
CLOUDFLARE_ACCOUNT_ID
```

The workflow must pass `accountId` explicitly to avoid account discovery calls.

---

## 5. Workstream A — Fresh Desktop Release Artifact

### 5.1 Problem

The v1.0.3 source tree is ahead of the current desktop binary. The existing Windows binary and NSIS installer were produced from the v1.0.2 build. This creates a release-integrity problem: the downloadable artifact does not prove the current source state.

### 5.2 Required Behavior

v1.0.4 must produce at least one fresh desktop artifact from the v1.0.4 source tree.

Required command:

```bash
cd apps/desktop
npm run tauri build
```

or, from repo root if workspace scripts support it:

```bash
npm run tauri build -w apps/desktop
```

The resulting artifact must be attached to the v1.0.4 GitHub release and referenced in release notes.

### 5.3 Artifact Metadata

Create or update:

```text
dist/release-manifest.json
```

Recommended shape:

```json
{
  "version": "1.0.4",
  "tag": "v1.0.4",
  "commit": "<git-sha>",
  "built_at": "2026-06-02T00:00:00Z",
  "artifacts": [
    {
      "platform": "windows-x64",
      "kind": "nsis-installer",
      "path": "target/release/bundle/nsis/Orqestra_1.0.4_x64-setup.exe",
      "status": "tested",
      "signed": false,
      "sha256": "<sha256>"
    }
  ],
  "warnings": [
    "Artifacts are unsigned beta builds."
  ]
}
```

### 5.4 Acceptance Criteria

- A fresh v1.0.4 desktop artifact is built from the v1.0.4 source tree.
- The artifact opens without a dev server.
- The artifact shows the v1.0.3/v1.0.4 onboarding and readiness UI.
- The artifact can create or open the sample project.
- The artifact can show readiness state.
- The artifact manifest includes version, commit, build time, platform, signing status, and checksum.
- Release notes do not point to a stale v1.0.2 binary.

---

## 6. Workstream B — Real AI Demo Readiness

### 6.1 Problem

The AI service endpoints are implemented, but real docs-agent and bugfix-agent demos require `ZAI_API_KEY`. Without the key, the system correctly degrades to mock or expected-auth behavior, but the release demo must distinguish real AI mode from degraded mode.

### 6.2 Required Behavior

When `ZAI_API_KEY` is set:

- `/health` passes.
- `/agent/docs` returns a real model-backed proposed edit.
- `/agent/bugfix` returns a real model-backed proposed edit within user-selected file scope.
- The UI labels outputs as AI-generated proposals requiring human review.
- ConfidenceGate remains propose/review-only.
- No automatic commit occurs.

When `ZAI_API_KEY` is absent:

- Readiness shows AI as degraded or unavailable.
- The demo script clearly marks AI-dependent steps as skipped or degraded.
- The UI must not claim real AI execution occurred.

### 6.3 Readiness DTO Requirements

Extend or verify the readiness report shape includes:

```typescript
export interface AiReadinessStatus {
  available: boolean;
  mode: 'real-ai' | 'degraded' | 'mock' | 'unavailable';
  provider: 'zai' | 'none';
  model: string | null;
  requires_env: 'ZAI_API_KEY' | null;
  message: string;
}
```

The DTO must not contain raw API keys or token-like values.

### 6.4 AI Demo Fixture

Add a deterministic demo fixture under:

```text
demo/ai-fixtures/
```

Recommended files:

```text
demo/ai-fixtures/docs-task.md
demo/ai-fixtures/bugfix-task.md
demo/ai-fixtures/README.before.md
demo/ai-fixtures/sample_bug.before.ts
```

The fixture must be small enough to run quickly and predictable enough to verify diff generation.

### 6.5 Acceptance Criteria

- `ZAI_API_KEY` is detected by readiness checks without exposing the key.
- Docs-agent real AI path is demoable when the key is present.
- Bugfix-agent real AI path is demoable when the key is present.
- Both agent paths remain review-only.
- Demo script has two paths: real-AI and degraded/no-key.
- CI or local smoke tests verify the no-key degraded path.
- Manual release checklist verifies the real-AI path when the key is available.

---

## 7. Workstream C — Cloudflare Dashboard Deployment Regression Lock

### 7.1 Problem

Dashboard deployment is now fixed, but the fix must be preserved in the repository. The specific issue was that the workflow attempted or triggered account discovery through Cloudflare membership lookup, which failed with error `9106`. Passing `accountId` explicitly resolved the deployment.

### 7.2 Required Behavior

The dashboard workflow must:

- Build roadmap JSON from the repository.
- Build the dashboard.
- Deploy to Cloudflare Pages on the configured branch/tag trigger.
- Pass `accountId` explicitly to the Cloudflare action.
- Use `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` from GitHub Secrets.
- Avoid direct `secrets.* != ''` conditions in `if:` expressions.

### 7.3 Workflow Pattern

Use this structure or equivalent:

```yaml
- name: Deploy dashboard to Cloudflare Pages
  uses: cloudflare/wrangler-action@v3
  with:
    apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
    accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
    command: pages deploy apps/dashboard/dist --project-name orqestra
```

If the project name differs, use the actual Cloudflare Pages project name.

### 7.4 Required Documentation Note

Add to release documentation:

```markdown
The dashboard deployment workflow passes Cloudflare `accountId` explicitly. This avoids relying on account discovery through the Cloudflare memberships API and prevents deployment failures when the API token is scoped only for Pages deployment.
```

### 7.5 Acceptance Criteria

- `build-dashboard` passes.
- `deploy-dashboard` passes.
- `orqestra.pages.dev` returns HTTP 200 after CI deployment.
- Dashboard footer/source metadata reflects the latest deployed commit.
- The release notes classify dashboard deployment as implemented and verified.

---

## 8. Workstream D — macOS/Linux Artifact Labeling and Bundler Targets

### 8.1 Problem

The project has release workflow support, but macOS/Linux artifact coverage remains limited. v1.0.4 must not imply full cross-platform support unless artifacts are actually produced and checked.

### 8.2 Required Behavior

Each platform must be classified as one of:

```text
tested
built-but-unverified
not-built
blocked
```

The classification must appear in:

- `dist/release-manifest.json`
- GitHub release notes
- `docs/RELEASE_ARTIFACTS.md`
- README release-status section

### 8.3 macOS Universal Build Rule

If macOS artifact generation is attempted, prefer a universal artifact:

```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
npm run tauri build -w apps/desktop -- --target universal-apple-darwin
```

If universal packaging fails, produce separate clearly labeled artifacts:

```text
macos-arm64 — built-but-unverified
macos-x64 — built-but-unverified or not-built
```

An unlabeled arm64-only artifact does not satisfy the v1.0.4 artifact labeling requirement.

### 8.4 Linux Build Rule

If Linux artifact generation is attempted, explicitly list which target was built:

```text
linux-x64.AppImage
linux-x64.deb
```

If local validation is not performed, mark the artifact `built-but-unverified`.

### 8.5 Acceptance Criteria

- Windows artifact is rebuilt and marked `tested` if smoke-tested.
- macOS artifact is either built and clearly labeled or explicitly marked `not-built`.
- Linux artifact is either built and clearly labeled or explicitly marked `not-built`.
- README does not imply support for unbuilt platforms.
- Release notes include unsigned beta warning.

---

## 9. Workstream E — Unsigned Beta Disclosure

### 9.1 Problem

Code signing and notarization are not done. This is acceptable for a beta, but it must be disclosed clearly to avoid a trust gap.

### 9.2 Required Behavior

Every release surface must state:

```text
Orqestra v1.0.4 desktop artifacts are unsigned beta builds. Your operating system may show a warning before launch.
```

Required locations:

- GitHub release notes
- `docs/RELEASE_ARTIFACTS.md`
- README install section
- Release manifest warnings array

### 9.3 Non-Goal

v1.0.4 does not require:

- Apple Developer ID signing
- macOS notarization
- Windows Authenticode certificate
- Linux repository signing

Those belong to a later production-readiness release.

### 9.4 Acceptance Criteria

- Unsigned status appears in all required docs.
- Release artifact manifest includes `signed: false` for unsigned artifacts.
- No release copy calls the binaries production-signed or notarized.

---

## 10. Workstream F — Demo-Gated Release Verification

### 10.1 Problem

v1.0.3 includes a deterministic demo script, but v1.0.4 must ensure the demo runs against the packaged artifact and the freshly deployed dashboard, not only against dev mode.

### 10.2 Required Demo Modes

#### Mode A — No-key beta demo

This is the default reviewer path.

Required steps:

1. Install or launch the v1.0.4 desktop artifact.
2. Complete first-run onboarding.
3. Create sample project.
4. Open readiness panel.
5. Verify AI shows degraded/no-key state.
6. Open dashboard at `orqestra.pages.dev`.
7. Export diagnostics bundle.
8. Confirm diagnostics bundle contains no secrets.

#### Mode B — Real-AI maintainer demo

This path requires `ZAI_API_KEY`.

Required steps:

1. Start AI service with `ZAI_API_KEY`.
2. Open sample project.
3. Run docs-agent on a docs task.
4. Review proposed diff.
5. Reject or accept manually.
6. Run bugfix-agent with user-selected files.
7. Review proposed diff.
8. Confirm no autonomous commit occurred.

### 10.3 Demo Evidence

Create:

```text
docs/DEMO_EVIDENCE_v1.0.4.md
```

Minimum contents:

```markdown
# v1.0.4 Demo Evidence

- Tag: v1.0.4
- Commit: <sha>
- Artifact: <name>
- Artifact SHA256: <sha256>
- Dashboard URL: https://orqestra.pages.dev
- Dashboard status: 200 OK
- Demo mode: no-key beta / real-AI maintainer
- Result: pass/fail
- Notes:
```

### 10.4 Acceptance Criteria

- No-key beta demo passes from packaged artifact.
- Dashboard is verified live during demo.
- Real-AI maintainer demo is either passed or explicitly marked unavailable because `ZAI_API_KEY` is absent.
- Demo evidence file is committed or attached to the release.

---

## 11. Testing Requirements

### 11.1 Existing Tests Must Remain Green

Required:

```bash
cargo test --workspace
npm run build -w apps/desktop
npm run build -w apps/dashboard
```

Required service checks:

```bash
python .Orqestra/test_http_endpoints.py
python .Orqestra/test_cdp_ui.py
```

### 11.2 New or Updated Tests

| Workstream | Test |
|---|---|
| Desktop artifact | Packaged app opens and shows onboarding/readiness |
| Release manifest | Manifest contains version, commit, artifact status, checksum, signing status |
| AI readiness | No-key mode is degraded and does not expose secrets |
| Real AI path | Manual smoke test with `ZAI_API_KEY` when available |
| Dashboard deploy | Workflow uses explicit `accountId` and deploy job passes |
| Redaction | Diagnostics bundle redacts API keys, PATs, tokens, secret-like values |
| Artifact labeling | macOS/Linux statuses are explicit and not overclaimed |

### 11.3 Manual Smoke Test

Before tagging v1.0.4:

1. Pull clean `master`.
2. Run `cargo test --workspace`.
3. Build desktop frontend.
4. Build dashboard frontend.
5. Run HTTP tests.
6. Run CDP UI tests.
7. Build v1.0.4 desktop artifact.
8. Generate release manifest with checksums.
9. Launch packaged artifact.
10. Complete onboarding.
11. Create sample project.
12. Verify readiness panel.
13. Export diagnostics bundle.
14. Confirm no secrets in diagnostics bundle.
15. Verify `orqestra.pages.dev` returns 200.
16. Run real-AI demo if `ZAI_API_KEY` is available.
17. Update release notes and README.
18. Tag `v1.0.4`.
19. Attach artifacts and manifest.

---

## 12. CI/CD Requirements

### 12.1 Dashboard Workflow

The dashboard workflow is now release-critical. It must preserve the explicit Cloudflare `accountId` fix.

Required jobs:

```text
build-dashboard
  → checkout
  → setup Rust
  → setup Node
  → cargo test -p md-indexer
  → npm ci
  → export roadmap JSON
  → npm run build -w apps/dashboard
  → upload dashboard artifact

deploy-dashboard
  → depends on build-dashboard
  → download dashboard artifact
  → wrangler pages deploy with apiToken + accountId
```

### 12.2 Desktop Release Workflow

The desktop release workflow should:

- Build at least Windows x64.
- Generate checksum.
- Generate release manifest.
- Upload artifact.
- Upload manifest.

Optional matrix:

```yaml
strategy:
  matrix:
    include:
      - os: windows-latest
        platform: windows-x64
      - os: macos-latest
        platform: macos-universal
      - os: ubuntu-latest
        platform: linux-x64
```

If macOS/Linux are not validated, mark them correctly in the manifest.

---

## 13. Documentation Updates

### 13.1 README

README must include:

- v1.0.4 current release status
- Dashboard live URL
- Dashboard CI deployment status
- Explicit unsigned beta warning
- Which desktop artifacts are available
- How to run with no AI key
- How to enable real AI with `ZAI_API_KEY`
- What remains backlog

### 13.2 CHANGELOG

Add:

```markdown
## [1.0.4] - 2026-06-02

### Added
- Fresh release manifest with artifact checksums, platform labels, and signing status
- Demo evidence file for packaged-artifact verification
- Real-AI demo path documentation for docs-agent and bugfix-agent

### Changed
- Dashboard deployment workflow now uses explicit Cloudflare `accountId`
- Release notes now distinguish tested, built-but-unverified, not-built, and unsigned artifacts
- Demo script now includes no-key beta and real-AI maintainer modes

### Fixed
- Dashboard CI deployment no longer relies on Cloudflare account discovery through memberships lookup
- v1.0.4 release artifacts are rebuilt from current source instead of reusing v1.0.2 binaries

### Security
- Diagnostics redaction remains enforced for exported bundles
- Release documentation states that desktop binaries are unsigned beta artifacts

### Known Limitations
- Code signing and notarization are not yet done
- Full native gix migration remains incomplete
- Architect agent remains mock-mode
- ML-Master exploration remains stub
- Edge relay is still backlog
```

### 13.3 Release Notes

Suggested release note summary:

```text
v1.0.4 completes the user-ready beta release by rebuilding desktop artifacts from current source, preserving the fixed Cloudflare Pages CI deployment, documenting real-AI and no-key demo modes, and publishing explicit artifact labels for platform support and unsigned beta status.
```

---

## 14. Roadmap Task Updates

Create or update the following roadmap tasks.

### TASK-2026-082 — Rebuild v1.0.4 Desktop Artifact

```yaml
---
pm-task: true
id: TASK-2026-082
title: "Rebuild desktop artifact from v1.0.4 source"
type: Task
status: backlog
priority: Critical
sprint: "Sprint 18"
epic: "Release Candidate Completion"
assignee: "agent-desktop"
labels:
  - desktop
  - release
  - tauri
---
```

Acceptance criteria:

- Fresh v1.0.4 desktop artifact is built.
- Artifact opens without dev server.
- Artifact shows onboarding/readiness UI.
- Artifact checksum is recorded in release manifest.

### TASK-2026-083 — Verify Real-AI Demo Path

```yaml
---
pm-task: true
id: TASK-2026-083
title: "Verify real-AI docs and bugfix agent demo path"
type: Task
status: backlog
priority: High
sprint: "Sprint 18"
epic: "Release Candidate Completion"
assignee: "agent-ai"
labels:
  - ai-service
  - agents
  - demo
---
```

Acceptance criteria:

- Readiness detects `ZAI_API_KEY` without exposing it.
- Docs-agent returns real AI proposed diff when key is present.
- Bugfix-agent returns real AI proposed diff within selected file scope.
- No-key degraded path is documented and tested.

### TASK-2026-084 — Lock Cloudflare Dashboard Deployment Workflow

```yaml
---
pm-task: true
id: TASK-2026-084
title: "Lock Cloudflare dashboard deployment workflow"
type: Task
status: backlog
priority: High
sprint: "Sprint 18"
epic: "Release Candidate Completion"
assignee: "agent-devops"
labels:
  - dashboard
  - cloudflare
  - ci
---
```

Acceptance criteria:

- `build-dashboard` passes.
- `deploy-dashboard` passes.
- Workflow uses explicit `accountId`.
- Release docs mention why `accountId` is required.

### TASK-2026-085 — Publish Artifact Labels and Unsigned Beta Notice

```yaml
---
pm-task: true
id: TASK-2026-085
title: "Publish artifact labels and unsigned beta notice"
type: Task
status: backlog
priority: High
sprint: "Sprint 18"
epic: "Release Candidate Completion"
assignee: "agent-release"
labels:
  - release
  - docs
  - artifacts
---
```

Acceptance criteria:

- Release manifest classifies each platform.
- Unsigned beta warning appears in README, release notes, and artifact docs.
- macOS/Linux artifacts are not overclaimed.

### TASK-2026-086 — Run Packaged Demo Gate

```yaml
---
pm-task: true
id: TASK-2026-086
title: "Run packaged demo gate for v1.0.4"
type: Task
status: backlog
priority: Critical
sprint: "Sprint 18"
epic: "Release Candidate Completion"
assignee: "agent-qa"
labels:
  - qa
  - demo
  - release
---
```

Acceptance criteria:

- No-key beta demo passes using packaged artifact.
- Dashboard is verified live.
- Diagnostics export is verified redacted.
- Demo evidence file is produced.

---

## 15. Release Checklist

### 15.1 Code

- [ ] Fresh v1.0.4 desktop artifact built
- [ ] Release manifest generated
- [ ] Artifact checksum generated
- [ ] Dashboard workflow uses explicit `accountId`
- [ ] AI readiness reports real/degraded mode accurately
- [ ] Demo fixture added or verified
- [ ] Documentation updated

### 15.2 Tests

- [ ] `cargo test --workspace`
- [ ] `npm run build -w apps/desktop`
- [ ] `npm run build -w apps/dashboard`
- [ ] HTTP endpoint tests
- [ ] CDP UI tests
- [ ] Packaged artifact smoke test
- [ ] Dashboard live 200 OK check
- [ ] Diagnostics redaction check

### 15.3 Release

- [ ] Create tag `v1.0.4`
- [ ] Push tag
- [ ] Attach desktop artifact
- [ ] Attach release manifest
- [ ] Attach demo evidence
- [ ] Publish release notes
- [ ] Verify dashboard URL
- [ ] Verify README links

---

## 16. Success Definition

v1.0.4 is successful when an external reviewer can:

1. Open the GitHub release.
2. Download a current desktop artifact built from the v1.0.4 source tree.
3. See a checksum and artifact status label.
4. Understand that binaries are unsigned beta builds.
5. Launch the packaged app without a dev server.
6. Complete onboarding.
7. Create or open a sample Orqestra project.
8. View readiness state.
9. Export a redacted diagnostics bundle.
10. Open `orqestra.pages.dev` and see the freshly deployed dashboard.
11. Understand whether AI is running in real or degraded mode.
12. Follow the demo script without relying on source-code knowledge.

The release is not successful if the desktop artifact is stale, dashboard CI silently fails, AI mode is ambiguous, platform artifact support is overclaimed, or unsigned beta status is hidden.

---

## 17. Post-v1.0.4 Backlog

After v1.0.4, continue with the trust-critical backend backlog in this recommended order:

1. Complete full native `gix` migration for remaining Git shell-outs.
2. Add AST/tree-sitter code analysis.
3. Implement dependency vulnerability scanning.
4. Implement real architect-agent execution.
5. Implement ML-Master exploration loop.
6. Build Cloudflare Worker + Durable Object CRDT relay.
7. Add code signing and notarization for production releases.

---

## Appendix A — Suggested Branch Plan

```bash
git checkout master
git pull
git checkout -b release/v1.0.4-rc-completion
```

Suggested commit sequence:

```text
build(desktop): rebuild v1.0.4 desktop artifact
ci(dashboard): lock cloudflare pages deploy account id
feat(release): generate artifact manifest with checksums
test(demo): add packaged artifact demo evidence
docs(release): document unsigned beta artifacts and AI modes
docs(changelog): add v1.0.4 release notes
```

---

## Appendix B — Minimum v1.0.4 Demo Script

1. Show GitHub release page.
2. Show artifact manifest and checksum.
3. Launch packaged desktop app.
4. Complete onboarding.
5. Create sample project.
6. Open readiness panel.
7. Show AI degraded state if no `ZAI_API_KEY`, or real-AI state if present.
8. Open dashboard at `https://orqestra.pages.dev`.
9. Show dashboard 200 OK and current roadmap data.
10. Export diagnostics bundle.
11. Open diagnostics bundle and show secrets are redacted.
12. If `ZAI_API_KEY` is present, run docs-agent proposal.
13. If `ZAI_API_KEY` is present, run bugfix-agent proposal with user-selected files.
14. Confirm all AI edits are review-only.
15. Show release notes and known limitations.

---

## Appendix C — Release Integrity Rule

Every public claim in README, release notes, dashboard copy, artifact guide, and demo script must be classified as one of:

```text
Implemented and verified
Implemented but local-only
Implemented but degraded without credentials
Implemented but unsigned beta
Built but unverified
Mock-mode
Backlog
```

No feature may be described as complete unless it has at least one of:

- Passing automated test
- Working local demo
- Successful CI deployment
- Packaged-artifact smoke test
- Attached release artifact evidence

This rule is mandatory for v1.0.4 and should remain part of every release afterward.
