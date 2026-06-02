# Orqestra

**Local-first, AI-native project management for Git repositories.**

Orqestra turns a Git repository into a structured workspace with roadmap tracking, semantic history, AI-assisted code review, and an optional public dashboard — all running locally.

## Try It

### Install (Windows)

Download `Orqestra_1.0.3_x64-setup.exe` from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases).

> The installer is unsigned. Windows SmartScreen may show a warning — click "More info" → "Run anyway".

### First Launch

1. Launch Orqestra — the onboarding wizard appears
2. Click **"Try sample project"** — generates a demo with 4 tasks
3. Explore Table, Gantt, and Kanban views
4. Check **Setup** panel for environment status

### Open Your Own Project

1. Click **"Open existing project"**
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

## What Works in v1.0.3

| Feature | Status | Notes |
|---------|--------|-------|
| Roadmap parsing | Implemented and verified | Local |
| Desktop PM views | Implemented and verified | Table, Gantt, Kanban |
| Dashboard | Deployed at [orqestra.pages.dev](https://orqestra.pages.dev) | Token-gated |
| OS keychain credentials | Implemented and verified | Windows Credential Manager |
| Docs agent | Implemented, review-only | Proposes edits for approval |
| Bugfix agent | Implemented, review-only | User-selected files only |
| First-run onboarding | Implemented | Guided wizard with sample project |
| Environment readiness | Implemented | Setup checks for all integrations |
| Project validation | Implemented | Validates folder before loading |
| Diagnostics export | Implemented | Redacted bundle, no raw secrets |

## What Requires Setup

| Integration | Setup | Enables |
|-------------|-------|---------|
| `ZAI_API_KEY` env var | Set before launch | Real AI output (agents work in mock mode without it) |
| Python AI service | `cd services/ai && uv run uvicorn orqestra_ai.main:app` | Docs/bugfix agent real calls |
| GitHub PAT | Settings → Save token | Push/pull for roadmap sync |
| Cloudflare secrets | GitHub repo → Actions secrets | Dashboard CI auto-deploy |

## What Is NOT Done

These features remain backlog or mock-mode:

- **Architect agent** — Mock-mode, not production
- **ML-Master exploration** — Stub, not implemented
- **Edge relay / CRDT sync** — Not available
- **Full native Git** — 9 shell-outs remain (commit creation is native gix)
- **AST code analysis** — Not started
- **Code signing** — Artifacts are unsigned beta builds

## Diagnostics

Open the **Diagnostics** panel to export a redacted support bundle. All secrets are automatically stripped before the bundle leaves the app. See [docs/DIAGNOSTICS.md](docs/DIAGNOSTICS.md).

## Documentation

| Document | Description |
|----------|-------------|
| [User Guide](docs/USER_GUIDE.md) | Complete usage guide |
| [First Run](docs/FIRST_RUN.md) | Quick start for new users |
| [Setup Checks](docs/SETUP_CHECKS.md) | Environment readiness reference |
| [Diagnostics](docs/DIAGNOSTICS.md) | Troubleshooting and export |
| [Release Artifacts](docs/RELEASE_ARTIFACTS.md) | Platform downloads and limitations |
| [Demo Script](docs/DEMO_SCRIPT_v1.0.3.md) | Deterministic demo walkthrough |

## Developer Setup

<details>
<summary>Build from source</summary>

### Prerequisites

- Rust 1.80+ (`rustup`)
- Node.js 20+ and npm
- Python 3.11+ and `uv` (for AI service)
- Git

### Build

```bash
git clone https://github.com/Elephant-Rock-Lab/Orqestra.git
cd Orqestra

# Build Rust workspace (4 crates + Tauri app)
cargo build --workspace

# Build desktop frontend
cd apps/desktop && npm ci && npm run build

# Build dashboard
cd apps/dashboard && npm ci && npm run build
```

### Test

```bash
# Rust tests (141 total)
cargo test --workspace

# TypeScript builds
npm run build -w apps/desktop
npm run build -w apps/dashboard
```

### Run AI Service

```bash
cd services/ai
uv run uvicorn orqestra_ai.main:app --port 8000
```

### Run Desktop (dev mode)

```bash
cd apps/desktop
npm run tauri dev
```

### Architecture

```
Orqestra/
├── crates/
│   ├── md-indexer/       # Markdown roadmap parser
│   ├── git-bridge/       # Semantic commits, backfill
│   ├── graph-store/      # Triple store for history
│   └── loro-engine/      # CRDT per-file sync
├── apps/
│   ├── desktop/          # Tauri 2.x + React app
│   └── dashboard/        # Cloudflare Pages dashboard
├── services/
│   └── ai/               # FastAPI AI service
├── agents/               # Agent workspaces and skills
└── roadmap/              # Project roadmap tasks
```

</details>

## License

Proprietary — Elephant Rock Lab.
