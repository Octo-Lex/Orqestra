# Installer Diagnostics

This guide helps diagnose Orqestra installation and launch failures. If you are reporting an issue, follow the steps below and include the relevant information.

---

## Installer Artifact

1. Confirm you downloaded the correct file: `Orqestra_1.0.7_x64-setup.exe`
2. Source: [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases)
3. The installer is for **Windows x64** only

## Verify SHA256

```powershell
Get-FileHash .\Orqestra_1.0.7_x64-setup.exe -Algorithm SHA256
```

Compare the result with the hash in `checksums.txt` or `release-manifest.json` attached to the release. If the hash does not match, re-download the installer.

## Verify Signature

```powershell
Get-AuthenticodeSignature .\Orqestra_1.0.7_x64-setup.exe
```

Expected result for v1.0.7: `Status: NotSigned` — the installer is unsigned. This is expected.

If a future release is signed, the status will show `Valid` with publisher certificate information.

## Windows Version

Check your Windows version:

```powershell
[System.Environment]::OSVersion.Version
winver
```

Orqestra requires Windows 10 (10.0) or later. If you are on an older version, the app may not launch.

## SmartScreen State

If Windows SmartScreen appears:

1. Click **"More info"**
2. Verify the publisher shows as "Unknown" (expected for unsigned)
3. Click **"Run anyway"**

If SmartScreen does not appear at all, your system's SmartScreen may be disabled or the installer was already allowed.

## Installer Logs

NSIS installer logs are not automatically written to disk. If the installer fails:

1. Run the installer from a terminal to see error output:
   ```powershell
   .\Orqestra_1.0.7_x64-setup.exe
   ```
2. Check if the install directory was created: `C:\Program Files\Orqestra\`
3. Check if `orqestra-desktop.exe` exists in the install directory

## App Launch Logs

If the app installs but does not launch:

1. Run the executable directly from a terminal:
   ```powershell
   & "C:\Program Files\Orqestra\orqestra-desktop.exe"
   ```
2. Note any error messages in the terminal output
3. Check the app data directory for crash logs:
   ```powershell
   dir "$env:APPDATA\com.elephantrocklab.orqestra"
   ```

## Tauri/WebView Runtime Issues

Orqestra uses Tauri 2.x which requires Microsoft Edge WebView2 Runtime. On Windows 10/11 this is usually pre-installed. If the app shows a WebView error:

1. Verify WebView2 is installed:
   ```powershell
   Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BEB-136B1F9D4A41}" 2>$null
   ```
2. If not installed, download from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

## Repository Open Failures

If the app launches but cannot open a repository:

1. Verify the folder contains a `roadmap/` subdirectory
2. Verify task files are `.md` format with `pm-task: true` in YAML frontmatter
3. Try the sample project first: onboarding wizard → "Try sample project"
4. See [Troubleshooting](troubleshooting.md#repository-does-not-open) for details

## AI Service Setup Failures

If AI features show errors:

1. Check AI service health: `curl http://localhost:8000/health`
2. Verify `services/ai/.env` contains `ZAI_API_KEY=...`
3. Restart the AI service: `cd services/ai && uv run uvicorn orqestra_ai.main:app`
4. See [Troubleshooting](troubleshooting.md#zai_api_key-not-detected) for full steps

## Git Operation Failures

If Git push/pull fails from within the app:

1. Verify Git is installed: `git --version`
2. Save a GitHub PAT in the Credentials panel (requires `repo` scope)
3. Test manually: `git push` from the repository directory
4. See [Troubleshooting](troubleshooting.md#git-pushpull-fails) for details

---

## What to Attach to an Issue

When filing an [install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml):

- Windows version
- Installer filename and SHA256 verification result
- SmartScreen behavior (yes/no)
- Install result (success/failure)
- Launch result (success/failure/crash)
- Terminal output if the app fails to launch
- Screenshots of any error dialogs

## What Not to Attach

**Do not attach or paste:**
- API keys (ZAI_API_KEY, etc.)
- GitHub Personal Access Tokens
- `.env` files
- Certificate material
- Password files
- Any file containing `Bearer`, `ghp_`, `sk-`, `secret:`, `token:`

If you need to share a diagnostics bundle, use the **Diagnostics** panel inside the app — it automatically redacts all secrets.

## Privacy and Secrets Warning

Orqestra's diagnostics export redacts known secret patterns. If you manually copy logs or screenshots, review them for API keys, tokens, or credentials before posting publicly.
