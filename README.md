# Orqestra

**Local-first, AI-native project management for Git repositories.**

Orqestra turns a Git repository into a structured workspace with roadmap tracking, semantic history, AI-assisted code review, and an optional public dashboard — all running locally.

## Public Beta Status

Orqestra v1.0.10 is a **public beta** for technical reviewers and early adopters. v1.0.10 does not yet include a signed Windows installer. It records the Linux AppImage runtime blocker found under WSL2/Xvfb testing; Linux remains unpromoted pending a native Linux desktop smoke test and blocker resolution.

## Quick Start for Public Beta Reviewers

### 1. Download

Download `Orqestra_1.0.10_x64-setup.exe` from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases).

The installer is **unsigned**. Windows SmartScreen will warn you. Click "More info" → "Run anyway".

### 2. Verify SHA256

```powershell
Get-FileHash .\Orqestra_1.0.10_x64-setup.exe -Algorithm SHA256
```

Compare against `checksums.txt` or `release-manifest.json` attached to the release.

### 3. Verify Signature

```powershell
Get-AuthenticodeSignature .\Orqestra_1.0.10_x64-setup.exe
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

The Windows installer is unsigned. Windows SmartScreen warnings are expected. v1.0.10 records the signing blocker status and the next action toward signed distribution.

Even when signing is implemented, SmartScreen may still warn for new or low-reputation downloads until reputation is established.

See [Signing Plan](docs/release-signing-plan.md) for the full path.

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows x64 | tested | NSIS installer, unsigned beta |
| Linux x64 | runtime-blocked | AppImage blocked: GTK init fails without display server |
| macOS | build-feasibility-verified | CI compiles universal binary, no DMG/app bundle |

See [Platform Confidence](docs/platform-confidence.md) for what each status means and promotion criteria.

### Linux AppImage Warning

The Linux AppImage runtime was tested under WSL2/Xvfb. GTK initialization failed because no display server was available. Linux remains unpromoted until the AppImage passes on a native Linux desktop with a real display server (X11 or Wayland).

To verify the Linux AppImage:

```bash
sha256sum Orqestra_1.0.10_x64.AppImage
chmod a+x Orqestra_1.0.10_x64.AppImage
./Orqestra_1.0.10_x64.AppImage
```

If you encounter issues, see [Troubleshooting](docs/troubleshooting.md) or [file an issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml).

## What Works

| Feature | Status | Notes |
|---------|--------|-------|
| Roadmap parsing | Implemented and verified | Local |
| Desktop PM views | Implemented and verified | Table, Gantt, Kanban |
| Dashboard | Deployed at [orqestra.pages.dev](https://orqestra.pages.dev) | CI auto-deployed, freshness metadata |
| OS keychain credentials | Implemented and verified | Windows Credential Manager |
| Docs agent | Implemented, review-only | Real AI when ZAI_API_KEY set |
| Bugfix agent | Implemented, review-only | User-selected files only |
| First-run onboarding | Implemented and verified | Guided wizard with sample project |
| Environment readiness | Implemented and verified | Setup checks for all integrations |
| Diagnostics export | Implemented and verified | Redacted bundle, no raw secrets |
| Release manifest | Implemented and verified | Provenance, signing, diagnostics, platform fields |
| Dashboard freshness | Implemented | Version, commit, timestamp in footer |
| Beta quickstart | Implemented | [docs/beta-quickstart.md](docs/beta-quickstart.md) |
| Troubleshooting | Implemented | [docs/troubleshooting.md](docs/troubleshooting.md) |
| Installer diagnostics | Implemented | [docs/installer-diagnostics.md](docs/installer-diagnostics.md) |
| Platform confidence | Implemented | [docs/platform-confidence.md](docs/platform-confidence.md) |
| Issue triage | Implemented | [docs/beta-issue-triage.md](docs/beta-issue-triage.md) |
| Issue templates | Implemented | Install, AI mode, dashboard, bug report |

## Known Limitations

- **Windows installer is unsigned** — SmartScreen warnings are expected
- **macOS artifacts are not built** — not available for this release
- **Linux artifacts are CI-built but not verified** — not recommended for public beta
- **Architect agent** — mock-mode
- **ML-Master exploration** — stub
- **Edge relay / CRDT sync** — not available
- **Full native Git** — 8 shell-outs remain
- **Code signing** — blocked, certificate not available

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
