# Release Artifacts

## Unsigned Beta Warning

Orqestra desktop artifacts are **unsigned beta builds**. Your operating system may show a warning before launch.

- **Windows:** SmartScreen may block the installer. Click "More info" -> "Run anyway".
- **Linux:** The AppImage is checksummed but not smoke-tested. See Linux AppImage Warning below.

Code signing and notarization are planned for a future production release.

## v1.0.9 Artifacts

| Platform | Artifact | Status | Signed | Notes |
|----------|----------|--------|--------|-------|
| Windows x64 | `Orqestra_1.0.9_x64-setup.exe` | tested | No | Unsigned public beta, signing blocked |
| Linux x64 | `Orqestra_1.0.9_x64.AppImage` | bundle-produced-unverified | N/A | CI-produced AppImage, not smoke-tested |
| macOS | none | build-feasibility-verified | No | CI compiles universal binary, no DMG |

## Linux AppImage Warning

The Linux AppImage is provided for early verification only. It has a checksum and was produced by CI, but it has not been smoke-tested on a Linux desktop. Linux is not yet a tested public beta platform.

## Downloading

Download from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases).

## Installing

### Windows
1. Download `Orqestra_1.0.9_x64-setup.exe`
2. Verify SHA256 against `checksums.txt` or `release-manifest.json`
3. Run the installer
4. Launch Orqestra from Start Menu

**Note:** The installer is unsigned. Windows SmartScreen may show a warning. Click "More info" -> "Run anyway" to proceed.

### Linux
1. Download `Orqestra_1.0.9_x64.AppImage`
2. Verify SHA256: `sha256sum Orqestra_1.0.9_x64.AppImage`
3. Mark executable: `chmod a+x Orqestra_1.0.9_x64.AppImage`
4. Launch: `./Orqestra_1.0.9_x64.AppImage`

If the AppImage does not launch, see [Troubleshooting](troubleshooting.md).

### macOS

Not available. CI compiles a universal binary but no DMG or app bundle is produced.

## Building from Source

See README.md for developer setup instructions.

## Artifact Manifest

Each release includes `release-manifest.json` with per-platform entries including:

- Full 40-char Git commit SHAs
- Full 64-char SHA256 checksums
- Signing status and blocker details
- Platform status (tested, bundle-produced-unverified, build-feasibility-verified)
- `compile_status` and `bundle_status` for non-tested platforms
- `final_artifact_state` for all artifacts
- `verification_status` on Linux artifact (`checksummed-not-smoke-tested`)
- CI workflow run ID

## Dashboard Deployment

The live dashboard at [orqestra.pages.dev](https://orqestra.pages.dev) is deployed via CI.

The dashboard deployment workflow passes Cloudflare `accountId` explicitly. This avoids relying on account discovery through the Cloudflare memberships API and prevents deployment failures when the API token is scoped only for Pages deployment.

## Platform Classification

Each platform is classified as one of:

| Status | Meaning |
|--------|---------|
| `tested` | Bundled artifact exists, smoke-tested, checksum verified, manifest agrees |
| `bundle-produced-unverified` | CI produces bundled artifact, checksum exists, no smoke test |
| `built-but-unverified` | CI compiles binary, no bundled artifact, no smoke test |
| `build-feasibility-verified` | CI compiles binary, no bundled artifact, no smoke test, compilation proven only |
| `not-built` | Not produced in this release |
| `blocked` | Known blocker preventing build |

**Compile success is not platform support.** A platform must have a bundled artifact, checksum, smoke evidence, and matching manifest entry to be promoted.

## Known Limitations

- All artifacts are **unsigned beta builds**
- Linux AppImage is checksummed but not smoke-tested
- macOS binary compiles in CI but no DMG/app bundle is produced
- Full native gix migration remains incomplete
- Architect agent remains mock-mode
- ML-Master exploration loop remains stub
