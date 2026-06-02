# Orqestra v1.0.4 Demo Script

**Tag:** v1.0.4  
**Purpose:** Deterministic walkthrough for external reviewers  

---

## Mode A — No-Key Beta Demo (default reviewer path)

This mode requires **no API keys** and works out of the box.

### 1. Install and Launch

1. Download `Orqestra_1.0.4_x64-setup.exe` from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases)
2. Run the installer (unsigned beta — Windows may show a warning)
3. Launch Orqestra from the Start Menu

### 2. First-Run Onboarding

1. The onboarding wizard appears automatically
2. **Welcome** step — overview of Orqestra capabilities
3. **Project** step — choose "Try sample project"
4. **Readiness** step — view environment status:
   - Git: found
   - Node/npm: found
   - Rust/Cargo: found
   - Python: found
   - AI: degraded (no API key) — **expected and correct**
   - GitHub token: not stored — **expected and correct**

### 3. Explore the Sample Project

1. The sample project opens with 4 demo tasks
2. Switch between **Table**, **Gantt**, and **Kanban** views
3. Verify all three views render correctly

### 4. Setup Panel

1. Open the **Setup** panel (gear icon)
2. Verify **Readiness** shows all tool statuses
3. Verify AI shows "degraded" mode
4. Verify each card shows correct status

### 5. Live Dashboard

1. Open [orqestra.pages.dev](https://orqestra.pages.dev) in a browser
2. Verify the page returns HTTP 200
3. Verify the dashboard shows roadmap data with task counts

### 6. Diagnostics Export

1. Open the **Diagnostics** panel
2. Click **Export Diagnostics Bundle**
3. Save the bundle to a local folder
4. Open the exported bundle files
5. **Verify no secrets appear** — no API keys, no PATs, no Bearer tokens

### 7. Summary

At this point, a reviewer has verified:
- App installs and launches without a dev server
- Onboarding wizard works end-to-end
- Sample project generates correctly
- All three PM views render
- AI mode is correctly labeled "degraded" without a key
- Dashboard is live and accessible
- Diagnostics export is redacted

---

## Mode B — Real-AI Maintainer Demo

This mode requires `ZAI_API_KEY` set in `services/ai/.env`.

### 1. Start AI Service

```bash
cd services/ai
uv run uvicorn orqestra_ai.main:app --port 8000
```

### 2. Verify Real AI

1. Open the **Readiness** panel
2. Verify AI shows "real-ai" mode (not degraded)
3. Verify model is "glm-5.1"

### 3. Docs Agent

1. Open a project with a `docs/` or `README.md` file
2. Navigate to the **Agent** panel
3. Select **Docs Agent**
4. Provide a task (e.g., "Update README tagline")
5. Click **Run**
6. **Review the proposed diff** — verify it is a suggestion, not auto-committed
7. Accept or reject the proposal manually

### 4. Bugfix Agent

1. Navigate to the **Agent** panel
2. Select **Bugfix Agent**
3. Select specific files for scope (e.g., a source file with a known issue)
4. Provide a bug description
5. Click **Run**
6. **Review the proposed fix** — verify it stays within user-selected file scope
7. Verify **no autonomous commit occurred**

### 5. Summary

At this point, a reviewer has verified:
- AI service connects with real model when key is present
- Docs agent produces real proposed edits
- Bugfix agent produces real proposed edits within scope
- All AI outputs are review-only (no auto-commit)
- ConfidenceGate enforces propose/review-only policy

---

## What Is NOT Tested in This Demo

These features remain backlog or mock-mode:

- Architect agent (mock-mode only)
- ML-Master exploration loop (stub)
- Edge relay / CRDT sync (not available)
- Code signing / notarization (unsigned beta)
- macOS / Linux desktop artifacts (not built or not verified)
