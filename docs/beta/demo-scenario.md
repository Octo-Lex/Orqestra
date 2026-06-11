# Demo Scenario — Orqestra v2.11.0

**Audience:** External beta evaluators.
**Time:** 10–15 minutes.
**Platform:** Windows x64.

---

## Prerequisites

- Orqestra installed (see [beta-quickstart.md](../beta-quickstart.md))
- A local Git repository with Markdown roadmap files, OR use the built-in sample project

---

## Step 1: Launch and Onboard

1. Open Orqestra from the Start menu.
2. The onboarding wizard appears with a welcome screen.
3. Select **"Try sample project"** to use the built-in demo, or **"Open existing repo"** to point at your own project.

**Expected result:** App opens to the main project view.

---

## Step 2: View Tasks

1. The task table shows tasks from the roadmap files.
2. Click **Kanban** to switch to board view — tasks appear in status columns.
3. Click **Gantt** to switch to timeline view.
4. Click **Table** to return to the table view.

**Expected result:** All three views render with task data.

---

## Step 3: Check Environment Readiness

1. Open the **Setup** or **Readiness** panel.
2. Review the status indicators:

| Check | What It Means |
|-------|---------------|
| Git | Is Git installed and is this a valid repository? |
| Credentials | Is the OS keychain available for credential storage? |
| AI Service | Is the local AI service running? (localhost:8000) |
| Dashboard | Can dashboard evidence be exported? |

3. If any check shows a warning, follow the guidance text next to it.

**Expected result:** Each status is clear, with actionable next steps for any warnings.

---

## Step 4: Run the Docs Agent (if AI service is running)

> **Skip to Step 5 if the AI service is unavailable.**

1. Select a Markdown file in the file tree (e.g., a file in `docs/`).
2. Click **"Run docs agent"**.
3. The agent analyzes the file and returns a diff.
4. Review the diff in the diff viewer.
5. Click **Accept** to apply, or **Reject** to discard.

**Expected result:** Reviewable diff with accept/reject. No write happens without explicit acceptance.

---

## Step 5: AI Unavailable Mode (if AI service is NOT running)

1. The readiness panel shows **"AI Service: Unavailable"**.
2. The message reads: "Project management, roadmap views, Git history, and dashboard export remain available. Agent execution requires the local AI service (localhost:8000) to be running."
3. PM views remain fully functional.
4. No mock or fake output appears.

**Expected result:** Graceful degradation with clear guidance.

---

## Step 6: View Public Evidence

1. Switch to the **Evidence** tab (no token required).
2. Review the panels:
   - **Release History** — versions and release types
   - **Test Count Trend** — test counts over releases
   - **Security Boundaries** — security properties
   - **Autonomy Policy** — governed autonomy settings
   - **Runtime Evidence** — path decision matrix
   - **Data Freshness** — last evidence update time

**Expected result:** Six evidence panels render with static data. No live telemetry.

---

## Step 7: Export Diagnostics

1. Open the diagnostics panel or use the export command.
2. Export the diagnostics bundle.
3. Open the resulting directory.
4. Verify `beta-readiness-summary.json` is present.
5. Verify no files contain tokens, PATs, API keys, or raw secret strings.
6. Verify project paths are hashed (not raw paths).

**Expected result:** Redacted diagnostics bundle with beta readiness summary.

---

## Step 8: Verify Evidence Integrity

1. Check that `beta-readiness-summary.json` contains:
   - `readiness` field (not `beta_ready`)
   - `checks` with boolean status for each component
   - `warnings` listing any degraded features
   - `blocked_features` listing unavailable features
   - `repo` with `path_hash` (not raw path)

**Expected result:** Structured readiness summary, no unconditional "ready" when degraded.

---

## What This Demo Does NOT Cover

- Cloud sync (not yet implemented)
- Source-code auto-apply (governed, not autonomous)
- macOS / Linux builds (not yet packaged)
- Production security certification
