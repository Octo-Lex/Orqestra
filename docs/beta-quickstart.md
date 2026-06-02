# Orqestra Public Beta Quickstart

**Audience:** Technical reviewers evaluating the Orqestra public beta.

---

## 1. Download

Download `Orqestra_1.0.6_x64-setup.exe` from the [latest release](https://github.com/Elephant-Rock-Lab/Orqestra/releases/latest).

Windows x64 is the only tested platform for this beta.

## 2. Verify

Open PowerShell and run:

```powershell
Get-FileHash .\Orqestra_1.0.6_x64-setup.exe -Algorithm SHA256
```

Compare the hash against `checksums.txt` or `release-manifest.json` attached to the release.

## 3. Install

Run the installer. The installer is **unsigned** — Windows SmartScreen will warn you. Click "More info" → "Run anyway".

See [Troubleshooting](troubleshooting.md#windows-smartscreen-warning) if you get a download or install block.

## 4. Launch

Open Orqestra from the Start menu. The onboarding wizard appears.

## 5. Try the Sample Project

1. In the onboarding wizard, click **"Try sample project"**
2. A demo project with 4 tasks is generated
3. Switch between **Table**, **Gantt**, and **Kanban** views
4. Open the **Setup** panel to see environment status

## 6. Check the Dashboard

Open [orqestra.pages.dev](https://orqestra.pages.dev) in a browser. Verify it shows current roadmap data. Check the footer for the generation timestamp and source commit.

## 7. Try No-Key Beta Mode

No API key is needed. In this mode:
- Roadmap parsing works
- All PM views work
- Dashboard works
- AI features show as "degraded" — this is correct and expected
- Diagnostics export works

See the **Setup** panel for the full readiness report.

## 8. Optional: Real-AI Maintainer Mode

If you have a `ZAI_API_KEY`:

1. Create `services/ai/.env` with `ZAI_API_KEY=your-key`
2. Start the AI service: `cd services/ai && uv run uvicorn orqestra_ai.main:app`
3. Restart Orqestra and check the **Setup** panel — AI should show "real-ai" mode
4. Try the docs-agent or bugfix-agent — both produce review-only proposals

**All AI outputs are review-only.** No autonomous commits occur.

## 9. Open Your Own Project

1. Click **"Open existing project"** in the onboarding wizard
2. Select a folder with a `roadmap/` directory containing task `.md` files
3. Each task needs YAML frontmatter with `pm-task: true`

```yaml
---
pm-task: true
id: TASK-001
title: "My task"
status: backlog
priority: High
created: "2026-06-01T00:00:00Z"
updated: "2026-06-01T00:00:00Z"
---
Task description here.
```

## 10. Export Diagnostics

Open the **Diagnostics** panel and click **Export Diagnostics Bundle**. The bundle is automatically redacted — no secrets are included.

## 11. Report Feedback

- [Install issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=install_issue.yml)
- [AI mode issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=ai_mode_issue.yml)
- [Dashboard issue](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=dashboard_issue.yml)
- [Bug report](https://github.com/Elephant-Rock-Lab/Orqestra/issues/new?template=bug_report.yml)

**Do not paste API keys or secrets in issues.**

---

## What This Beta Does Not Include

- Signed installer (SmartScreen warnings are expected)
- macOS artifacts
- Verified Linux artifacts
- Autonomous agent commits (all AI is review-only)
- Architect agent (mock-mode)
- ML-Master exploration (stub)
- CRDT real-time sync (backlog)

See the full [Known Limitations](../README.md#known-limitations) in the README.
