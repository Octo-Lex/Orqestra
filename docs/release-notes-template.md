# Orqestra v1.0.9 -- Linux Bundle Verification

## Summary

v1.0.9 introduces a CI-produced Linux AppImage with SHA256 verification, but Linux is not yet a tested public beta platform because no desktop smoke test has been performed.

## Download

| Platform | File | SHA256 | Verification |
|----------|------|--------|-------------|
| Windows x64 | `Orqestra_1.0.9_x64-setup.exe` | See `checksums.txt` | Smoke-tested |
| Linux x64 | `Orqestra_1.0.9_x64.AppImage` | See `checksums.txt` | Checksummed, not smoke-tested |

## Platform Status

| Platform | Status | Compile | Bundle | Smoke | Release Blocking |
|----------|--------|---------|--------|-------|-----------------|
| Windows x64 | tested | pass | NSIS installer | pass | yes |
| Linux x64 | bundle-produced-unverified | pass | AppImage | not run | no |
| macOS | build-feasibility-verified | pass | not configured | not run | no |

## Windows x64

- Primary tested public beta platform
- NSIS installer: `Orqestra_1.0.9_x64-setup.exe`
- Unsigned -- SmartScreen warnings expected
- 15/15 smoke steps pass

## Linux x64

**The Linux AppImage is provided for early verification only. It has a checksum and was produced by CI, but it has not been smoke-tested on a Linux desktop. Linux is not yet a tested public beta platform.**

- CI produces AppImage (Tauri `appimage` target added)
- SHA256 checksum is published
- No smoke test was performed (no Linux desktop available)
- To verify: `sha256sum Orqestra_1.0.9_x64.AppImage && chmod a+x Orqestra_1.0.9_x64.AppImage && ./Orqestra_1.0.9_x64.AppImage`

## macOS

- Unchanged from v1.0.8
- CI compiles universal binary for `universal-apple-darwin`
- No DMG or app bundle produced

## Verify SHA256

Windows:
```powershell
Get-FileHash .\Orqestra_1.0.9_x64-setup.exe -Algorithm SHA256
```

Linux:
```bash
sha256sum Orqestra_1.0.9_x64.AppImage
```

Compare against `checksums.txt` or `release-manifest.json` in release assets.

## Platform Evidence

- Platform matrix: `demo/v1.0.9-platform-matrix.md`
- Windows smoke: `demo/v1.0.9-windows-smoke.md`
- Linux bundle evidence: `demo/v1.0.9-linux-bundle-evidence.md`
- Linux smoke blocker: `demo/v1.0.9-linux-smoke-blocked.md`
- Demo evidence: `demo/v1.0.9-demo-evidence.md`

## Signing Status

Windows signing is **blocked** (certificate-not-available). The installer is unsigned. SmartScreen will warn. See [Signing Plan](docs/release-signing-plan.md).

## Known Limitations

- Windows installer is unsigned
- Linux AppImage is checksummed but not smoke-tested
- Linux is not a tested public beta platform
- macOS has no bundled artifact
- Some agent paths remain review-only or scaffolded

## Release Provenance

See `release-manifest.json` in release assets for: full Git SHAs, CI workflow run ID, artifact checksums, signing status, platform matrix, compile/bundle status, and diagnostics links.

## Checksums

See `checksums.txt` in release assets.
