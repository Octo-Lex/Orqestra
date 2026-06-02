# Orqestra v1.0.3 Demo Script

This script provides a deterministic demo path for external reviewers.

## Prerequisites

- Orqestra installed or launched from source
- No prior setup required (sample project works offline)

## Steps

### 1. Launch Application
Start Orqestra. The onboarding wizard should appear automatically.

**Expected:** Welcome screen with three options.

### 2. Review Welcome Screen
Read the one-sentence description and note the distinction between local and optional features.

**Expected:** Clear copy explaining Orqestra works locally.

### 3. Create Sample Project
Click **"Try sample project"**.

**Expected:** Project created at `~/Orqestra-Sample/` with 4 tasks.

### 4. Verify Project Validation
The app should show the project validated as `valid`.

**Expected:** Green checkmark, task count = 4.

### 5. Review Readiness Panel
Click through to the readiness step. Review local tools, AI status, credentials.

**Expected:**
- Git: found
- AI: degraded or unavailable (expected without setup)
- GitHub token: missing (expected without setup)
- Dashboard: shows live URL status

### 6. Open Workspace
Click **"Open Workspace"** to enter the main app.

**Expected:** Task table loads showing 4 sample tasks.

### 7. Switch Views
Use the view switcher to change between:
- **Table** — sortable task list
- **Gantt** — timeline with dependencies
- **Kanban** — drag-and-drop status columns

**Expected:** All three views render the 4 sample tasks correctly.

### 8. Open a Task
Click on a task row to see its details.

**Expected:** Task title, status, priority, description visible.

### 9. Kanban Status Change
Switch to Kanban view. Drag a task from one column to another.

**Expected:** Task status updates and persists.

### 10. Setup Panel
Click **"Setup"** in the toolbar.

**Expected:** Readiness panel with all check cards visible.

### 11. Diagnostics Export
Click **"Diagnostics"** in the toolbar. Click **"Export Diagnostics"**.

**Expected:**
- Bundle created at `.Orqestra/orqestra-diagnostics-*/`
- Shows file count and redaction summary
- `contains_raw_secrets: false`

### 12. Verify Redaction
Open the diagnostic bundle. Search for any known secret patterns.

**Expected:** No `ghp_`, `sk-`, `Bearer `, or raw key values found.

### 13. Recovery Cards
Scroll to the **Common Issues** section in the Diagnostics panel.

**Expected:** Cards for ROADMAP_NOT_FOUND, AI_KEY_MISSING, etc. with recovery advice.

### 14. Feature-State Table
Open `README.md` in the repository. Find the **Feature-State Table**.

**Expected:** Each feature classified as "Implemented", "Mock-mode", or "Backlog".

### 15. Dashboard (Optional)
If internet is available, visit [orqestra.pages.dev](https://orqestra.pages.dev).

**Expected:** Dashboard loads with Gantt and Kanban views behind token gate.

## Cleanup

To reset onboarding state for another demo:
- Delete the `Orqestra-Sample/` directory
- Click **"Run setup wizard"** from the project picker

## Passing Criteria

The demo is successful if:
- [x] Onboarding wizard appears on first launch
- [x] Sample project creates and validates
- [x] All three PM views render tasks
- [x] Readiness panel shows environment status
- [x] Missing AI/cloud setup is a warning, not a blocker
- [x] Diagnostics export produces redacted bundle
- [x] README feature-state table is truthful
