# Self-Serve Beta Checklist

**Version:** v2.11.0
**Purpose:** Canonical smoke path for external beta validation.

---

## Pre-conditions

- Windows x64 machine
- Downloaded installer from [latest release](https://github.com/Octo-Lex/Orqestra/releases/latest)
- No contributor-level knowledge of the Orqestra codebase

---

## Checklist

### 1. Install and Launch

- [ ] Installer runs without error
- [ ] Application opens to onboarding wizard
- [ ] Onboarding wizard explains what Orqestra does

### 2. Open a Repository

- [ ] Can select a local Git repository
- [ ] Or: can use the sample project from the wizard
- [ ] App detects whether the repo contains roadmap files

### 3. Roadmap and Project Management

- [ ] If roadmap files exist: task table renders with tasks
- [ ] Can switch between Table, Kanban, and Gantt views
- [ ] If no roadmap files: guidance message explains what to do

### 4. Environment Readiness

- [ ] Readiness panel shows Git status (installed/not installed)
- [ ] Readiness panel shows credential vault status
- [ ] Readiness panel shows AI service status (connected/unavailable)
- [ ] Each status has clear next-step guidance

### 5. AI Agent Flow (if AI service available)

- [ ] Can invoke the docs agent on a docs file
- [ ] Agent returns a reviewable diff
- [ ] Can accept or reject the diff
- [ ] No accidental write without explicit accept

### 6. AI Degraded Mode (if AI service unavailable)

- [ ] AI status shows "Unavailable" (not "Connected")
- [ ] Clear message: "Agent execution requires the local AI service to be running"
- [ ] Project management, roadmap views, Git history, and dashboard export remain usable
- [ ] No mock or fake agent output appears

### 7. Dashboard Export

- [ ] Can export dashboard data (public evidence surface)
- [ ] Evidence tab shows release history, test counts, security boundaries, autonomy policy, runtime evidence
- [ ] Evidence data is static (no live telemetry)

### 8. Diagnostics Export

- [ ] Can export a diagnostics bundle
- [ ] Bundle includes beta-readiness-summary.json
- [ ] Bundle does not contain tokens, PATs, API keys, or raw secret strings
- [ ] Project paths are hashed, not raw

### 9. Recovery

- [ ] If the app crashes and restarts, previous project state is restored
- [ ] If project state is corrupt, app recovers with a backup and fresh state
- [ ] Reset onboarding does not clear keychain credentials

---

## What This Checklist Does NOT Cover

- macOS or Linux (not yet packaged)
- Cloud real-time sync (not yet implemented)
- Source-code auto-apply by AI agents
- Cross-agent orchestration
- Production security certification
