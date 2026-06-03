# Orqestra v1.0.10 -- Linux Smoke Verification

## Summary

v1.0.10 records the Linux AppImage runtime blocker found under WSL2/Xvfb testing. The AppImage binary is valid and all shared dependencies resolve, but GTK initialization fails without a display server. Linux remains unpromoted pending a native Linux desktop smoke test.

## Download

| Platform | File | SHA256 | Status |
|----------|------|--------|--------|
| Windows x64 | `Orqestra_1.0.10_x64-setup.exe` | See `checksums.txt` | Smoke-tested |
| Linux x64 | `Orqestra_1.0.10_x64.AppImage` | See `checksums.txt` | Runtime-blocked |

## Platform Status

| Platform | Status | Artifact | Smoke | Runtime | Blocking |
|----------|--------|----------|-------|---------|----------|
| Windows x64 | tested | NSIS | pass | Windows 10 | yes |
| Linux x64 | runtime-blocked | AppImage | blocked | WSL2 (no display) | no |
| macOS | build-feasibility-verified | none | -- | CI only | no |

## Linux Runtime Blocker

**The Linux AppImage runtime was tested under WSL2 Ubuntu 24.04.** GTK initialization failed because no display server (X11 or Wayland) was available:

```
thread 'main' panicked at tao-0.35.3/src/platform_impl/linux/event_loop.rs:217:53:
Failed to initialize gtk backend!: BoolError { message: "Failed to initialize GTK" }
```

This is an environmental limitation, not an application defect. On a native Linux desktop with a real display server, the app is expected to launch normally.

**Linux is not promoted to tested until the AppImage passes on a native Linux desktop.**

## Verify SHA256

Windows:
```powershell
Get-FileHash .\Orqestra_1.0.10_x64-setup.exe -Algorithm SHA256
```

Linux:
```bash
sha256sum Orqestra_1.0.10_x64.AppImage
```

## Platform Evidence

- Platform matrix: `demo/v1.0.10-platform-matrix.md`
- Windows smoke: `demo/v1.0.10-windows-smoke.md`
- Linux WSL2 smoke attempt: `demo/v1.0.10-linux-wsl2-smoke-attempt.md`
- Linux runtime blocker: `demo/v1.0.10-linux-runtime-blocker.md`
- Demo evidence: `demo/v1.0.10-demo-evidence.md`

## Signing Status

Windows signing is **blocked** (certificate-not-available). The installer is unsigned. SmartScreen will warn.

## Known Limitations

- Windows installer is unsigned
- Linux AppImage runtime-blocked: GTK init fails without display server
- Linux not promoted without native desktop smoke
- macOS has no bundled artifact
- Some agent paths remain review-only or scaffolded

## Checksums

See `checksums.txt` in release assets.
