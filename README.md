# Orqestra

**Local-first, AI-native project management for Git repositories.**

Orqestra turns a Git repository into a structured workspace with roadmap tracking, semantic history, AI-assisted code review, and an optional public dashboard — all running locally.

## Public Beta Status

Orqestra is a **public beta** for technical reviewers and early adopters. The current release is **v1.5.0** with 328 passing tests, review-only agents, and an opt-in safe diff context pilot.

## Quick Start for Public Beta Reviewers

### 1. Download

Download `Orqestra_1.5.0_x64-setup.exe` from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases).

The installer is **unsigned**. Windows SmartScreen will warn you. Click "More info" → "Run anyway".

### 2. Verify SHA256

```powershell
Get-FileHash .\Orqestra_1.5.0_x64-setup.exe -Algorithm SHA256
```

Compare against `checksums.txt` or `release-manifest.json` attached to the release.

### 3. Verify Signature

```powershell
Get-AuthenticodeSignature .\Orqestra_1.5.0_x64-setup.exe
```

Expected: `Status: NotSigned` — the installer is unsigned because no code-signing certificate has been configured.

### 4. Install and Launch

Run the installer, then open Orqestra from the Start menu. The onboarding wizard appears.

### 5. Run the No-Key Beta Demo

No API key needed. Click **"Try sample project"** in the onboarding wizard. For step-by-step instructions, see the **[Beta Quickstart](docs/beta-quickstart.md)**.

### Troubleshooting

If anything goes wrong, see **[Troubleshooting Guide](docs/troubleshooting.md)** or **[Installer Diagnostics](docs/installer-diagnostics.md)**.

---

## Windows SmartScreen

The Windows installer is unsigned. Windows SmartScreen warnings are expected.

Even when signing is implemented, SmartScreen may still warn for new or low-reputation downloads until reputation is established.

See [Signing Plan](docs/release-signing-plan.md) for the full path.

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows x64 | **release-tested** | NSIS installer, unsigned beta |
| Linux x64 | **smoke-tested** | Ubuntu 24.04 GNOME smoke pass (v1.0.12); CI-built AppImage |
| macOS | **build-feasibility-verified** | CI compiles universal binary; no DMG/app bundle |

See [Platform Confidence](docs/platform-confidence.md) for what each status means and promotion criteria.

---

## Capability Matrix

| Feature | Status | Notes |
|---------|--------|-------|
| Roadmap parsing & indexing | Implemented and verified | Markdown-native, self-hosted |
| Desktop PM views | Implemented and verified | Table, Gantt, Kanban |
| Dashboard | Deployed at [orqestra.pages.dev](https://orqestra.pages.dev) | CI auto-deployed on master push |
| OS keychain credentials | Implemented and verified | Windows Credential Manager; Linux Secret Service; macOS Keychain |
| Docs agent | Implemented, review-only | Real AI service (`/agent/docs`); Agent Context v2 |
| Bugfix agent | Implemented, review-only | Real AI service (`/agent/bugfix`); Agent Context v2 |
| Semantic commit preparation | Implemented, proposal-only | Deterministic heuristics, no AI dependency; 1341 LOC |
| Native Git operations | Implemented, read-only scope | Hybrid gix + CLI fallback (see provider matrix below) |
| Agent Context v2 | Implemented and verified | Schema-versioned, content-free, review-only constraints enforced |
| Safe diff context pilot | Implemented pilot, disabled by default | Opt-in via `ORQESTRA_SAFE_DIFF_CONTEXT` env var |
| Knowledge graph & triple store | Implemented | Content-addressed triple store with commit indexer |
| CRDT sync (local) | Implemented | Loro per-file document model, two-peer offline merge verified |
| Shockwave merge UI | **Mock/prototype** | Uses fixture data, not real merge conflict resolution |
| Vector/embedding search | Implemented in Python AI service | `all-MiniLM-L6-v2` embeddings + cosine similarity via `/query_history` endpoint |
| First-run onboarding | Implemented and verified | Guided wizard with sample project |
| Diagnostics export | Implemented and verified | Redacted bundle, no raw secrets |
| Release manifest | Implemented and verified | Provenance, signing, diagnostics, platform fields |
| Issue templates | Implemented | Install, AI mode, dashboard, bug report |

### Agent Matrix

| Agent | Status | Mode | Endpoint |
|-------|--------|------|----------|
| docs-agent | **Real, review-only** | Agent Context v2 + safe diff context (pilot) | `POST /agent/docs` |
| bugfix-agent | **Real, review-only** | Agent Context v2 + safe diff context (pilot) | `POST /agent/bugfix` |
| architect-agent | **Not implemented** | — | — |
| Autonomy | **Disabled** | `auto_commit: false`, `auto_apply: false`, `autonomous_actions: false` | — |

### Git Provider Matrix

| Operation | Provider | Status |
|-----------|----------|--------|
| HEAD SHA read | gix (native) | Verified |
| Branch name read | gix (native) | Verified |
| Recent commit metadata | gix (native traversal) | Verified |
| Commit creation | gix (native, tree-from-index via CLI) | Verified |
| Repository snapshot | gix hybrid (branch/HEAD via gix, counts via CLI) | Verified |
| Changed file summary | gix hybrid | Verified |
| Diff/stat | CLI fallback (`git diff --stat`) | Verified |
| Safe diff context extraction | CLI fallback (`git diff --unified=3`) | Verified, pilot |
| Staging | CLI fallback (`git add`) | Verified |
| Push/pull | **Not implemented in git-bridge** | Backlog |
| Merge/rebase | **Not implemented in git-bridge** | Backlog |

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

---

## Known Limitations

- **Windows installer is unsigned** — SmartScreen warnings are expected; code-signing certificate not available
- **macOS** — CI builds a universal binary but no DMG/app bundle is published
- **Linux AppImage** — CI-built; smoke-tested on Ubuntu 24.04 GNOME; AppImage naming has a CI versioning issue
- **Architect agent** — not implemented
- **ML-Master exploration** — stub
- **Edge relay / CRDT sync** — local CRDT works; Cloudflare Durable Object relay not implemented
- **Git write operations** — push, pull, merge, rebase not migrated to native providers; remain CLI-only or not implemented
- **Code signing** — blocked, certificate not available
- **Safe diff context** — pilot, disabled by default, CLI-backed provider only
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
| [Native Git](docs/native-git.md) | Hybrid gix + CLI provider details |
| [Semantic Commit Preparation](docs/semantic-commit-preparation.md) | Proposal-only deterministic commit pipeline |
| [Agent Context Quality](docs/agent-context-quality.md) | Agent Context v2 and content policy |
| [Safe Diff Context](docs/safe-diff-context.md) | Opt-in diff pilot for review-only agents |
| [Issue Triage](docs/beta-issue-triage.md) | How beta feedback is managed |
| [Signing Plan](docs/release-signing-plan.md) | Path to signed, notarized releases |
| [Release Artifacts](docs/RELEASE_ARTIFACTS.md) | Platform downloads and limitations |

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
