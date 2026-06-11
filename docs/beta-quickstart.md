# Orqestra Public Beta Quickstart

**Audience:** Technical reviewers evaluating the Orqestra public beta.
**Version:** v2.14.1

---

## 1. Download

Download the latest installer from the [latest release](https://github.com/Octo-Lex/Orqestra/releases/latest).

Windows x64 is the only tested platform for this beta. See [platform support](beta/platform-support.md) for macOS/Linux status.

## 2. Verify

Open PowerShell and run:

```powershell
Get-FileHash .\Orqestra_*_x64-setup.exe -Algorithm SHA256
```

Compare the hash against `checksums.txt` or `release-manifest.json` attached to the release.

## 3. Install

Run the installer. The installer is **unsigned** — Windows SmartScreen will warn you. Click "More info" → "Run anyway".

See [Troubleshooting](beta/troubleshooting.md#windows-smartscreen-warning) if you get a download or install block.

## 4. Launch

Open Orqestra from the Start menu. The onboarding wizard appears.

## 5. Try the Sample Project

1. In the onboarding wizard, click **"Try sample project"**
2. A demo project with tasks is generated
3. Switch between **Table**, **Gantt**, and **Kanban** views
4. Open the **Readiness** panel to see environment status

## 6. Check Environment Readiness

The readiness panel shows the status of:

| Component | What It Checks |
|-----------|---------------|
| Git | Installed, repo detected, branch, working tree state |
| Credentials | OS keychain available |
| AI Service | localhost:8000 reachable |
| Dashboard | Export capability |

Each check has clear guidance for any issues.

## 7. Check the Dashboard

Open [orqestra.pages.dev](https://orqestra.pages.dev) in a browser. The **Evidence** tab shows:

- Release history
- Test count trends
- Security boundaries
- Autonomy policy
- Runtime evidence
- Data freshness

No token required. Evidence is static (build-time, not live telemetry).

## 8. AI Modes

### No AI Service (default)

Without the AI service running:
- All PM views work (Table, Kanban, Gantt)
- Git history works
- Dashboard export works
- Diagnostics export works
- AI agent features show as **"Unavailable"** with clear guidance
- **No mock or fake output** appears

This is the expected state for most beta evaluators.

### With AI Service

If you have access to the AI service:

1. Start the AI service on localhost:8000
2. Check the readiness panel — AI should show "Connected"
3. Run the docs-agent on a Markdown file
4. Review the diff in the diff viewer
5. Accept or reject — no write happens without explicit acceptance

**All AI outputs are review-only. `auto_commit` is always false.**

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

Open the **Diagnostics** panel and click **Export Diagnostics Bundle**.

The bundle includes:
- App version and platform info
- Environment readiness report
- Git provider diagnostics
- AI health check
- Credential status
- Agent matrix
- Beta readiness summary
- Patch governance status
- Roadmap status

**The bundle never contains:** tokens, PATs, API keys, raw secret strings, or unhashed project paths.

## 11. Report Feedback

- [Bug report](https://github.com/Octo-Lex/Orqestra/issues/new?template=bug_report.yml)

**Do not paste API keys or secrets in issues.** Export diagnostics and attach the redacted bundle instead.

---

## What This Beta Does Not Include

- Signed installer (SmartScreen warnings are expected)
- macOS artifacts (source build only)
- Verified Linux artifacts (source build only)
- Autonomous agent commits (all AI is review-only, `auto_commit` is always false)
- Cloud real-time sync (not yet implemented)
- Production security certification

See the full [demo scenario](beta/demo-scenario.md) for a step-by-step walkthrough.
