# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),

## [1.0.10] - 2026-06-03

### Added
- Linux AppImage runtime smoke attempt under WSL2/Xvfb
- Linux runtime environment recording (Ubuntu 24.04 WSL2, GTK 3.24, WebKit2GTK 2.52)
- Linux runtime blocker evidence: GTK init fails without display server (`tao-0.35.3`)
- `runtime-blocked` and `runtime-evidence-wsl2` statuses in manifest validator
- `runtime_attempted`, `smoke_result`, `runtime_environment`, `runtime_blocker` manifest fields

### Changed
- Linux status: `bundle-produced-unverified` -> `runtime-blocked` (WSL2 GTK init failure recorded)
- Platform confidence documentation updated with WSL2 runtime evidence
- Linux troubleshooting updated with GTK/display server failure guidance

### Security
- SHA256 verification required for Linux AppImage
- Issue templates continue warning users not to share API keys, tokens, or secrets

### Known Limitations
- Linux AppImage runtime-blocked: GTK cannot init without display server
- Linux not promoted without native desktop smoke test
- Windows installer remains unsigned
- macOS remains build-feasibility-only

## [1.0.9] - 2026-06-03

### Added
- Linux AppImage bundle target configured in Tauri (`appimage` added to targets)
- CI rename step canonicalizes Linux AppImage filename
- CI manifest generation now includes Linux artifact with SHA256
- Linux bundle evidence (`demo/v1.0.9-linux-bundle-evidence.md`)
- Linux smoke blocker evidence (`demo/v1.0.9-linux-smoke-blocked.md`)
- Platform matrix evidence for v1.0.9
- `bundle-produced-unverified` status in manifest validator
- Linux troubleshooting sections in troubleshooting docs
- Linux AppImage warning in README and release notes
- `verification_status: checksummed-not-smoke-tested` on Linux artifact

### Changed
- Linux status: `built-but-unverified` -> `bundle-produced-unverified`
- CI workflow updated to discover, rename, and checksum Linux artifacts
- Manifest validator allows `PENDING_CI` placeholder for CI-built artifacts

### Security
- Windows signing blocker remains explicit (certificate-not-available)
- SHA256 verification required for every published artifact
- Linux artifact is not described as tested or supported

### Known Limitations
- Linux AppImage is checksummed but not smoke-tested
- Linux is not a tested public beta platform
- macOS remains feasibility-only
- Windows installer remains unsigned

## [1.0.8] - 2026-06-03

### Added
- Cross-platform CI evidence: Linux x64 and macOS compile successfully in CI (Run #26847116112)
- Platform matrix evidence document (`demo/v1.0.8-platform-matrix.md`)
- Linux build evidence (`demo/v1.0.8-linux-build-evidence.md`) -- CI compiles binary, no AppImage/DEB
- macOS feasibility evidence (`demo/v1.0.8-macos-feasibility.md`) -- CI compiles universal binary, no DMG
- Manifest platform fields: `compile_status`, `bundle_status`, `public_artifact`, `smoke_tested`, `release_blocking`
- Platform verification section in manifest with CI run reference
- Validator enforces: compile success alone cannot promote a platform
- Validator enforces: `final_artifact_state` on all artifacts
- Validator enforces: no platform marked tested without smoke evidence
- macOS promoted to `build-feasibility-verified` based on CI evidence

### Changed
- macOS status: `not-built` -> `build-feasibility-verified` (CI compilation proven)
- Platform confidence documentation updated for v1.0.8 evidence
- Manifest validator updated with platform-specific promotion rules

### Security
- Windows signing blocker remains explicit (certificate-not-available)
- SHA256 verification required for every published artifact
- No platform promoted without checksum and smoke evidence

### Known Limitations
- Windows installer remains unsigned unless signing credentials become available
- Linux binary compiles but no AppImage/DEB bundle is produced
- macOS binary compiles but no DMG/app bundle is produced
- Compile success is not platform support

## [1.0.7] - 2026-06-02

### Added
- Signing blocker evidence — manifest records exact blocker (certificate-not-available) and next action
- Signature verification evidence file (`demo/v1.0.7-signature-verification.md`) even for unsigned artifacts
- Installer diagnostics guide (`docs/installer-diagnostics.md`) with SHA256, signature, WebView, log, and AI service checks
- Platform confidence document (`docs/platform-confidence.md`) explaining tested/built-but-unverified/not-built criteria
- Public beta issue triage guide (`docs/beta-issue-triage.md`) with labels, severity, and response policy
- `final_artifact_state` field in manifest artifacts to prevent hash-before-signing ambiguity
- Hash ordering assertion in demo evidence
- Install issue template updated with signature status, SHA256, SmartScreen, install location, and secrets confirmation fields
- SmartScreen guidance split into unsigned vs. signed-but-low-reputation sections in troubleshooting

### Changed
- Release manifest now includes `signing` block, `diagnostics` block, and platform `smoke_evidence` fields
- Manifest validator extended for signing, diagnostics, and platform evidence fields
- README includes signature verification command and conservative SmartScreen language
- Platform matrix unchanged: Windows tested, macOS not-built, Linux built-but-unverified

### Security
- Signing secrets must not be printed, committed, or attached to issues
- Issue templates warn users not to share API keys, GitHub tokens, .env files, or certificate material
- Unsigned installer warning retained prominently in README, release notes, and manifest
- Install issue template requires secrets removal confirmation

### Known Limitations
- SmartScreen may still warn even for signed early beta installers
- macOS artifacts remain unavailable
- Linux remains built-but-unverified
- Some advanced agent paths remain review-only or scaffolded
- Code signing remains blocked (certificate not available)

## [1.0.6] - 2026-06-02

### Added
- Public beta quickstart guide (`docs/beta-quickstart.md`)
- Troubleshooting guide for install, launch, dashboard, Git, and AI mode failures (`docs/troubleshooting.md`)
- Four GitHub issue templates: install, AI mode, dashboard, bug report
- Dashboard freshness metadata: release version, source commit, generation timestamp in exported JSON
- Dashboard footer displays release version and full source commit
- Release manifest now includes `distribution` and `dashboard` sections
- Signing readiness status table in release signing plan
- Release-link audit gate to verify all GitHub release links resolve

### Changed
- README restructured with reviewer quickstart as primary path
- Platform decision: Windows-only beta remains (macOS/Linux deferred)
- Release manifest validator checks `distribution` and `dashboard` sections
- Roadmap export now includes `release` metadata object with full source commit SHA

### Security
- Unsigned installer warning remains prominent
- Issue templates explicitly warn users not to paste API keys or secrets
- Real-AI mode documentation separates key setup from no-key beta mode

### Known Limitations
- Windows remains the only tested public beta platform
- macOS artifacts remain unavailable
- Linux remains built-but-unverified
- Some advanced agent paths remain review-only or scaffolded
- Code signing remains pending (certificate not yet available)

## [1.0.5] - 2026-06-02

### Added
- Release provenance manifest with full 40-char Git SHAs, CI workflow run ID, and verification block
- Manifest validation script (`scripts/validate-release-manifest.ts`) with schema enforcement
- Public beta platform status matrix (tested / built-but-unverified / not-built / deferred)
- Windows installer smoke-test evidence (`demo/v1.0.5-windows-smoke.md`)
- Full demo evidence with public claim review checklist (`demo/v1.0.5-demo-evidence.md`)
- SHA256 verification instructions in README and release notes
- Release signing and notarization plan (`docs/release-signing-plan.md`)
- CI workflow hardened: SHA256 generation, manifest generation, checksum upload, provenance fields
- Release notes template for CI-generated releases
- `dist/checksums.txt` generated alongside artifacts

### Changed
- Version aligned: installer, app, Cargo, Tauri config all say 1.0.5
- README restructured for public beta: status, download+verify, platform support, AI modes, provenance
- Release manifest now uses canonical `release-manifest.json` at repo root (not `dist/`)
- Platform labels use explicit tested/not-built/built-but-unverified/deferred statuses
- Public claims classified: beta, local-only, unsigned, review-only, scaffolded, backlog
- CI gates separated: required (must pass) vs maintainer release gates (real-AI, smoke)

### Security
- Unsigned installer warning made prominent in README, release notes, manifest
- SHA256 verification path documented with PowerShell command
- Real-AI mode setup documented separately from no-key beta mode
- Manifest validator checks for secret patterns

### Known Limitations
- Windows installer remains unsigned
- macOS artifacts are not built
- Linux artifact remains unverified
- Some advanced agent paths remain review-only or scaffolded
- Code signing and notarization are planned but not implemented

## [1.0.4] - 2026-06-02

### Added
- Fresh release manifest with artifact checksums, platform labels, and signing status
- Demo evidence file for packaged-artifact verification
- Real-AI demo path documentation for docs-agent and bugfix-agent
- AI demo fixtures (`demo/ai-fixtures/`) with deterministic test inputs
- `AiReadinessStatus` TypeScript interface mapping raw readiness to spec-aligned modes
- Unsigned beta warning in all release-facing documentation
- Platform classification table (tested / built-but-unverified / not-built / blocked)

### Changed
- Dashboard deployment workflow now uses explicit Cloudflare `accountId`
- Release notes now distinguish tested, built-but-unverified, not-built, and unsigned artifacts
- Demo script now includes no-key beta and real-AI maintainer modes
- AI service loads `ZAI_API_KEY` from `.env` via `python-dotenv` (no manual env export needed)
- `docs/RELEASE_ARTIFACTS.md` restructured with v1.0.4 platform statuses and dashboard deployment note

### Fixed
- Dashboard CI deployment no longer relies on Cloudflare account discovery through memberships lookup
- v1.0.4 release artifacts are rebuilt from current source instead of reusing v1.0.2 binaries
- AI service correctly detects and uses `ZAI_API_KEY` when present in `.env`

### Security
- Diagnostics redaction remains enforced for exported bundles
- Release documentation states that desktop binaries are unsigned beta artifacts
- `AiReadinessStatus` DTO never exposes raw API keys or token values

### Known Limitations
- Code signing and notarization are not yet done
- Full native gix migration remains incomplete (8 shell-outs remain)
- Architect agent remains mock-mode
- ML-Master exploration remains stub
- Edge relay is still backlog
- macOS artifacts require bundler target configuration
- Linux artifacts are CI-built but not locally validated

## [1.0.3] - 2026-06-02

### Added
- First-run onboarding wizard with guided setup flow
- Environment and integration readiness panel with status cards
- Project validation before workspace load (valid/repairable/not_orqestra/invalid/inaccessible)
- Generated sample Orqestra project for external reviewers
- Diagnostics panel and redacted diagnostic bundle export
- Error recovery cards for common setup failures (ROADMAP_NOT_FOUND, AI_KEY_MISSING, etc.)
- Release artifact manifest and clearer platform labels
- User-ready beta demo script (docs/DEMO_SCRIPT_v1.0.3.md)
- User guide, first run guide, setup checks, diagnostics docs
- 26 new Rust tests (onboarding, readiness, project validation, diagnostics, redaction)
- 13 new TypeScript modules (wizard steps, readiness/diagnostics panels, lib modules)
- 6 new Tauri commands (onboarding, validation, sample project, readiness, diagnostics, recovery)

### Changed
- README now foregrounds install/evaluation path before source-build path
- Missing AI/cloud setup is represented as degraded readiness, not generic failure
- Project loading now validates roadmap structure before opening the main workspace
- Onboarding state persists across app restarts (no secrets stored)

### Security
- Diagnostics export redacts API keys, GitHub tokens, bearer tokens, and secret-like values
- Readiness DTOs are forbidden from exposing raw secret material
- Onboarding state excludes credentials and unlock data
- Redaction tests verify all known secret patterns are handled

### Known Limitations
- Architect agent remains mock-mode
- ML-Master exploration remains stub/backlog
- Full native gix migration remains incomplete (9 shell-outs remain)
- AST/tree-sitter analysis remains backlog
- Edge relay / Durable Objects remain backlog
- Some artifacts may be unsigned beta builds

## [1.0.2] - 2026-06-01
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
