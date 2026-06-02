# Orqestra v1.0.5 — Public Beta Hardening

## Summary

Orqestra v1.0.5 is a **public beta** for technical reviewers and early adopters. It includes a tested Windows x64 installer, a live dashboard, roadmap indexing, semantic commit infrastructure, and real-AI review flows. It is not yet a production product. The installer is unsigned, macOS artifacts are not yet provided, Linux is not yet verified, and some advanced agent paths remain review-only or scaffolded.

## Download

Download `Orqestra_1.0.5_x64-setup.exe` from the assets below.

## Verify the Installer

```powershell
Get-FileHash .\Orqestra_1.0.5_x64-setup.exe -Algorithm SHA256
```

Compare the output against the SHA256 in `release-manifest.json` or `checksums.txt`.

## Unsigned Installer Warning

Orqestra v1.0.5 desktop artifacts are **unsigned beta builds**. Windows SmartScreen will show a warning. Click "More info" → "Run anyway" to proceed.

## Platform Status

| Platform | Status | Notes |
|----------|--------|-------|
| Windows x64 | tested | NSIS installer, unsigned |
| macOS | not-built | Deferred to future release |
| Linux x64 | built-but-unverified | CI builds exist, not locally validated |

## What Works

- Roadmap parsing and indexing (local)
- Desktop PM views: Table, Gantt, Kanban
- Dashboard: [orqestra.pages.dev](https://orqestra.pages.dev) (CI auto-deployed)
- OS keychain credential storage (Windows Credential Manager)
- Docs agent: review-only real-AI proposals when `ZAI_API_KEY` is set
- Bugfix agent: review-only, user-selected file scope
- First-run onboarding wizard
- Environment readiness checks
- Project validation
- Diagnostics export with secret redaction
- Release manifest with SHA256 checksums

## AI Modes

### No-Key Beta Mode (default)

Works out of the box with no API keys. AI features show as "degraded" or "mock". All other features work normally.

### Real-AI Maintainer Mode

Requires `ZAI_API_KEY` set in `services/ai/.env`. Start the AI service with:

```bash
cd services/ai
uv run uvicorn orqestra_ai.main:app
```

Docs-agent and bugfix-agent will produce real AI proposals. **All agent outputs are review-only** — no autonomous commits.

## Known Limitations

- Windows installer is **unsigned** — SmartScreen warnings are expected
- macOS artifacts are **not built**
- Linux artifacts are CI-built but **not locally verified**
- Architect agent remains **mock-mode**
- ML-Master exploration loop remains **stub**
- Full native gix migration incomplete (8 shell-outs remain)
- Code signing and notarization are **planned but not implemented**

## Release Provenance

See `release-manifest.json` for full provenance:
- Tag commit, source commit, build commit
- CI workflow run ID
- Artifact SHA256 checksums
- Platform status matrix

## Checksums

See `checksums.txt` in the release assets.

## Demo Evidence

- [Demo script](docs/DEMO_SCRIPT_v1.0.4.md)
- [Demo evidence](demo/v1.0.5-demo-evidence.md)
- [Windows smoke test](demo/v1.0.5-windows-smoke.md)

## Security Notes

- Diagnostics export redacts all known secret patterns
- Readiness DTOs never expose raw tokens or keys
- Agent actions require human review before any commit
- Test on non-sensitive repositories first
