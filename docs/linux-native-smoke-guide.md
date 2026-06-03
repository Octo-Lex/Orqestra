# Native Linux AppImage Smoke Guide

This guide helps contributors run the Orqestra Linux AppImage smoke test on a native Linux desktop.

## Purpose

Orqestra is a desktop application built with Tauri (Rust + WebView). The Linux AppImage is produced by CI but needs native Linux desktop testing to verify it works outside the CI environment.

Completing this guide and submitting evidence will help promote Linux to a tested platform.

## Required Environment

You need a **native Linux desktop** with a graphical session (X11 or Wayland).

**Recommended:** Ubuntu 24.04 GNOME

**Also supported:** Fedora Workstation, Debian desktop, Linux Mint, or any distro with:
- GTK 3.24+
- WebKit2GTK 4.1+
- FUSE 3+

**Not suitable for smoke testing:**
- WSL2 / WSLg
- SSH-only servers
- Docker containers without desktop
- CI environments

## Download the AppImage

1. Go to the [latest release](https://github.com/Elephant-Rock-Lab/Orqestra/releases/latest)
2. Download `Orqestra_1.0.12_x64.AppImage`
3. Download `checksums.txt`

## Verify SHA256

```bash
sha256sum -c checksums.txt --ignore-missing
```

Expected output: `Orqestra_1.0.12_x64.AppImage: OK`

Alternatively:

```bash
sha256sum Orqestra_1.0.12_x64.AppImage
```

Compare the output against the SHA256 in `checksums.txt` and `release-manifest.json`.

## Make the AppImage Executable

```bash
chmod a+x Orqestra_1.0.12_x64.AppImage
```

## Launch from Terminal

```bash
./Orqestra_1.0.12_x64.AppImage
```

Record the terminal output. You may see non-fatal warnings like:
- `libEGL warning: DRI3 error` -- software rendering fallback, non-fatal
- `Gtk-WARNING` about theme -- cosmetic, non-fatal

## Record Runtime Environment

Run these commands and include the output in your report:

```bash
echo "Distribution: $(cat /etc/os-release | grep PRETTY_NAME | cut -d= -f2 | tr -d '"')"
echo "Kernel: $(uname -r)"
echo "Architecture: $(uname -m)"
echo "Desktop: $XDG_CURRENT_DESKTOP"
echo "Display server: $XDG_SESSION_TYPE"
echo "GTK: $(dpkg -l libgtk-3-0t64 2>/dev/null | tail -1 | awk '{print $3}' || rpm -q gtk3 2>/dev/null | head -1)"
echo "WebKit2GTK: $(dpkg -l libwebkit2gtk-4.1-0 2>/dev/null | tail -1 | awk '{print $3}' || rpm -q webkit2gtk4.1 2>/dev/null | head -1)"
echo "FUSE: $(dpkg -l libfuse3-3 2>/dev/null | tail -1 | awk '{print $3}' || rpm -q fuse3-libs 2>/dev/null | head -1)"
echo "Machine: physical / VM / cloud"
```

## Open an Orqestra Repository

1. When the app launches, it shows a folder picker
2. Select the Orqestra repository folder (or any folder with `.md` files)
3. Confirm the app loads the file list

## Verify Roadmap UI

1. Check that the roadmap/task list appears
2. Click on a task to view its details
3. Confirm the UI is responsive and readable

## Verify Dashboard Link

1. Look for the dashboard link in the app
2. Click it
3. Confirm it opens https://orqestra.pages.dev in your browser
4. Confirm the dashboard shows content

## Verify No-Key Beta Mode

1. The app should work without entering any API keys
2. Confirm you can browse tasks and views without authentication
3. Confirm no errors appear related to missing keys

## Relaunch Test

1. Close the app completely
2. Launch the AppImage again
3. Confirm it starts successfully the second time

## Evidence to Capture

Please fill out the [Linux Smoke Evidence Template](linux-smoke-evidence-template.md) and submit it via the [Linux Smoke Report](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=linux_smoke_report.yml) issue form.

**Required:**
- Filled evidence template
- Terminal output (with secrets removed)
- At least one screenshot of the main window

## What Counts as Pass

All 9 smoke steps completed:
1. chmod executable -- pass
2. Launch from terminal -- pass
3. App window opens -- pass
4. Repository opens -- pass
5. Roadmap/task UI loads -- pass
6. Dashboard link opens -- pass
7. Dashboard shows release metadata -- pass
8. No-key beta mode works -- pass
9. Close and relaunch -- pass

## What Counts as Fail

Any smoke step fails with a specific error. Include:
- The exact error message
- Terminal output
- Screenshots of any error dialogs

## What Counts as Blocked

You cannot complete the smoke test due to environment constraints:
- Missing runtime dependencies (WebKit2GTK, GTK, FUSE)
- Display server issues
- AppImage won't mount (FUSE not available)

Report these too -- they help improve the troubleshooting guide.

## What Not to Share

**Do NOT include in your report:**
- API keys or tokens
- GitHub personal access tokens
- `.env` file contents
- Private repository contents
- Screenshots containing visible secrets or tokens
- Passwords

Remove or redact any secrets before submitting.
