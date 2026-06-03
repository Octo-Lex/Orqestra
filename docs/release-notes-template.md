# Orqestra v1.0.12 -- Native Linux Contributor Smoke Enablement

## Summary

v1.0.12 does not promote Linux to a tested beta platform. It publishes a native Linux smoke guide, evidence template, and GitHub report form so contributors with a real Linux desktop can run the promotion-grade AppImage smoke test. Prior WSLg runtime evidence remains recorded.

## Download

| Platform | File | SHA256 | Status |
|----------|------|--------|--------|
| Windows x64 | `Orqestra_1.0.12_x64-setup.exe` | See `checksums.txt` | Smoke-tested |
| Linux x64 | `Orqestra_1.0.12_x64.AppImage` | See `checksums.txt` | Native-smoke-blocked |

## Platform Status

| Platform | Status | Artifact | Runtime Env | Blocking |
|----------|--------|----------|-------------|----------|
| Windows x64 | tested | NSIS | Windows 11 Pro | yes |
| Linux x64 | native-smoke-blocked | AppImage | Contributor kit published | no |
| macOS | build-feasibility-verified | none | CI only | no |

## Linux Contributor Smoke Kit

This release includes three tools for Linux contributors:

1. **[Smoke Guide](../docs/linux-native-smoke-guide.md)** -- step-by-step instructions for testing the AppImage on a native Linux desktop
2. **[Evidence Template](../docs/linux-smoke-evidence-template.md)** -- copy-paste template to record your results
3. **[Issue Form](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=linux_smoke_report.yml)** -- submit your smoke results directly on GitHub

If you have a native Linux desktop (Ubuntu 24.04 GNOME, Fedora, Debian, etc.), you can help promote Linux to a tested platform by running the 9-step smoke flow and submitting evidence.

## Prior WSLg Evidence

The v1.0.11 WSLg evidence shows the app runs successfully:
- Main window 1280x720
- WebKit processes running
- 384MB stable for 6+ minutes
- Screenshot included

This evidence is preserved but not used for promotion because WSLg is not a native desktop.

## Signing Status

Windows signing is **blocked** (certificate-not-available). The installer is unsigned. SmartScreen will warn.

## Known Limitations

- Windows installer is unsigned
- Linux contributor smoke kit published, native smoke not yet completed
- Linux not promoted without native desktop smoke
- macOS has no bundled artifact

## Checksums

See `checksums.txt` in release assets.
