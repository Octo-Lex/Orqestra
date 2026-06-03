# Orqestra v1.0.12 -- Linux Promoted to Tested

## Summary

v1.0.12 promotes Linux x64 to a tested beta platform for the AppImage artifact after native Ubuntu 24.04 GNOME smoke verification. Linux support remains beta-grade and validated only for the recorded environment.

## Download

| Platform | File | SHA256 | Status |
|----------|------|--------|--------|
| Windows x64 | `Orqestra_1.0.12_x64-setup.exe` | See `checksums.txt` | Smoke-tested (15/15) |
| Linux x64 | `Orqestra_1.0.12_x64.AppImage` | See `checksums.txt` | Smoke-tested (9/9) |

## Platform Status

| Platform | Status | Artifact | Runtime Env | Blocking |
|----------|--------|----------|-------------|----------|
| Windows x64 | tested | NSIS | Windows 11 Pro | yes |
| Linux x64 | tested | AppImage | Ubuntu 24.04.4 GNOME | no |
| macOS | build-feasibility-verified | none | CI only | no |

## Linux Smoke Verification

Native Ubuntu 24.04.4 LTS GNOME smoke on QEMU VM (Proxmox 8.4.10):

- WebKit2GTK 2.52.3 (NetworkProcess + WebProcess running)
- Orqestra window 1280x768 confirmed via xwininfo
- AppImage SHA256 verified
- Close and relaunch successful
- Memory stable at 162 MB RSS

**Caveat:** Dashboard link open deferred (headless VM, no browser). Dashboard availability verified independently (200 OK).

## Contributor Smoke Kit

For testing on other Linux distros:

1. **[Smoke Guide](../blob/master/docs/linux-native-smoke-guide.md)** -- step-by-step instructions
2. **[Evidence Template](../blob/master/docs/linux-smoke-evidence-template.md)** -- copy-paste template
3. **[Issue Form](../../issues/new?template=linux_smoke_report.yml)** -- submit results on GitHub

## Signing Status

Windows signing is **blocked** (certificate-not-available). The installer is unsigned. SmartScreen will warn.

## Known Limitations

- Windows installer is unsigned
- Linux screenshot blocked by Wayland rootless compositor; process+window evidence recorded
- Linux tested on Ubuntu 24.04 only; other distros welcome
- macOS has no bundled artifact

## Checksums

See `checksums.txt` in release assets.
