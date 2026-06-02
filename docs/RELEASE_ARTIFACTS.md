# Release Artifacts

## v1.0.3 Artifacts

| Platform | Artifact | Status | Notes |
|----------|----------|--------|-------|
| Windows x64 | `Orqestra_1.0.3_x64-setup.exe` | Available | NSIS installer, unsigned beta |
| macOS Apple Silicon | `.dmg` | CI-dependent | Requires macOS runner |
| macOS Intel | `.dmg` | CI-dependent | Separate or universal build |
| Linux x64 | `.AppImage` or `.deb` | CI-dependent | Requires Tauri system dependencies |

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

Each release includes `release-artifacts.json`:

```json
{
  "version": "1.0.3",
  "git_sha": "<sha>",
  "built_at": "2026-06-02T00:00:00Z",
  "platform": "windows-x64",
  "artifact": "Orqestra_1.0.3_x64-setup.exe",
  "signed": false,
  "known_limitations": ["Unsigned beta artifact"]
}
```

## Known Limitations

- All artifacts are **unsigned beta builds**
- Code signing and notarization are planned for a future release
- macOS and Linux artifacts depend on CI runner availability
