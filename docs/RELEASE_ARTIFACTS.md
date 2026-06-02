# Release Artifacts

## Unsigned Beta Warning

Orqestra v1.0.4 desktop artifacts are **unsigned beta builds**. Your operating system may show a warning before launch.

- **Windows:** SmartScreen may block the installer. Click "More info" → "Run anyway".
- **macOS:** Gatekeeper will block the app. Right-click → Open to bypass.
- **Linux:** No signing infrastructure needed for AppImage.

Code signing and notarization are planned for a future production release.

## v1.0.5 Artifacts

| Platform | Artifact | Status | Notes |
|----------|----------|--------|-------|
| Windows x64 | `Orqestra_1.0.5_x64-setup.exe` | tested | NSIS installer, unsigned public beta |
| macOS Apple Silicon | `.dmg` | not-built | Deferred to future release |
| macOS Intel | `.dmg` | not-built | Deferred to future release |
| Linux x64 | `.AppImage` / `.deb` | built-but-unverified | CI builds, not locally validated |

## Downloading

Download from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases).

## Installing

### Windows
1. Download `Orqestra_1.0.3_x64-setup.exe`
2. Run the installer
3. Launch Orqestra from Start Menu

**Note:** The installer is unsigned. Windows SmartScreen may show a warning. Click "More info" → "Run anyway" to proceed.

### macOS
1. Download the `.dmg` file
2. Open the DMG
3. Drag Orqestra to Applications folder
4. On first launch: right-click → Open (to bypass Gatekeeper)

### Linux
1. Download the `.AppImage`
2. `chmod +x Orqestra-*.AppImage`
3. `./Orqestra-*.AppImage`

## Building from Source

See README.md for developer setup instructions.

## Artifact Manifest

Each release includes `release-manifest.json` with per-platform entries:

```json
{
  "version": "1.0.4",
  "tag": "v1.0.4",
  "commit": "<sha>",
  "built_at": "2026-06-02T00:00:00Z",
  "artifacts": [
    {
      "platform": "windows-x64",
      "kind": "nsis-installer",
      "path": "target/release/bundle/nsis/Orqestra_1.0.4_x64-setup.exe",
      "status": "tested",
      "signed": false,
      "sha256": "<sha256>"
    }
  ],
  "warnings": ["Artifacts are unsigned beta builds."]
}
```

## Dashboard Deployment

The live dashboard at [orqestra.pages.dev](https://orqestra.pages.dev) is deployed via CI.

The dashboard deployment workflow passes Cloudflare `accountId` explicitly. This avoids relying on account discovery through the Cloudflare memberships API and prevents deployment failures when the API token is scoped only for Pages deployment.

## Platform Classification

Each platform is classified as one of:

| Status | Meaning |
|--------|--------|
| `tested` | Built locally, smoke-tested, artifact verified |
| `built-but-unverified` | Built in CI, no local validation |
| `not-built` | Not produced in this release |
| `blocked` | Known blocker preventing build |

## Known Limitations

- All artifacts are **unsigned beta builds**
- Code signing and notarization are planned for a future release
- macOS artifacts are not built (requires bundler target configuration)
- Linux artifacts are CI-built but unverified locally
- Full native gix migration remains incomplete
- Architect agent remains mock-mode
- ML-Master exploration loop remains stub
