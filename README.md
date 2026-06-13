# Orqestra

**Governed AI-native development beta.**

Orqestra is a local-first desktop application that turns a Git repository into a structured workspace with roadmap tracking, semantic history, AI-assisted code review, and an optional public dashboard — all running locally with bounded agent authority.

## Classification

Orqestra is a **governed AI-native development beta** for technical reviewers and early adopters. The current sealed release is **v2.14.11** with 963 passing tests across 4 suites (Rust, Worker, Dashboard, Python), governed file IO, truthful relay status, provider-agnostic AI configuration, and consent-gated external evidence intake.

> **Note:** The canonical truth source for release metadata is `release-manifest.json`. The dashboard evidence surface at [orqestra.pages.dev](https://orqestra.pages.dev) shows aggregate static data. No external beta session has occurred yet.

## Quick Start for Beta Reviewers

### 1. Download

Download the latest installer from [GitHub Releases](https://github.com/Octo-Lex/Orqestra/releases).

The installer is **unsigned**. Windows SmartScreen will warn you. Click "More info" → "Run anyway".

### 2. Verify SHA256

```powershell
Get-FileHash .\Orqestra_x64-setup.exe -Algorithm SHA256
```

Compare against `checksums.txt` or `release-manifest.json` attached to the release.

> The installer is unsigned. Windows SmartScreen will warn. Click "More info" → "Run anyway".

### 3. Verify Signature

```powershell
Get-AuthenticodeSignature .\Orqestra_x64-setup.exe
```

Expected: `Status: NotSigned` — the installer is unsigned because no code-signing certificate has been configured.

### 4. Install and Launch

Run the installer, then open Orqestra from the Start menu. The environment check panel appears with 10 readiness probes.

### 5. Run the No-Key Beta Demo

No API key needed. Click **"Try sample project"** in the onboarding wizard. For step-by-step instructions, see the **[Beta Quickstart](docs/beta-quickstart.md)**.

### 6. Export Diagnostics

If anything goes wrong, use the diagnostics export button. The bundle contains 13 diagnostic files with all secrets redacted. See **[Troubleshooting Guide](docs/troubleshooting.md)**.

---

## Windows SmartScreen

The Windows installer is unsigned. Windows SmartScreen warnings are expected.

Even when signing is implemented, SmartScreen may still warn for new or low-reputation downloads until reputation is established.

See [Signing Plan](docs/release-signing-plan.md) for the full path.

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows x64 | **release-tested** | NSIS installer, unsigned beta |
| Linux x64 | **smoke-tested** | Ubuntu 24.04 GNOME smoke pass; CI-built AppImage |
| macOS | **build-feasibility-verified** | CI compiles universal binary; no DMG/app bundle |

See [Platform Confidence](docs/platform-confidence.md) for what each status means and promotion criteria.

---

## Capability Matrix

| Feature | Status | Since | Notes |
|---------|--------|-------|-------|
| Roadmap parsing & indexing | **verified** | v1.0.0 | Markdown-native, self-hosted |
| Desktop PM views | **verified** | v1.0.0 | Table, Gantt, Kanban |
| Dashboard | **deployed** | v1.0.0 | CI auto-deployed on master push |
| OS keychain credentials | **verified** | v1.1.0 | Windows Credential Manager; Linux Secret Service; macOS Keychain |
| Docs agent | **verified**, review-only | v1.1.0 | Real AI service (`/agent/docs`); Agent Context v2 |
| Bugfix agent | **verified**, review-only, symbol-aware | v1.1.0 | Real AI service (`/agent/bugfix`); Agent Context v2 + symbols |
| Architect agent | **verified**, read-only planner | v1.9.0 | Real AI service (`/agent/architect`); produces plans only |
| Semantic commit preparation | **verified**, proposal-only | v1.3.0 | Deterministic heuristics, no AI dependency |
| Agent Context v2 | **verified** | v1.4.0 | Schema-versioned, content-free, review-only constraints |
| Safe diff context | **pilot** | v1.5.0 | Opt-in via `ORQESTRA_SAFE_DIFF_CONTEXT` env var |
| Git provider diagnostics | **verified** | v1.6.0 | Per-operation provider report (gix/gix-hybrid/CLI) |
| Patch governance | **verified** | v1.7.0 | Atomic writes, audit trail, forbidden path enforcement |
| Code intelligence (tree-sitter) | **verified** | v1.8.0 | Rust + TypeScript symbol extraction, bounded parsing |
| Native Git (read-only) | **verified** | v1.2.0 | Hybrid gix + CLI fallback |
| Native Git (commit) | **gix-hybrid** | v1.3.0 | Tree-from-index via CLI `git write-tree` |
| Native Git (push/pull/merge) | **not-implemented** | — | Backlog |
| Vector/embedding search | **implemented** | v1.0.0 | Python AI service, `all-MiniLM-L6-v2` + cosine similarity |
| Knowledge graph & triple store | **implemented** | v1.0.0 | Content-addressed triple store with commit indexer |
| CRDT sync (local) | **implemented** | v1.0.0 | Loro per-file document model, two-peer offline merge |
| Shockwave merge UI | **mock/prototype** | v1.0.0 | Uses fixture data, not real merge conflict resolution |
| First-run environment checks | **verified** | v2.0.0 | 10 non-mutating probes |
| Diagnostics bundle export | **verified** | v2.0.0 | 13 files, secret-redacted, non-mutating |

### Agent Matrix

| Agent | Mode | Endpoint | Writes | Patch-Governed |
|-------|------|----------|--------|---------------|
| docs-agent | review-only | `POST /agent/docs` | via governance | yes |
| bugfix-agent | review-only, symbol-aware | `POST /agent/bugfix` | via governance | yes |
| architect-agent | read-only planner | `POST /agent/architect` | **no** | no |
| autonomy | **disabled** | — | — | — |

Key safety properties:
- No agent can auto-commit
- AI-service failure does not fabricate plans
- Repository and `.Orqestra` runtime state remain protected
- Architect output has no patch-shaped fields (no `before`/`after`/`edits`)

### Git Provider Matrix

| Operation | Provider | Status |
|-----------|----------|--------|
| HEAD SHA read | gix (native) | Verified |
| Branch name read | gix (native) | Verified |
| Recent commit metadata | gix (native traversal) | Verified |
| Commit creation | gix-hybrid (tree-from-index via CLI) | Verified |
| Repository snapshot | gix hybrid (branch/HEAD via gix, counts via CLI) | Verified |
| Changed file summary | gix hybrid | Verified |
| Diff/stat | CLI fallback (`git diff --stat`) | Verified |
| Safe diff context extraction | CLI fallback (`git diff --unified=3`) | Verified, pilot |
| Staging | CLI fallback (`git add`) | Verified |
| Push/pull | **not-implemented** | Backlog |
| Merge/rebase | **not-implemented** | Backlog |

### Test Trend

| Release | Tests | Delta |
|---------|-------|-------|
| v1.0.12 | 141 | — |
| v1.1.0 | 151 | +10 |
| v1.1.1 | 165 | +14 |
| v1.2.0 | 194 | +29 |
| v1.2.1 | 215 | +21 |
| v1.3.0 | 240 | +25 |
| v1.3.1 | 269 | +29 |
| v1.4.0 | 287 | +18 |
| v1.4.1 | 305 | +18 |
| v1.5.0 | 328 | +23 |
| v1.5.1 | 328 | +0 |
| v1.6.0 | 345 | +17 |
| v1.7.0 | 376 | +31 |
| v1.8.0 | 410 | +34 |
| v1.9.0 | 427 | +17 |
| v2.0.0 | 447 | +20 |

---

## Known Limitations

- **Windows installer is unsigned** — SmartScreen warnings are expected; code-signing certificate not available
- **macOS** — CI builds a universal binary but no DMG/app bundle is published; no human smoke test
- **Linux AppImage** — CI-built; smoke-tested on Ubuntu 24.04 GNOME; AppImage naming has a CI versioning issue
- **Git write operations** — push, pull, merge, rebase not migrated to native providers
- **Cloudflare CRDT relay** — local CRDT works; relay not implemented
- **Code signing** — blocked, certificate not available
- **Safe diff context** — pilot, disabled by default
- **Shockwave merge** — mock/prototype, uses fixture data

---

## Documentation Doctrine

> Documentation is advisory. Code on disk is authoritative. Release claims must be backed by tests, artifacts, manifests, or verified implementation.
>
> If a document says X but the code does Y, the code is correct. File a docs issue.

---

## Report an Issue

- [Install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml)
- [AI mode issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=ai_mode_issue.yml)
- [Dashboard issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=dashboard_issue.yml)
- [Bug report](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=bug_report.yml)

**Do not paste API keys or secrets in issues.**

## Release Provenance

Each release includes `release-manifest.json` with: full Git SHAs, CI workflow run ID, artifact checksums, signing status, platform matrix, diagnostics links, and dashboard freshness.

## Documentation

| Document | Description |
|----------|-------------|
| [Beta Quickstart](docs/beta-quickstart.md) | Step-by-step reviewer guide |
| [Troubleshooting](docs/troubleshooting.md) | Common issues and fixes |
| [Installer Diagnostics](docs/installer-diagnostics.md) | Install failure diagnostic steps |
| [Platform Confidence](docs/platform-confidence.md) | What each platform status means |
| [Product Readiness](docs/product-readiness.md) | Capability maturity and verification |
| [Patch Governance](docs/patch-governance.md) | Agent patch application guard and audit trail |
| [Architect Agent](docs/architect-agent.md) | Read-only planner, no file writes |
| [Code Intelligence](docs/code-intelligence.md) | Tree-sitter symbol extraction |
| [Native Git](docs/native-git.md) | Hybrid gix + CLI provider details |
| [Semantic Commit Preparation](docs/semantic-commit-preparation.md) | Proposal-only deterministic commit pipeline |
| [Agent Context Quality](docs/agent-context-quality.md) | Agent Context v2 and content policy |
| [Safe Diff Context](docs/safe-diff-context.md) | Opt-in diff pilot for review-only agents |
| [Issue Triage](docs/beta-issue-triage.md) | How beta feedback is managed |
| [Signing Plan](docs/release-signing-plan.md) | Path to signed, notarized releases |
| [Release Artifacts](docs/RELEASE_ARTIFACTS.md) | Platform downloads and limitations |
| [Demo Script v2.0.0](docs/DEMO_SCRIPT_v2.0.0.md) | Deterministic beta demo walkthrough |

## Developer Setup

<details>
<summary>Build from source</summary>

```bash
git clone https://github.com/Elephant-Rock-Lab/Orqestra.git
cd Orqestra
cargo build --workspace
cd apps/desktop && npm ci && npm run build
cd apps/dashboard && npm ci && npm run build
cargo test --workspace
npx tsx scripts/validate-release-manifest.ts release-manifest.json
```

</details>

## License

Proprietary — Elephant Rock Lab.
