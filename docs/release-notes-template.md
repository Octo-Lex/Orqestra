# Orqestra v1.0.11 -- Linux WSLg Runtime Evidence

## Summary

v1.0.11 verifies that the Linux AppImage launches successfully under WSLg with GTK/WebKit initialized, but Linux is not promoted to a tested beta platform because native Linux desktop smoke has not yet been run.

## Download

| Platform | File | SHA256 | Status |
|----------|------|--------|--------|
| Windows x64 | `Orqestra_1.0.11_x64-setup.exe` | See `checksums.txt` | Smoke-tested |
| Linux x64 | `Orqestra_1.0.11_x64.AppImage` | See `checksums.txt` | runtime-evidence-wslg |

## Platform Status

| Platform | Status | Artifact | Runtime Env | Blocking |
|----------|--------|----------|-------------|----------|
| Windows x64 | tested | NSIS | Windows 11 Pro | yes |
| Linux x64 | runtime-evidence-wslg | AppImage | WSLg Ubuntu 24.04 | no |
| macOS | build-feasibility-verified | none | CI only | no |

## Linux WSLg Runtime Evidence

The Linux AppImage was tested under WSLg (Windows Subsystem for Linux GUI) on Ubuntu 24.04:

- **Launch:** Pass -- AppImage launches via `systemd-run`
- **Main window:** 1280x720 "Orqestra" window created
- **WebKit:** NetworkProcess + WebProcess running
- **Memory:** 384MB RSS, stable for 6+ minutes
- **Screenshot:** Captured (see release assets)

GTK initialized, WebKit rendered content, and the app ran stably. However, WSLg is a compatibility layer, not a native Linux desktop. Steps 8-12 of the smoke flow (folder selection, roadmap UI, dashboard) could not be completed because WSLg provides no interactive desktop session in this configuration.

**Linux is not promoted to tested until the AppImage passes on a native Linux desktop.**

## Verify SHA256

Windows:
```powershell
Get-FileHash .\Orqestra_1.0.11_x64-setup.exe -Algorithm SHA256
```

Linux:
```bash
sha256sum Orqestra_1.0.11_x64.AppImage
```

## Platform Evidence

- Platform matrix: `demo/v1.0.11-platform-matrix.md`
- Windows smoke: `demo/v1.0.11-windows-smoke.md`
- Linux WSLg smoke: `demo/v1.0.11-linux-wslg-smoke.md`
- WSLg screenshot: `demo/v1.0.11-wslg-screenshot.png`
- Demo evidence: `demo/v1.0.11-demo-evidence.md`

## Signing Status

Windows signing is **blocked** (certificate-not-available). The installer is unsigned. SmartScreen will warn.

## Known Limitations

- Windows installer is unsigned
- Linux AppImage runtime-evidence-wslg: passes under WSLg, not promoted
- Linux not promoted without native desktop smoke
- macOS has no bundled artifact
- Some agent paths remain review-only or scaffolded

## Checksums

See `checksums.txt` in release assets.
