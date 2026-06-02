# Orqestra v1.0.8 -- Platform Verification Beta

## Summary

v1.0.8 adds cross-platform CI evidence. Linux x64 and macOS now have documented compile evidence from CI, but neither is promoted because no standard bundle and no smoke evidence exist. Windows remains the only tested public beta platform.

## Download

| Platform | File | SHA256 |
|----------|------|--------|
| Windows x64 | `Orqestra_1.0.8_x64-setup.exe` | See `checksums.txt` or `release-manifest.json` |

## Platform Status

| Platform | Status | Compile | Bundle | Smoke | Release Blocking |
|----------|--------|---------|--------|-------|-----------------|
| Windows x64 | tested | pass | NSIS installer | pass | yes |
| Linux x64 | built-but-unverified | pass | not configured | no | no |
| macOS | build-feasibility-verified | pass | not configured | no | no |

**Compile success is not platform support.** No platform is promoted unless it has a bundled artifact, checksum, smoke evidence, and matching manifest entry.

## Windows x64

- Primary tested public beta platform
- NSIS installer: `Orqestra_1.0.8_x64-setup.exe`
- Unsigned -- SmartScreen warnings expected
- 15/15 smoke steps pass

## Linux x64

- CI compiles the binary successfully (141 tests pass)
- No AppImage, DEB, or RPM bundle produced (bundler targets not configured)
- Not smoke-tested
- Not promoted

## macOS

- CI compiles universal binary for `universal-apple-darwin` (141 tests pass)
- No DMG or app bundle produced (bundler targets not configured)
- Not signed, not notarized, not smoke-tested
- Not promoted

## Verify SHA256

```powershell
Get-FileHash .\Orqestra_1.0.8_x64-setup.exe -Algorithm SHA256
```

Compare against `checksums.txt` or `release-manifest.json` in release assets.

## Platform Evidence

- Platform matrix: `demo/v1.0.8-platform-matrix.md`
- Windows smoke: `demo/v1.0.8-windows-smoke.md`
- Linux build evidence: `demo/v1.0.8-linux-build-evidence.md`
- macOS feasibility: `demo/v1.0.8-macos-feasibility.md`
- Demo evidence: `demo/v1.0.8-demo-evidence.md`

## Signing Status

Windows signing is **blocked** (certificate-not-available). The installer is unsigned. SmartScreen will warn. See [Signing Plan](docs/release-signing-plan.md).

## Known Limitations

- Windows installer is unsigned
- Linux has no bundled artifact (AppImage/DEB)
- macOS has no bundled artifact (DMG)
- Compile success is not platform support
- Some agent paths remain review-only or scaffolded

## Release Provenance

See `release-manifest.json` in release assets for: full Git SHAs, CI workflow run ID, artifact checksums, signing status, platform matrix, compile/bundle status, and diagnostics links.

## Checksums

See `checksums.txt` in release assets.
