# Orqestra

**Local-first, AI-native development environment.**

Orqestra manages your project through markdown files in `roadmap/` — tasks, sprints, and epics are all defined in YAML frontmatter. AI agents understand your codebase through semantic commits, knowledge graphs, and reasoning traces. Everything is stored locally in `.Orqestra/` and synced via CRDT.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Orqestra Desktop                    │
│            (Tauri 2.x + React 19)                   │
│                                                      │
│  ┌──────────┐ ┌──────────┐ ┌─────────────────────┐  │
│  │Task Table│ │  Gantt   │ │     Kanban Board     │  │
│  └──────────┘ └──────────┘ └─────────────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌─────────────────────┐  │
│  │  Commit  │ │  Query   │ │    Agent Panel       │  │
│  │  Panel   │ │  History │ │    (3 workspaces)    │  │
│  └──────────┘ └──────────┘ └─────────────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌─────────────────────┐  │
│  │ Semantic │ │Shockwave │ │    CRDT Sync         │  │
│  │  Diff    │ │  Merge   │ │    Panel             │  │
│  └──────────┘ └──────────┘ └─────────────────────┘  │
├─────────────────────────────────────────────────────┤
│              Rust Core (Workspace Crates)            │
│  ┌────────────┐ ┌────────────┐ ┌────────────────┐   │
│  │ md-indexer │ │ git-bridge │ │  graph-store   │   │
│  │  (parser)  │ │ (semantic  │ │  (triples +    │   │
│  │            │ │  commits)  │ │   vector)      │   │
│  └────────────┘ └────────────┘ └────────────────┘   │
│  ┌────────────┐                                     │
│  │loro-engine │  CRDT sync with offline merge       │
│  │  (Loro)    │  Token-based access control          │
│  └────────────┘                                     │
├─────────────────────────────────────────────────────┤
│              Python AI Service (FastAPI)             │
│  /extract-intent  /embed  /query-history  /explore  │
│  sentence-transformers · Z.ai gateway · reasoning   │
├─────────────────────────────────────────────────────┤
│              Public Dashboard (Cloudflare Pages)     │
│  Gantt · Kanban · Table · TokenGate                 │
└─────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- **Rust** 1.80+ (`rustup`)
- **Node.js** 22+ (`nvm install 22`)
- **Python** 3.12+ (`uv` recommended)

### Build

```bash
# Clone
git clone https://github.com/Elephant-Rock-Lab/Orqestra.git
cd Orqestra

# Rust workspace (core crates + Tauri backend)
cargo build --workspace

# Desktop frontend
cd apps/desktop
npm install
npm run tauri dev

# AI service
cd services/ai
uv sync
uv run uvicorn orqestra_ai.main:app --reload

# Public dashboard
cd apps/dashboard
npm install
npm run dev     # development
npm run build   # production → dist/
```

### Run Tests

```bash
# All Rust tests (66 tests)
cargo test --workspace

# Individual crates
cargo test -p md-indexer    # 37 tests
cargo test -p git-bridge    # 10 tests
cargo test -p graph-store   # 7 tests
cargo test -p loro-engine   # 12 tests
```

## Project Structure

```
Orqestra/
├── crates/
│   ├── md-indexer/       # YAML frontmatter parser + dependency graph
│   ├── git-bridge/       # Semantic commit pipeline + backfill
│   ├── graph-store/      # Triple store + commit indexer
│   └── loro-engine/      # Loro CRDT sync + token auth
├── apps/
│   ├── desktop/          # Tauri 2.x desktop app (React 19 + TypeScript)
│   └── dashboard/        # Public read-only dashboard (Cloudflare Pages)
├── services/
│   └── ai/               # FastAPI AI service (intent extraction + embeddings)
├── agents/
│   ├── workspaces/       # Agent persona configs (architect, bugfix, docs)
│   └── skills/           # SKILL.md definitions (debugging, docs, testing)
├── roadmap/              # Project tracker — single source of truth
│   ├── _index.md         # Coordinator: sprints, epics, team
│   └── TASK-*.md         # Individual task files
├── .Orqestra/            # Generated data (gitignored)
│   └── graph/            # Commit stubs, triples, reasoning traces
├── .github/
│   └── workflows/
│       └── orqestra-agents.yml  # Agent fleet triggered on issues
└── CHANGELOG.md
```

## Key Concepts

### Roadmap as Source of Truth
Every task lives in `roadmap/TASK-YYYY-NNN.md` with structured YAML frontmatter. The `_index.md` coordinator defines sprints, epics, and team membership. No database, no API — just files in your repo.

### Semantic Commits
Every commit carries structured metadata: intent summary, affected concepts, confidence score, and a reasoning trace. Stored in `.Orqestra/graph/commits/{hash}.json`. The ConfidenceGate auto-commits at ≥0.90 confidence and flags below 0.50.

### Knowledge Graph
Commit metadata is decomposed into subject-predicate-object triples. Vector search (via `all-MiniLM-L6-v2` embeddings) enables natural-language queries like "When did we introduce rate limiting?"

### Agent Fleet
Three agent personas — architect, bugfix, docs — each with their own workspace config, skill set, and confidence thresholds. Tasks are routed by label matching. The GitHub Action in `.github/workflows/orqestra-agents.yml` triggers the fleet when issues are created.

### CRDT Sync
Loro CRDT enables real-time collaborative editing of task files. Each file is an independent `LoroDoc` with offline delta export/import. Two peers can diverge offline and merge cleanly with zero data loss. Token-based access control gates write operations.

## Configuration

| File | Purpose |
|------|---------|
| `roadmap/_index.md` | Sprint definitions, epics, team |
| `agents/workspaces/*/workspace.yml` | Agent persona configs |
| `agents/skills/*/SKILL.md` | Skill definitions |
| `services/ai/.env` | `ZAI_API_KEY` for AI gateway |
| `.Orqestra/` | Generated graph data (gitignored) |

## Dashboard Deployment

```bash
cd apps/dashboard
npm run build
npx wrangler pages deploy dist --project-name=orqestra
```

The dashboard is a static React site deployable to Cloudflare Pages, Netlify, or any static host. It renders read-only Gantt, Kanban, and Table views from roadmap data.

## License

Private repository — © 2026 Elephant Rock Lab
