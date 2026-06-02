# First Run Guide

## Quick Start

1. **Launch Orqestra** — The onboarding wizard appears automatically
2. **Choose "Try sample project"** — Generates a demo with 4 tasks
3. **Review readiness** — See which integrations are configured
4. **Click "Open Workspace"** — Start exploring

That's it! You can now:
- Switch between **Table**, **Gantt**, and **Kanban** views
- Click on tasks to see details
- Try the **Docs agent** on sample files
- Export **diagnostics** to verify everything works

## Opening Your Own Project

1. Click **"Open existing project"**
2. Select a folder containing a `roadmap/` directory
3. Orqestra validates the structure and loads your tasks

### What Makes a Valid Project?

```
my-project/
├── roadmap/
│   ├── _index.md          # Optional coordinator
│   ├── TASK-001.md         # Task files with YAML frontmatter
│   └── TASK-002.md
├── Orqestra.toml           # Optional config
└── .Orqestra/              # Optional local metadata
```

Each task file needs YAML frontmatter:

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

## Optional Setup

### AI Service (Optional)
- Install Python 3.11+ and `uv`
- Run `cd services/ai && uv run uvicorn orqestra_ai.main:app --port 8000`
- Set `ZAI_API_KEY` environment variable for real AI output
- Without the key, agents work in **mock/fallback mode**

### GitHub Integration (Optional)
- Open **Settings** in the app
- Enter a GitHub Personal Access Token
- Enables push/pull for roadmap sync

### Dashboard Deployment (Optional)
- Add Cloudflare secrets to GitHub repository
- Push to master to trigger CI deployment
- Dashboard available at `orqestra.pages.dev`

## What Works Without Any Setup

- Local project management (Table, Gantt, Kanban)
- Task status updates via drag-and-drop
- Sample project generation
- Diagnostics export
- Project validation

## Getting Help

- Open **Diagnostics** panel for troubleshooting
- Export a diagnostic bundle for support
- Check the **Setup** panel for environment status
