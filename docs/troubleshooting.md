# Troubleshooting Orqestra Public Beta

This guide covers the most common issues encountered during Orqestra beta installation and usage.

---

## Windows SmartScreen Warning

### If the installer is unsigned

**What happened:** Windows shows "Windows protected your PC" when you run the installer.

**Why:** Orqestra installers are currently unsigned beta builds. Windows SmartScreen flags unknown/unsigned executables.

**What to try:**
1. Click **"More info"**
2. Click **"Run anyway"**

**How to verify the download is legitimate:** Check the SHA256 against `checksums.txt` in the release assets:
```powershell
Get-FileHash .\Orqestra_1.0.7_x64-setup.exe -Algorithm SHA256
```

**How to verify Authenticode signature:**
```powershell
Get-AuthenticodeSignature .\Orqestra_1.0.7_x64-setup.exe
```
Expected: `Status: NotSigned`

**Where to report it:** This is expected behavior. No report needed. See [Signing Plan](release-signing-plan.md) for the path to signed releases.

### If the installer is signed but still warned

**What happened:** A future signed installer still shows a SmartScreen warning.

**Why:** SmartScreen uses reputation-based detection. New or low-reputation signed applications may still be flagged until sufficient download history is established.

**What to try:**
1. Verify the signature is valid: `Get-AuthenticodeSignature .\Orqestra_*.exe`
2. If the signature is valid and from the expected publisher, click **"More info"** → **"Run anyway"**
3. Reputation builds over time as more users install the signed version

**Where to report it:** [Install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml) — include the signature status and publisher name.

### What to include in an install issue

- Whether SmartScreen appeared (yes/no)
- Whether you verified SHA256 (yes/no)
- Whether you verified the signature (yes/no/unsigned)
- Your Windows version
- A screenshot of the warning if possible

---

## Installer Download Blocked

**What happened:** Your browser blocks the `.exe` download.

**Why:** Browsers may flag unsigned executables from GitHub Releases.

**What to try:**
1. In Chrome: click the download arrow → "Keep"
2. In Edge: click "..." → "Keep" → "Show more" → "Keep anyway"
3. In Firefox: right-click the download → "Allow download"

**Where to report it:** If the download itself fails (not just a warning), open an [install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml).

---

## App Does Not Launch

**What happened:** You installed Orqestra but clicking the icon does nothing or shows an error.

**Why it may have happened:**
- Missing Visual C++ Runtime (rare on Windows 10/11)
- Antivirus software blocking the launch
- Corrupted installation

**What to try:**
1. Check Task Manager for `orqestra-desktop.exe` — it may be running but not visible
2. Right-click the shortcut → "Run as administrator"
3. Temporarily disable antivirus and try again
4. Reinstall using the latest installer from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases)
5. Open a terminal and run `"C:\Program Files\Orqestra\orqestra-desktop.exe"` to see error output

**Where to report it:** [Install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml) — include your Windows version and any error messages.

---

## Repository Does Not Open

**What happened:** You selected a folder but Orqestra shows "not a valid Orqestra project."

**Why it may have happened:**
- The folder does not contain a `roadmap/` directory
- Task files are missing the required `pm-task: true` frontmatter

**What to try:**
1. Make sure the folder has a `roadmap/` subdirectory
2. Each task file must start with:
   ```yaml
   ---
   pm-task: true
   id: TASK-001
   title: "My task"
   status: backlog
   ---
   ```
3. Use **"Try sample project"** in the onboarding wizard to see a working example
4. Check the [beta quickstart](beta-quickstart.md) for step-by-step instructions

**Where to report it:** [Bug report](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=bug_report.yml)

---

## Roadmap Does Not Load

**What happened:** You opened a repository but no tasks appear.

**Why it may have happened:**
- Task files lack `pm-task: true` in frontmatter
- Files are not `.md` format
- The `roadmap/` directory is empty

**What to try:**
1. Open the `roadmap/` folder in File Explorer and verify `.md` files exist
2. Open a task file in Notepad and verify the YAML frontmatter includes `pm-task: true`
3. Use the **Setup** panel to check project validation status
4. Try the sample project first to confirm the app works, then compare your file structure

**Where to report it:** [Bug report](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=bug_report.yml)

---

## Dashboard Looks Stale

**What happened:** The dashboard at [orqestra.pages.dev](https://orqestra.pages.dev) does not show current data.

**Why it may have happened:**
- Dashboard is deployed from CI — it updates when the `roadmap/` directory changes on `master`
- Your browser may be caching the old version

**What to try:**
1. Hard-refresh: `Ctrl+Shift+R`
2. Check the footer for the "Generated at" timestamp and "Source commit"
3. Compare the source commit with the latest commit on the [repository](https://github.com/Elephant-Rock-Lab/Orqestra/commits/master)

**Where to report it:** [Dashboard issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=dashboard_issue.yml)

---

## No-Key Beta Mode

**What it is:** The default mode. No API keys required. AI features show as "degraded" or "mock". All roadmap, PM view, dashboard, and diagnostic features work normally.

**What works without a key:**
- Roadmap parsing and all PM views (Table, Gantt, Kanban)
- Dashboard at orqestra.pages.dev
- Diagnostics export
- Project validation
- Credential management

**What does not work without a key:**
- Real AI proposed edits from docs-agent
- Real AI proposed fixes from bugfix-agent
- Intent extraction from commit diffs

**This is expected behavior.** No fix or report needed.

---

## Real-AI Maintainer Mode

**What it is:** Optional mode that enables real AI agent responses. Requires `ZAI_API_KEY`.

**How to enable:**
1. Create `services/ai/.env` with your key:
   ```
   ZAI_API_KEY=your-key-here
   ```
2. Start the AI service:
   ```bash
   cd services/ai
   uv run uvicorn orqestra_ai.main:app
   ```
3. Open Orqestra and check the **Setup** panel — AI should show "real-ai" mode

**Important:** All agent outputs are **review-only**. No autonomous commits occur.

---

## ZAI_API_KEY Not Detected

**What happened:** You set `ZAI_API_KEY` but the readiness panel still shows "degraded."

**Why it may have happened:**
- The key is not in `services/ai/.env` (the AI service loads from this file)
- The AI service was started before you set the key
- The key is in a system environment variable but the service reads `.env` first

**What to try:**
1. Verify `services/ai/.env` exists and contains `ZAI_API_KEY=...`
2. Restart the AI service: stop it (Ctrl+C) and run `uv run uvicorn orqestra_ai.main:app` again
3. Check the service health: open `http://localhost:8000/health` in a browser
4. If the key is only in your system env, copy it to `services/ai/.env`

**Where to report it:** [AI mode issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=ai_mode_issue.yml)

---

## AI Service Health Check Fails

**What happened:** The readiness panel shows "AI service unreachable" at `localhost:8000`.

**Why it may have happened:**
- The AI service is not running
- It is running on a different port
- A firewall is blocking localhost connections

**What to try:**
1. Start the service: `cd services/ai && uv run uvicorn orqestra_ai.main:app`
2. Verify it is running: open `http://localhost:8000/health` — should return `{"status":"ok"}`
3. If using a different port, note it is not currently configurable without code changes

**Where to report it:** [AI mode issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=ai_mode_issue.yml)

---

## Git Push/Pull Fails

**What happened:** You tried to sync a roadmap repository but got a Git error.

**Why it may have happened:**
- No GitHub PAT (Personal Access Token) stored
- The PAT has expired or lacks repository permissions
- The repository remote URL is incorrect

**What to try:**
1. Open the **Credentials** panel in Orqestra and save a GitHub PAT
2. Ensure the PAT has `repo` scope
3. Verify the remote URL: `git remote -v` in the repository directory
4. If using a new PAT format, ensure it starts with `github_pat_`

**Where to report it:** [Bug report](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=bug_report.yml)

---

## Where Logs Are Stored

Orqestra desktop logs are stored in:
- **Windows:** `%APPDATA%\com.elephantrocklab.orqestra\logs\`

AI service logs appear in the terminal where you ran `uvicorn`.

Diagnostics bundles can be exported from the **Diagnostics** panel inside the app.

---

## How to File a Useful Issue

1. Go to [Issues](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new/choose)
2. Pick the template that matches your problem
3. Include:
   - Your OS version
   - The Orqestra version (shown in the Setup panel)
   - Steps to reproduce
   - Any error messages or screenshots
4. **Do not paste API keys, tokens, or secrets** in the issue

## Linux AppImage Does Not Launch

**What happened:** The `.AppImage` file does not open when double-clicked or run from terminal.

**Why it may have happened:**
- The file is not marked executable
- Missing runtime dependencies (webkit2gtk, GTK3)
- AppImage FUSE support is not available

**What to try:**
1. Mark as executable: `chmod a+x Orqestra_1.0.9_x64.AppImage`
2. Run from terminal: `./Orqestra_1.0.9_x64.AppImage`
3. If FUSE errors appear, try extracting first: `./Orqestra_1.0.9_x64.AppImage --appimage-extract`
4. Install dependencies: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev`

**Where to report it:** [Install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml)

---

## Linux Permission Denied

**What happened:** `bash: ./Orqestra_1.0.9_x64.AppImage: Permission denied`

**What to try:**
1. `chmod a+x Orqestra_1.0.9_x64.AppImage`
2. If downloaded from browser, browsers may strip the execute bit

---

## Linux Missing Runtime Dependencies

**What happened:** AppImage fails with library errors (webkit2gtk, GTK, etc.).

**What to try:**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libappindicator3-dev
```

---

## Linux Running from Terminal for Logs

Run the AppImage from a terminal to capture error output:

```bash
./Orqestra_1.0.9_x64.AppImage 2>&1 | tee orqestra-launch.log
```

Include the terminal output when filing an issue.

---

## Quick links:
- [Install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml)
- [AI mode issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=ai_mode_issue.yml)
- [Dashboard issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=dashboard_issue.yml)
- [Bug report](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=bug_report.yml)
