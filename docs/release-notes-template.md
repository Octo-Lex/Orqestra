# Orqestra v1.0.6 — Beta Distribution Hardening

## Summary

Orqestra v1.0.6 is a beta distribution hardening release. It improves the public beta install path, verification instructions, first-run guidance, troubleshooting documentation, issue templates, dashboard freshness, and signing readiness. It does not introduce major new product functionality.

## Who This Beta Is For

Technical reviewers and early adopters evaluating Orqestra as a local-first, AI-native project management tool for Git repositories.

## Download and Verify

Download `Orqestra_1.0.6_x64-setup.exe` from the assets below.

```powershell
Get-FileHash .\Orqestra_1.0.6_x64-setup.exe -Algorithm SHA256
```

Compare against `checksums.txt` or `release-manifest.json`.

The installer is **unsigned**. Windows SmartScreen will warn you. Click "More info" → "Run anyway".

## First-Run Quickstart

1. Launch Orqestra → onboarding wizard appears
2. Click **"Try sample project"** → demo with 4 tasks
3. Explore Table, Gantt, and Kanban views
4. Check **Setup** panel for environment status

Full guide: [Beta Quickstart](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/docs/beta-quickstart.md)

## No-Key Beta Mode

Works out of the box. AI features show as "degraded". All roadmap, PM views, dashboard, and diagnostic features work normally.

## Real-AI Maintainer Mode

Requires `ZAI_API_KEY` in `services/ai/.env`. Start AI service with `cd services/ai && uv run uvicorn orqestra_ai.main:app`. All agent outputs are **review-only**.

## Dashboard Freshness

Live at [orqestra.pages.dev](https://orqestra.pages.dev) — footer shows release version, source commit, and generation timestamp.

## Platform Status

| Platform | Status |
|----------|--------|
| Windows x64 | tested |
| macOS | not-built |
| Linux x64 | built-but-unverified |

## Signing Status

Unsigned beta. See [Signing Plan](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/docs/release-signing-plan.md) for readiness status.

## Troubleshooting

See [Troubleshooting Guide](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/docs/troubleshooting.md) for common issues.

## Report an Issue

- [Install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml)
- [AI mode issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=ai_mode_issue.yml)
- [Dashboard issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=dashboard_issue.yml)
- [Bug report](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=bug_report.yml)

**Do not paste API keys or secrets in issues.**

## Known Limitations

- Unsigned installer (SmartScreen warnings expected)
- macOS artifacts not available
- Linux not verified for public beta
- Architect agent is mock-mode
- ML-Master is stub
- Edge relay / CRDT sync not available
- Code signing pending

## Release Provenance

See `release-manifest.json` for full provenance: tag commit, source commit, build commit, CI workflow run ID, artifact checksums, platform matrix, distribution metadata, dashboard freshness.

## Checksums

See `checksums.txt` in the release assets.

## Demo Evidence

- [Demo evidence](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/demo/v1.0.6-demo-evidence.md)
- [Windows smoke test](https://github.com/Elephant-Rock-Lab/Orqestra/blob/master/demo/v1.0.6-windows-smoke.md)
