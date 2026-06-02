# Orqestra

**Local-first, AI-native project management for Git repositories.**

Orqestra turns a Git repository into a structured workspace with roadmap tracking, semantic history, AI-assisted code review, and an optional public dashboard — all running locally.

## Public Beta Status

Orqestra v1.0.5 is a **public beta** for technical reviewers and early adopters. It includes a tested Windows x64 installer, a live dashboard, roadmap indexing, semantic commit infrastructure, and real-AI review flows. It is not yet a production product. The installer is unsigned, macOS artifacts are not yet provided, Linux is not yet verified, and some advanced agent paths remain review-only or scaffolded.

## Download and Verify

### Download

Download `Orqestra_1.0.5_x64-setup.exe` from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases).

### Verify the Installer

```powershell
Get-FileHash .\Orqestra_1.0.5_x64-setup.exe -Algorithm SHA256
```

Compare against the SHA256 in `release-manifest.json` or `checksums.txt` attached to the release.

### Unsigned Installer Warning

The installer is **unsigned**. Windows SmartScreen will show a warning. This is expected beta behavior. Click "More info" -> "Run anyway" to proceed.

See [Release Signing Plan](docs/release-signing-plan.md) for the path beyond unsigned beta.

## First Launch

1. Launch Orqestra — the onboarding wizard appears
2. Click **"Try sample project"** — generates a demo with 4 tasks
3. Explore Table, Gantt, and Kanban views
4. Check **Setup** panel for environment status

### Open Your Own Project

1. Click **"Open existing project"**
2. Select a folder with a `roadmap/` directory containing task `.md` files
3. Each task needs YAML frontmatter with `pm-task: true`

```yaml
---
pm-task: true
id: TASK-001
title: "My task"
status: backlog
priority: High
created: "2026-06-01T00:00:00Z"
updated: "2026-06-01T00:00:00Z"
---
Task description here.
```

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows x64 | tested | NSIS installer, unsigned beta |
| macOS | not-built | Deferred to future release |
| Linux x64 | built-but-unverified | CI builds exist, not locally validated |

## What Works in v1.0.5

| Feature | Status | Notes |
|---------|--------|-------|
| Roadmap parsing | Implemented and verified | Local |
| Desktop PM views | Implemented and verified | Table, Gantt, Kanban |
| Dashboard | Deployed at [orqestra.pages.dev](https://orqestra.pages.dev) | CI auto-deployed |
| OS keychain credentials | Implemented and verified | Windows Credential Manager |
| Docs agent | Implemented, review-only | Real AI when ZAI_API_KEY set; degraded without it |
| Bugfix agent | Implemented, review-only | User-selected files only |
| First-run onboarding | Implemented and verified | Guided wizard with sample project |
| Environment readiness | Implemented and verified | Setup checks for all integrations |
| Project validation | Implemented and verified | Validates folder before loading |
| Diagnostics export | Implemented and verified | Redacted bundle, no raw secrets |
| Release manifest | Implemented and verified | SHA256 checksums, provenance, platform labels |
| AI demo fixtures | Implemented | Deterministic inputs for docs/bugfix agent demos |
| Manifest validation | Implemented | `scripts/validate-release-manifest.ts` |

## No-Key Beta Mode

Works out of the box with no API keys. AI features show as "degraded" or "mock". All other features work normally. This is the default reviewer experience.

## Real-AI Maintainer Mode

Requires `ZAI_API_KEY` set in `services/ai/.env`:

```bash
cd services/ai
echo "ZAI_API_KEY=your-key-here" > .env
uv run uvicorn orqestra_ai.main:app
```

Docs-agent and bugfix-agent will produce real AI proposals. **All agent outputs are review-only** — no autonomous commits. Agent actions require human review before any commit.

## Known Limitations

- **Windows installer is unsigned** — SmartScreen warnings are expected
- **macOS artifacts are not built** — not available for this release
- **Linux artifacts are CI-built but not locally verified** — not recommended for public beta users
- **Architect agent** — mock-mode, not production
- **ML-Master exploration** — stub, not implemented
- **Edge relay / CRDT sync** — not available
- **Full native Git** — 8 shell-outs remain (commit creation is native gix)
- **AST code analysis** — not started
- **Code signing** — planned, see [signing plan](docs/release-signing-plan.md)

## Security Notes

- Diagnostics export redacts all known secret patterns
- Readiness DTOs never expose raw tokens or keys
- Agent actions require human review before any commit
- Credential storage uses OS keychain (Windows Credential Manager)
- **Test on non-sensitive repositories first**

## Release Provenance

Each release includes `release-manifest.json` with:
- Full 40-char Git SHAs (tag commit, source commit, build commit)
- CI workflow run ID
- Artifact SHA256 checksums
- Platform status matrix
- Verification results

See the manifest attached to the [v1.0.5 release](https://github.com/Elephant-Rock-Lab/Orqestra/releases/tag/v1.0.5).

## Diagnostics

Open the **Diagnostics** panel to export a redacted support bundle. All secrets are automatically stripped before the bundle leaves the app. See [docs/DIAGNOSTICS.md](docs/DIAGNOSTICS.md).

## Documentation

| Document | Description |
|----------|-------------|
| [User Guide](docs/USER_GUIDE.md) | Complete usage guide |
| [First Run](docs/FIRST_RUN.md) | Quick start for new users |
| [Setup Checks](docs/SETUP_CHECKS.md) | Environment readiness reference |
| [Diagnostics](docs/DIAGNOSTICS.md) | Troubleshooting and export |
| [Release Artifacts](docs/RELEASE_ARTIFACTS.md) | Platform downloads and limitations |
| [Signing Plan](docs/release-signing-plan.md) | Path to signed, notarized releases |
| [Demo Script](docs/DEMO_SCRIPT_v1.0.4.md) | Deterministic demo walkthrough |
| [Demo Evidence](demo/v1.0.5-demo-evidence.md) | v1.0.5 verification record |
| [Windows Smoke](demo/v1.0.5-windows-smoke.md) | Windows installer smoke test |

## Developer Setup

<details>
<summary>Build from source</summary>

### Prerequisites

- Rust 1.80+ (`rustup`)
- Node.js 20+ and npm
- Python 3.11+ and `uv` (for AI service)
- Git

### Build

```bash
git clone https://github.com/Elephant-Rock-Lab/Orqestra.git
cd Orqestra

# Build Rust workspace (4 crates + Tauri app)
cargo build --workspace

# Build desktop frontend
cd apps/desktop && npm ci && npm run build

# Build dashboard
cd apps/dashboard && npm ci && npm run build
```

### Test

```bash
# Rust tests; exact count recorded in demo evidence
cargo test --workspace

# TypeScript builds
npm run build -w apps/desktop
npm run build -w apps/dashboard
```

### Run AI Service

```bash
cd services/ai
uv run uvicorn orqestra_ai.main:app --port 8000
```

### Run Desktop (dev mode)

```bash
cd apps/desktop
npm run tauri dev
```

### Validate Release Manifest

```bash
npx tsx scripts/validate-release-manifest.ts release-manifest.json
```

### Architecture

```
Orqestra/
+-- crates/
|   +-- md-indexer/       # Markdown roadmap parser
|   +-- git-bridge/       # Semantic commits, backfill
|   +-- graph-store/      # Triple store for history
|   +-- loro-engine/      # CRDT per-file sync
+-- apps/
|   +-- desktop/          # Tauri 2.x + React app
|   +-- dashboard/        # Cloudflare Pages dashboard
+-- services/
|   +-- ai/               # FastAPI AI service
+-- agents/               # Agent workspaces and skills
+-- roadmap/              # Project roadmap tasks
+-- scripts/              # Release tooling
+-- demo/                 # Demo fixtures and evidence
```

</details>

## License

Proprietary — Elephant Rock Lab.
