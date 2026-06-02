# Orqestra

**Local-first, AI-native project management for Git repositories.**

Orqestra turns a Git repository into a structured workspace with roadmap tracking, semantic history, AI-assisted code review, and an optional public dashboard — all running locally.

## Public Beta Status

Orqestra v1.0.6 is a **public beta** for technical reviewers and early adopters. It includes a tested Windows x64 installer, a live dashboard, roadmap indexing, semantic commit infrastructure, and real-AI review flows. It is not yet a production product. The installer is unsigned, macOS artifacts are not yet provided, Linux is not yet verified, and some advanced agent paths remain review-only or scaffolded.

## Quick Start for Public Beta Reviewers

### 1. Download

Download `Orqestra_1.0.6_x64-setup.exe` from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases).

The installer is **unsigned**. Windows SmartScreen will warn you. Click "More info" → "Run anyway".

### 2. Verify SHA256

```powershell
Get-FileHash .\Orqestra_1.0.6_x64-setup.exe -Algorithm SHA256
```

Compare against `checksums.txt` or `release-manifest.json` attached to the release.

### 3. Install and Launch

Run the installer, then open Orqestra from the Start menu. The onboarding wizard appears.

### 4. Run the No-Key Beta Demo

No API key needed. Click **"Try sample project"** in the onboarding wizard. Explore Table, Gantt, and Kanban views. AI features will show as "degraded" — this is correct.

For step-by-step instructions, see the **[Beta Quickstart](docs/beta-quickstart.md)**.

### 5. Optional: Real-AI Maintainer Mode

If you have a `ZAI_API_KEY`, see the [Real-AI setup instructions](docs/beta-quickstart.md#8-optional-real-ai-maintainer-mode) in the quickstart guide.

### Troubleshooting

If anything goes wrong, see **[Troubleshooting Guide](docs/troubleshooting.md)** or [file an issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new/choose).

---

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows x64 | tested | NSIS installer, unsigned beta |
| macOS | not-built | Deferred to future release |
| Linux x64 | built-but-unverified | CI builds exist, not locally validated |

## What Works

| Feature | Status | Notes |
|---------|--------|-------|
| Roadmap parsing | Implemented and verified | Local |
| Desktop PM views | Implemented and verified | Table, Gantt, Kanban |
| Dashboard | Deployed at [orqestra.pages.dev](https://orqestra.pages.dev) | CI auto-deployed, shows freshness metadata |
| OS keychain credentials | Implemented and verified | Windows Credential Manager |
| Docs agent | Implemented, review-only | Real AI when ZAI_API_KEY set; degraded without it |
| Bugfix agent | Implemented, review-only | User-selected files only |
| First-run onboarding | Implemented and verified | Guided wizard with sample project |
| Environment readiness | Implemented and verified | Setup checks for all integrations |
| Project validation | Implemented and verified | Validates folder before loading |
| Diagnostics export | Implemented and verified | Redacted bundle, no raw secrets |
| Release manifest | Implemented and verified | SHA256 checksums, provenance, platform labels |
| Manifest validation | Implemented | `scripts/validate-release-manifest.ts` |
| Dashboard freshness | Implemented | Release version, timestamp, source commit visible |
| Beta quickstart | Implemented | [docs/beta-quickstart.md](docs/beta-quickstart.md) |
| Troubleshooting guide | Implemented | [docs/troubleshooting.md](docs/troubleshooting.md) |
| Issue templates | Implemented | Install, AI mode, dashboard, bug report |

## No-Key Beta Mode

Works out of the box with no API keys. AI features show as "degraded" or "mock". All other features work normally. This is the default reviewer experience.

## Real-AI Maintainer Mode

Requires `ZAI_API_KEY` in `services/ai/.env`. Docs-agent and bugfix-agent produce real AI proposals. **All agent outputs are review-only** — no autonomous commits. See the [quickstart](docs/beta-quickstart.md#8-optional-real-ai-maintainer-mode) for setup.

## Known Limitations

- **Windows installer is unsigned** — SmartScreen warnings are expected
- **macOS artifacts are not built** — not available for this release
- **Linux artifacts are CI-built but not locally verified** — not recommended for public beta
- **Architect agent** — mock-mode
- **ML-Master exploration** — stub
- **Edge relay / CRDT sync** — not available
- **Full native Git** — 8 shell-outs remain (commit creation is native gix)
- **Code signing** — planned, see [signing plan](docs/release-signing-plan.md)

## Security Notes

- Diagnostics export redacts all known secret patterns
- Readiness DTOs never expose raw tokens or keys
- Agent actions require human review before any commit
- **Test on non-sensitive repositories first**

## Report an Issue

- [Install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml)
- [AI mode issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=ai_mode_issue.yml)
- [Dashboard issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=dashboard_issue.yml)
- [Bug report](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=bug_report.yml)

**Do not paste API keys or secrets in issues.**

## Release Provenance

Each release includes `release-manifest.json` with:
- Full 40-char Git SHAs (tag commit, source commit, build commit)
- CI workflow run ID
- Artifact SHA256 checksums
- Platform status matrix
- Distribution metadata (quickstart, troubleshooting, issue templates)
- Dashboard freshness metadata

## Documentation

| Document | Description |
|----------|-------------|
| [Beta Quickstart](docs/beta-quickstart.md) | Step-by-step reviewer guide |
| [Troubleshooting](docs/troubleshooting.md) | Common issues and fixes |
| [User Guide](docs/USER_GUIDE.md) | Complete usage guide |
| [First Run](docs/FIRST_RUN.md) | Quick start for new users |
| [Setup Checks](docs/SETUP_CHECKS.md) | Environment readiness reference |
| [Diagnostics](docs/DIAGNOSTICS.md) | Troubleshooting and export |
| [Release Artifacts](docs/RELEASE_ARTIFACTS.md) | Platform downloads and limitations |
| [Signing Plan](docs/release-signing-plan.md) | Path to signed, notarized releases |
| [Demo Evidence](demo/v1.0.6-demo-evidence.md) | v1.0.6 verification record |
| [Windows Smoke](demo/v1.0.6-windows-smoke.md) | Windows installer smoke test |

## Developer Setup

<details>
<summary>Build from source</summary>

### Prerequisites

- Rust 1.80+, Node.js 20+, Python 3.11+, Git

### Build

```bash
git clone https://github.com/Elephant-Rock-Lab/Orqestra.git
cd Orqestra
cargo build --workspace
cd apps/desktop && npm ci && npm run build
cd apps/dashboard && npm ci && npm run build
```

### Test

```bash
cargo test --workspace
```

### Validate Release Manifest

```bash
npx tsx scripts/validate-release-manifest.ts release-manifest.json
```

</details>

## License

Proprietary — Elephant Rock Lab.
