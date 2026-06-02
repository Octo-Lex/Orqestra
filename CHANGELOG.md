# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.2] - 2026-06-02

### Added
- OS-keychain-backed credential vault using `keyring-core` + `windows-native-keyring-store`
- Session-only fallback vault when OS keychain is unavailable
- Token masking for all errors crossing the Rust/TypeScript boundary
- Legacy XOR credential migration with safe verify-then-delete pattern
- CI-generated dashboard data and Cloudflare Pages deployment from master
- Cross-platform desktop release workflow for Windows, macOS, and Linux
- Native `gix` semantic commit path (commit object creation + HEAD update)
- Real bugfix-agent execution path with user-selected file scope
- `POST /agent/bugfix` endpoint in AI service
- `run_bugfix_agent_cmd` with path validation against user-selected files
- `read_project_file_cmd` with path-traversal protection
- 5 roadmap tasks for v1.0.2 (TASK-071 through TASK-075)

### Changed
- Credential storage no longer relies on XOR-based persistence
- Dashboard deployment now reflects CI-generated roadmap JSON
- Bugfix-agent output is explicitly review-only and cannot auto-commit
- Release artifacts are produced from tagged builds
- gix upgraded from 0.66 to 0.84 for better commit creation API

### Security
- Raw GitHub PATs are never returned to TypeScript after save
- Insecure credential persistence fallback is disallowed
- Token masking added for logs and UI errors
- OS-keychain failure is a blocking persistence error

### Known Limitations
- File staging and diff formatting still use shell-out git commands
- Full edge worker remains backlog
- Durable Object CRDT relay remains backlog
- AST/tree-sitter analysis remains backlog
- ML-Master exploration remains incomplete
- Architect-agent execution remains mock-mode
- Bugfix-agent cannot discover files automatically
- Agent commits require human approval

## [1.0.1] - 2026-06-01

### Added
- Real roadmap JSON export via `orqestra export --format=json --out <path>`
- `_index.md` coordinator parser for sprints, epics, and team data
- Dashboard consumes generated `orqestra-roadmap.json` instead of hardcoded data
- Loading and error states for dashboard data fetch
- Footer showing generation timestamp and source commit
- Cloudflare Pages CI workflow (`.github/workflows/dashboard.yml`)
- `wrangler.toml` for one-command deployment
- Production Tauri build: NSIS installer for Windows x64
- Encrypted credential storage: `credentials.rs` with save/get/status/delete/migrate
- `POST /agent/docs` endpoint in AI service
- `run_docs_agent_cmd` in Tauri: calls real AI service with file scope enforcement
- `DiffReviewPanel`: side-by-side before/after diff display with accept/reject
- Docs agent execution path in `AgentPanel` with "Run Docs Agent" button
- v1.0.1 policy: all agent outputs require human review, auto-commit disabled
- Roadmap tasks TASK-2026-066 through TASK-2026-070 covering hardening work

### Changed
- Dashboard `data.ts` no longer contains hardcoded mock roadmap data
- Dashboard `PublicKanban` and `PublicGantt` accept `tasks` prop from fetched JSON
- Dashboard `App.tsx` is now async with fetch-based data loading
- `tauri.conf.json` targets `nsis` instead of `all` (WiX unavailable)
- PAT storage uses encrypted vault instead of plaintext tauri-plugin-store JSON
- Agent execution UI distinguishes mock, proposed, and human-approved actions

### Security
- Removed plaintext PAT persistence via tauri-plugin-store
- Added encrypted blob storage at `app-data/github-pat.enc`
- Token masking in error messages (truncate >40 chars)
- No raw PAT in logs or UI state
- Migration path: legacy JSON → encrypted vault → verified → delete legacy

### Known Limitations
- Full edge worker is still backlog
- Full gix migration is still backlog
- AST/tree-sitter analysis is still backlog
- ML-Master exploration remains stub
- Agents do not auto-commit code changes
- Dashboard not yet deployed to live Cloudflare Pages URL
- Dashboard encryption uses XOR with machine-derived key (not AES-256-GCM)

## [1.0.0] - 2026-06-01

### Added — Phase 0: Foundation
- `crates/md-indexer/`: YAML frontmatter parser, dependency graph builder, DOT export
- CLI binary: `orqestra deps --format=dot --project-root <path>`
- Duration newtype: `u32` minutes bridging YAML `"8h"`, JSON `480`, TypeScript `number`
- 37 unit tests covering parser, graph, indexer, and duration edge cases

### Added — Phase 1: Desktop + AI Pipeline
- `apps/desktop/`: Tauri 2.x + React 19 + TypeScript + Vite desktop application
- TaskTable: renders all parsed roadmap tasks with status, priority, progress
- Git sync: `git_pull_roadmap` / `git_push_roadmap` via `tauri-plugin-shell`
- PAT storage via `tauri-plugin-store`
- `crates/git-bridge/`: semantic commit pipeline with Pending→Complete upgrade
- AI backfill: POSTs to `/extract-intent`, upgrades stubs via atomic write-rename
- ConfidenceGate: `auto_commit` ≥0.90, `propose` ≥0.70, `flag` ≥0.50, `abort` <0.50
- CommitPanel: commit UI with 4-phase tracking and color-coded gate action
- `services/ai/`: FastAPI AI service with `/health`, `/extract-intent`, `/embed`
- `reasoning.py`: async reasoning trace storage
- E2E test binary against Z.ai: confidence 100%, `auto_commit` gate action
- Browser E2E framework with `BROWSER_TEST=1` env var and Vite alias mocks

### Added — Phase 2: Project Management
- GanttView: Canvas-based Gantt chart with horizontal timeline
- KanbanView: drag-and-drop columns (via `@dnd-kit/core`)
- SmartScheduler: automatic scheduling based on dependencies and availability
- TimeTracking: timer per task with cumulative logging
- ViewSwitcher: Table / Gantt / Kanban mode switching
- `update_task_status_cmd`: Tauri command for status transitions
- E2E: 5/5 browser checks (Table+TimeTracking, Gantt, Kanban, View switching)

### Added — Phase 3: Multi-Agent Workspaces
- `agents/workspaces/`: architect, bugfix, docs workspace configs
- `agents/skills/`: debugging, documentation, testing skill definitions
- AgentWorkspace.ts: workspace config loader
- SkillLoader.ts: SKILL.md parser
- AgentRouter.ts: label-based task→agent matching
- AgentRunner.ts: parallel multi-agent execution
- AgentPanel.tsx: agent dispatch UI with per-task status tracking
- Rust Tauri commands in `commands/agents.rs`
- E2E: 8/8 checks — 3 agents ran in parallel, different output, no context bleed

### Added — Phase 4: Semantic Git & Queryable History
- `crates/graph-store/`: triple store for commit metadata (7 tests, 26 triples)
- Python `query_history.py`: vector search via `all-MiniLM-L6-v2` embeddings
- Tauri graph commands: index, query, query_history, read_trace, read_commit_stub
- QueryHistory.tsx: NL query UI with expandable commit detail
- SemanticDiff.tsx: "What Changed" + "Why (Intent)" side-by-side panels
- ShockwaveMerge.tsx: merge conflict UI with AI resolution proposals
- E2E: 14/14 browser checks including vector search accuracy

### Added — Phase 5: Cloud Sync & Public Dashboard
- `crates/loro-engine/`: Loro CRDT per-file documents with peer IDs, offline delta export/import
- 2-peer offline merge: both peers converge to identical state, zero data loss
- Snapshot persistence: save/load Loro snapshots to `.Orqestra/crdt/`
- Token-based access control: master/write/read tokens with scope gating
- 13 CRDT sync Tauri commands in `commands/sync.rs`
- LoroProvider.ts + SyncPanel.tsx: sync status, merge demo, token management
- `apps/dashboard/`: standalone React site with PublicGantt, PublicKanban, TokenGate, Table view
- E2E: 15/15 browser checks pass

### Added — Phase 6: Self-Hosting
- `roadmap/_index.md`: authoritative coordinator with sprints, epics, team
- 12 task files covering Phases 0–6 (done) and future work (backlog)
- `.github/workflows/orqestra-agents.yml`: agent fleet triggered on issue creation
- Dashboard built as static site ready for Cloudflare Pages deployment
- CHANGELOG.md and README.md for v1.0.0 release
- 66 Rust tests passing (37+10+7+12)
- 56+ browser E2E checks passing across all phases

### Technical Stack
| Component | Technology |
|-----------|-----------|
| Core Engine | Rust 1.95, serde, serde_yaml, petgraph |
| CRDT | Loro (loro crate) |
| Git Operations | git CLI (gix migration planned) |
| Desktop | Tauri 2.x, React 19, TypeScript 5.7, Vite 6 |
| AI Service | Python 3.14, FastAPI, sentence-transformers |
| AI Gateway | Z.ai (Anthropic-compatible API) |
| Dashboard | React 19, Vite 6, Cloudflare Pages |
| CI/CD | GitHub Actions |

[1.0.0]: https://github.com/Elephant-Rock-Lab/Orqestra/releases/tag/v1.0.0
