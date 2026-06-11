# Platform Support Matrix — Orqestra v2.11.0

## Supported Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| **Windows x64** | ✅ Supported beta path | Installer available, tested |
| **macOS** | ⚠️ Not yet packaged | Source build only (`cargo tauri dev`) |
| **Linux** | ⚠️ Not yet packaged | Source build only (`cargo tauri dev`) |

## Windows x64

- Installer: `.exe` from [latest release](https://github.com/Octo-Lex/Orqestra/releases/latest)
- Installer is unsigned — Windows SmartScreen will warn. Click "More info" → "Run anyway".
- SHA-256 checksum available in release artifacts.

## macOS

- No packaged installer yet.
- To build from source:
  1. Install Rust, Node.js, and Xcode Command Line Tools
  2. Clone the repository
  3. Run `cargo tauri dev` in `apps/desktop/`

## Linux

- No packaged installer yet.
- To build from source:
  1. Install Rust, Node.js, and system dependencies (webkit2gtk, etc.)
  2. Clone the repository
  3. Run `cargo tauri dev` in `apps/desktop/`

## What We Do NOT Claim

- Production security certification on any platform
- Cross-platform binary stability
- macOS or Linux as tested beta paths
- Mobile or web application support
