# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
