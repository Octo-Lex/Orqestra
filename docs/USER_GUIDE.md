# Orqestra User Guide

## What is Orqestra?

Orqestra is a local-first, AI-native development environment that turns a Git repository into a project-management workspace with semantic history, agent-assisted development, and optional public dashboard.

## Getting Started

### First Launch

When you launch Orqestra for the first time, you'll see the onboarding wizard:

1. **Welcome** — Overview of what Orqestra does
2. **Choose Project** — Open an existing repository or create a sample project
3. **Environment Readiness** — See which integrations are configured
4. **Open Workspace** — Start using the app

### Opening a Project

Click **Open existing project** to select a folder. Orqestra validates the folder structure:

| Status | Meaning |
|--------|---------|
| Valid | Ready to use — roadmap tasks will load |
| Repairable | Mostly valid — optional files are missing |
| Not Orqestra | No roadmap/ directory found |
| Invalid | Required files are malformed |

### Sample Project

Click **Try sample project** to generate a demo with:
- 4 tasks in different states (backlog, in-progress, done)
- Dependencies between tasks
- Labels for agent routing
- Sample source files for agent demos

## Views

### Table View
Default view showing all tasks in a sortable table with columns for status, priority, assignee, sprint, and dates.

### Gantt View
Timeline visualization showing task durations, dependencies, and scheduling.

### Kanban View
Drag-and-drop board with columns for each status. Drag tasks between columns to update their status.

## AI Features

### Docs Agent
Generates documentation suggestions in **review-only mode**. Proposes edits that you can accept or reject.

**Requires:** Running AI service + `ZAI_API_KEY` environment variable

### Bugfix Agent
Reviews selected files and proposes fixes. **Review-only mode** — cannot auto-commit.

**Requires:** Running AI service + `ZAI_API_KEY` + user-selected file scope

### AI Modes

| Mode | Description |
|------|-------------|
| Real AI | Service reachable + API key configured |
| Degraded/Mock | Service reachable but no API key — returns structured fallbacks |
| Unavailable | Service not running — AI features disabled |

## Setup & Readiness

Open the **Setup** panel from the toolbar to see:
- Local tool availability (git, node, python, etc.)
- AI service status and API key configuration
- GitHub credential storage status
- Dashboard deployment status

## Credentials

GitHub Personal Access Tokens are stored in your OS keychain (Windows Credential Manager on Windows). Tokens are never stored in plain text files or sent to external services.

## Diagnostics

Open the **Diagnostics** panel to:
1. View environment status
2. Export a diagnostic bundle (ZIP with all secrets redacted)
3. See recovery advice for common issues

## Dashboard

The public dashboard is deployed at [orqestra.pages.dev](https://orqestra.pages.dev). It shows the project roadmap with Gantt and Kanban views behind a token gate.

### Deployment

To auto-deploy the dashboard via CI:
1. Add `CLOUDFLARE_API_TOKEN` as a GitHub repository secret
2. Add `CLOUDFLARE_ACCOUNT_ID` as a GitHub repository secret
3. Push to `master` or trigger the workflow manually

## Known Limitations

| Feature | Status |
|---------|--------|
| Roadmap parsing | Implemented and verified |
| Desktop PM views | Implemented and verified |
| Dashboard | Implemented and deployed |
| OS keychain credentials | Implemented and verified |
| Docs agent | Implemented, review-only |
| Bugfix agent | Implemented, review-only |
| Architect agent | **Mock-mode** — not production |
| ML-Master exploration | **Stub/backlog** — not available |
| Edge relay / CRDT sync | **Backlog** — not available |
| Full native Git | **Partial** — some shell-outs remain |
| AST code analysis | **Backlog** — not available |
