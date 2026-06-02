# Release Artifacts

## Unsigned Beta Warning

Orqestra desktop artifacts are **unsigned beta builds**. Your operating system may show a warning before launch.

- **Windows:** SmartScreen may block the installer. Click "More info" -> "Run anyway".
- **Linux:** No bundled artifact is available for this release.
- **macOS:** No bundled artifact is available for this release.

Code signing and notarization are planned for a future production release.

## v1.0.8 Artifacts

| Platform | Artifact | Status | Signed | Notes |
|----------|----------|--------|--------|-------|
| Windows x64 | `Orqestra_1.0.8_x64-setup.exe` | tested | No | Unsigned public beta, signing blocked |
| Linux x64 | none | built-but-unverified | N/A | CI compiles binary, no AppImage/DEB bundle |
| macOS | none | build-feasibility-verified | No | CI compiles universal binary, no DMG/app bundle |

## Downloading

Download from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases).

## Installing

### Windows
1. Download `Orqestra_1.0.8_x64-setup.exe`
2. Verify SHA256 against `checksums.txt` or `release-manifest.json`
3. Run the installer
4. Launch Orqestra from Start Menu

**Note:** The installer is unsigned. Windows SmartScreen may show a warning. Click "More info" -> "Run anyway" to proceed.

### Linux

Not available. CI compiles a binary but no AppImage or DEB bundle is produced. The Tauri bundler targets for Linux are not configured.

### macOS

Not available. CI compiles a universal binary but no DMG or app bundle is produced. The Tauri bundler targets for macOS are not configured.

## Building from Source

See README.md for developer setup instructions.

## Artifact Manifest

Each release includes `release-manifest.json` with per-platform entries including:

- Full 40-char Git commit SHAs
- Full 64-char SHA256 checksums
- Signing status and blocker details
- Platform status (tested, built-but-unverified, build-feasibility-verified)
- `compile_status` and `bundle_status` for non-tested platforms
- `final_artifact_state` for all artifacts
- CI workflow run ID

## Dashboard Deployment

The live dashboard at [orqestra.pages.dev](https://orqestra.pages.dev) is deployed via CI.

The dashboard deployment workflow passes Cloudflare `accountId` explicitly. This avoids relying on account discovery through the Cloudflare memberships API and prevents deployment failures when the API token is scoped only for Pages deployment.

## Platform Classification

Each platform is classified as one of:

| Status | Meaning |
|--------|---------|
| `tested` | Bundled artifact exists, smoke-tested, checksum verified, manifest agrees |
| `built-but-unverified` | CI compiles binary, no bundled artifact, no smoke test |
| `build-feasibility-verified` | CI compiles binary, no bundled artifact, no smoke test, compilation proven only |
| `not-built` | Not produced in this release |
| `blocked` | Known blocker preventing build |

**Compile success is not platform support.** A platform must have a bundled artifact, checksum, smoke evidence, and matching manifest entry to be promoted.

## Known Limitations

- All artifacts are **unsigned beta builds**
- Code signing and notarization are planned for a future release
- Linux binary compiles in CI but no AppImage/DEB bundle is produced
- macOS binary compiles in CI but no DMG/app bundle is produced
- Full native gix migration remains incomplete
- Architect agent remains mock-mode
- ML-Master exploration loop remains stub
