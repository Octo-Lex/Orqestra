# Troubleshooting — Orqestra Beta

## Common Issues

### Windows SmartScreen Warning

**Symptom:** Windows blocks the installer with "Windows protected your PC".

**Fix:** Click "More info" → "Run anyway". The installer is unsigned but checksum-verified.

---

### AI Service Unavailable

**Symptom:** Readiness panel shows "AI Service: Unavailable". Agent buttons are disabled or show degraded state.

**What this means:** The local AI service (localhost:8000) is not running.

**What still works:**
- Project management views (Table, Kanban, Gantt)
- Git history browsing
- Dashboard evidence export
- Diagnostics export

**What does not work:**
- Running docs, bugfix, or architect agents
- AI-powered backfill

**Fix:** Start the AI service. If you don't have the AI service configured, this is expected for beta evaluation — all other features remain available.

---

### No Roadmap Files Detected

**Symptom:** Task table is empty. Readiness shows "roadmap: not found".

**What this means:** The opened repository does not contain a `roadmap/` directory with Markdown files.

**Fix:**
1. Create a `roadmap/` directory in the repository root
2. Add Markdown files with task descriptions
3. Or use the sample project from the onboarding wizard

---

### OS Keychain Unavailable

**Symptom:** Credentials panel shows "Keychain: unavailable".

**What this means:** The OS keychain (Windows Credential Manager) could not be accessed.

**What this affects:** Credential storage for remote Git providers.

**What still works:** All read-only operations, local Git operations.

**Fix:** Ensure Windows Credential Manager service is running. Orqestra does not auto-fix credentials — this is a diagnostic only.

---

### Corrupt Project State

**Symptom:** App fails to load project state or shows unexpected behavior.

**What Orqestra does:** On next launch, the corrupt `app-state.json` is backed up (`.corrupt.bak`) and a fresh state is created.

**What is preserved:** Keychain credentials are never cleared by state recovery.

**Fix:** The app recovers automatically. If issues persist, export diagnostics and file a beta issue.

---

### Dashboard Shows "Evidence Unavailable"

**Symptom:** Evidence panels show fallback messages instead of data.

**What this means:** The dashboard was built without evidence data, or evidence files failed schema validation.

**Fix:** This is a build-time issue, not a user-facing fix. Report via beta issue tracker.

---

## Exporting Diagnostics

If you encounter an issue not listed here:

1. Open the diagnostics panel
2. Click "Export Diagnostics"
3. The bundle is saved to `.Orqestra/orqestra-diagnostics-<timestamp>/`
4. Review the bundle before sharing — secrets are redacted, but verify
5. Share the bundle with the beta feedback channel

**The diagnostics bundle never contains:** tokens, PATs, API keys, raw secret strings, or unhashed project paths.
