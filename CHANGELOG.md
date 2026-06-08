# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),

## [2.6.0] - 2026-06-08

### Added
- Controlled low-risk autonomy pilot: docs-only auto-apply
- `AutonomySettings` DTO persisted in AppState, disabled by default
- 12-gate auto-apply decision engine (`security/auto_apply.rs`)
- Docs-safe allowlist: `docs/` and `README.md` only
- `CHANGELOG.md` and `roadmap/` explicitly excluded
- README.md uses stricter confidence threshold (0.90 vs 0.80)
- `set_autonomy_settings_cmd` — server-side validation, no frontend policy widening
- `auto_apply_patch_cmd` — Rust loads persisted settings, never trusts caller
- Per-session auto-apply cap (default 5)
- `RequiresReview` decision — never writes, records audit only
- Patch size computed server-side from actual content
- Path canonicalization and traversal detection before allowlist checks
- Redacted auto-apply audit records (path class, not raw path)
- `get_autonomy_settings_cmd` — read-only settings access
- 33 new auto_apply unit tests + 17 onboarding persistence tests

### Changed
- Diagnostics report autonomy as `pilot` mode with policy metadata
- `AutonomySettings` stored in `app-state.json` alongside AppState

### Security
- Frontend never authoritative for autonomy policy
- Auto-apply never commits
- All existing PatchApplicationGuard checks still apply
- Traversal and symlink escape attempts fail closed
- Audit records contain no source bodies, tokens, or raw paths

## [2.5.3] - 2026-06-08

### Added
- Persistent onboarding state (`app-state.json` in Tauri app data dir)
- Atomic write (tmp + rename) with corruption recovery (backup + default)
- `ProjectRecord` with stable `project_id` (SHA-256 of canonical root)
- Recent projects list (max 10, deduped by project_id)
- `switch_project_cmd` — updates last project, triggers relay reconnect
- `record_project_access_cmd` — updates status metadata per project
- `reset_onboarding_cmd(clear_metadata, clear_project_history)` — never clears OS keychain
- `CredentialStatus` and `RelayConnectionStatus` as metadata-only enums
- 9 onboarding_types unit tests + 8 persistence integration tests

### Changed
- `OnboardingStateManager` loads from disk on first access, saves on every mutation
- `last_known_relay_status` — metadata only, recomputed on project open
- `last_known_credential_status` — metadata only, global credential availability

### Security
- No secrets, tokens, PATs, or CRDT data in `app-state.json`
- Reset onboarding never clears OS-keychain credentials
- Corrupt file backed up as `app-state.corrupt.{timestamp}.json`

## [2.5.2] - 2026-06-08

### Added
- Real desktop relay connection via Rust-owned WebSocket lifecycle (`tokio-tungstenite`)
- `RelayActor`: async tokio task managing connect/reconnect/send/receive
- `RelayActorHandle`: deterministic shutdown via `watch` channel
- `RelayEvent`: channel-based events (Tauri-free, `loro-engine` has no tauri dep)
- Double-connect guard: returns `ALREADY_CONNECTED` instead of spawning duplicate actor
- Client-side remote delta deduplication (capped `seen_remote_ids`)
- Offline queue preservation on disconnect (deltas replayed on reconnect)
- Exponential backoff reconnect (1s → 30s)
- Redacted Tauri events: `relay:connected`, `relay:disconnected`, `relay:delta-received` (no raw paths/data/tokens)
- `RelayStatus` gains `reconnect_attempt` and `last_error` fields
- 12 relay actor unit tests + 13 desktop relay connection tests

### Changed
- `RelayClient.status()` includes `reconnect_attempt`, `last_error`
- `RelayClient.new_message_id()` made public
- Worker `sync_relay.test.ts` updated for async `validateToken`/`generateToken`
- Module split: `relay.rs` (state machine) vs `relay_actor.rs` (WebSocket lifecycle)

## [2.5.1] - 2026-06-07

### Security
- **Critical:** Sync relay HMAC replaced with real HMAC-SHA256 (was djb2-like string hash)
- **Critical:** Hardcoded `default-master-token` removed from desktop binary
- **Critical:** Cloudflare and ZAI credentials rotated, stored only in provider secret stores
- **High:** Tauri CSP enabled (was `null`) — no wildcards, no `unsafe-eval`
- **High:** Patch checksum uses real SHA-256 (was `DefaultHasher` 16-char)
- **High:** Legacy 16-char checksums rejected (`LEGACY_CHECKSUM_FORMAT`)
- **High:** GitHub PAT temp file uses unique temp dir, `create_new`, RAII cleanup
- Token format versioned: `ork_v2_` prefix, legacy v1 rejected (`UNSUPPORTED_TOKEN_VERSION`)
- Constant-time HMAC comparison in relay auth
- TokenManager without master secret cannot generate admin tokens
- Secret scanning CI workflow added
- 17 security boundary tests
- 12 relay auth tests

### Changed
- `TokenManager::new` accepts `Option<&str>` (None = desktop mode, no admin)
- `TokenManager::generate` returns `Result<String, String>` (error if no master secret + admin)
- PAT askpath classified as interim mitigation (v2.5.2+ will use in-memory flow)

## [2.5.0] - 2026-06-06

### Added
- Native Git write: tree building from index (no CLI `git write-tree`)
- Native commit creation via gix (no CLI)
- Compare-and-swap HEAD update (aborts if HEAD changed mid-commit)
- All-or-nothing native path (no mixing native + CLI in same commit)
- CLI fallback path (`gix-hybrid-fallback`) always available
- `CommitPathDiagnostic` DTO with tree/commit/head_update methods
- `NativeWriteCommitResult` with provider label derivation
- Reviewed proposal ID required for native commit
- Operational-risk gate in commit proposal
- `GitWriteMethod` enum (Native/CliFallback)
- 18 native write tests

### Changed
- Provider label `gix` only when fully CLI-free
- Any CLI fallback → `gix-hybrid-fallback`
- Disk cleaned (26.1 GiB freed via cargo clean)

### Security
- Empty commit messages rejected
- Empty proposal IDs rejected
- HEAD race protection (compare-and-swap)
- No auto-commit path introduced
- Index consistency guaranteed after commit

## [2.4.0] - 2026-06-06

### Added
- Operational risk classifier (path-based, deterministic)
- 13 risk categories: DependencyManifest, DependencyLockfile, CiWorkflow, CloudflareConfig, TauriConfig, ReleaseManifest, CredentialOrSecretConfig, BuildConfig, RepoPolicyConfig, ToolchainConfig, PackageManagerConfig, ContainerConfig, UnknownSensitiveConfig
- 5 severity levels: Critical, High, Medium, Low, Info
- 13 stable reason codes
- Multi-risk per file (classify_path returns Vec<OperationalRisk>)
- Highest severity determines enforcement
- CredentialOrSecretConfig rejects write outright (no human override)
- blocks_auto_apply forbids future auto-apply, not human apply
- UnknownSensitiveConfig escalated to Medium in sensitive directories
- operational-risk.json in diagnostics bundle (17th file, path-hashed)
- ArchitectRiskSummary DTO
- operational_risks on AgentContextV2 and SemanticCommitContext
- Patch governance integration: credential paths rejected, critical files flagged
- 27 operational risk tests

### Changed
- Diagnostics bundle: 16 → 17 files (added operational-risk.json)

### Security
- Credential/secret config writes rejected outright
- Diagnostics use path hashes, not raw paths
- No file content parsing for classification

## [2.3.0] - 2026-06-05

### Added
- Hunk-to-symbol impact mapping (interval overlap semantics)
- ParsedHunk DTO with old/new line ranges
- SymbolImpact DTO with overlap_type, confidence, parent_symbol metadata
- OverlapType enum: InsideSymbol, TouchesSymbolBoundary, NearSymbol, FileLevelOnly, NoSymbolMatch
- One hunk → many symbols mapping
- Most-specific symbol wins; parent as metadata (not separate impact)
- Bounded output: max 50 per file, 200 total, truncation reported
- ArchitectSymbolSummary DTO for architect-agent context
- hunk_level_symbols on AgentContextV2 (high-confidence filter)
- Hunk header parser (unified diff format)
- 16 code-intel hunk_map tests + 11 integration tests

### Changed
- Primary mapping uses new-file line ranges against current-file symbols
- Deleted hunks/files degrade gracefully without parsing old blobs
- code-intel dependency added to git-bridge

### Security
- SymbolImpact contains no source bodies
- Path redaction in diagnostics bundles
- Read-only: no file writes, no mutations

## [2.2.0] - 2026-06-05

### Added
- Dashboard source metadata panel (export-time info, not viewer's local HEAD)
- Desktop coherence check command (`check_dashboard_coherence_cmd`)
- Canonical roadmap state hash (SHA-256 over sorted canonical JSON)
- Freshness states: current, stale, diverged, local-only, relay-unavailable, unknown
- `coherence.json` in diagnostics bundle (16th file, redacted)
- `DashboardCoherencePanel.tsx` — desktop component for local-vs-dashboard comparison
- `SourceMetadata.tsx` — static dashboard export metadata
- Dashboard JSON backward-compatible optional `coherence` field
- 12 coherence tests

### Changed
- Terminology: "coherence" consistently throughout
- Diagnostics bundle: 15 → 16 files (added coherence.json)
- `git_bridge::get_head_hash` now re-exported from lib

### Security
- coherence.json contains no secrets, source bodies, raw deltas, task titles, or unredacted workspace IDs
- Static dashboard does not claim freshness relative to viewer's local HEAD
- Desktop computes local-vs-dashboard freshness separately

## [2.1.0] - 2026-06-05

### Added
- `services/sync-relay/` — Cloudflare Worker + Durable Object relay
- Wire protocol v1 with message_id, sequence, ack/error, protocol_version
- Payload bounds: 1 MiB delta, 10 MiB snapshot, 100 queued deltas, 20 peers per room
- Token auth: Worker-only master secret, workspace-scoped read/write/admin tokens
- `SyncRoom` Durable Object with peer tracking, dedupe, broadcast, snapshot persistence, GC
- `crates/loro-engine/src/protocol.rs` — protocol types + validation (8 tests)
- `crates/loro-engine/src/relay.rs` — RelayClient with queue, reconnect, ack handling (8 tests)
- Worker/DO test suite (9 tests in `services/sync-relay/`)
- Tauri commands: `connect_relay_cmd`, `disconnect_relay_cmd`, `relay_status_cmd`
- `sync-status.json` in diagnostics bundle (redacted — no tokens, no workspace IDs)
- `docs/cloud-crdt-relay.md`

### Changed
- Diagnostics bundle: 14 → 15 files (added sync-status.json)
- SyncPanel now routes relay through Tauri commands

### Security
- Master secret lives only in Worker environment
- Desktop stores only workspace-scoped tokens
- Every message includes protocol_version
- Duplicate message_id replay is idempotent
- sync-status.json redacts workspace_id (SHA-256 hash) and relay URL (hostname only)
- Dashboard docs do not imply real-time relay-backed updates

### Known Limitations
- Cloudflare Durable Objects requires paid Workers plan
- Token HMAC uses simple keyed hash (not crypto.subtle in Worker)
- WebSocket lifecycle managed by frontend (Tauri command stores state only)

## [2.0.0] - 2026-06-05

### Added
- 10 first-run environment checks (non-mutating probes)
- 10 Tauri probe commands: check_git_available, check_repo_selectable, check_roadmap_valid, check_ai_service, check_credential_provider, check_dashboard_status, check_agent_endpoints, check_patch_governance, check_code_intel, check_git_provider
- Expanded diagnostics bundle from 7 to 13 files (added git-provider, credential-status, agent-matrix, patch-governance, code-intel, roadmap-status)
- Machine-checkable redaction tests (8 tests verifying no secrets, source bodies, raw diffs, .env content, or private file contents)
- First-run probe tests (12 tests verifying all checks are non-mutating)
- AI service and agent endpoint checks return optional/degraded (not setup failure)
- docs/patch-governance.md — patch application guard documentation
- docs/architect-agent.md — read-only planner documentation
- docs/code-intelligence.md — tree-sitter symbol extraction documentation
- docs/DEMO_SCRIPT_v2.0.0.md — deterministic 15-step beta demo
- Manifest beta_hardening section with validator gates

### Changed
- README rewritten to v2.0.0 reality (447 tests, agent portfolio complete, governed beta classification)
- docs/product-readiness.md rewritten to v2.0.0 (patch governance, code intel, architect, first-run checks, diagnostics bundle)
- FirstRunGuide.tsx expanded from 5 to 10 environment checks with descriptions
- Classification updated to "governed AI-native development beta"

### Security
- All first-run probes are strictly non-mutating
- Bundle export is non-mutating (no .Orqestra state changes)
- Redaction verified by machine-checkable tests
- AI service unavailability does not fail setup

### Known Limitations
- Windows installer is unsigned
- macOS build-feasibility-only (no human smoke test)
- Linux AppImage CI versioning issue persists
- Cloudflare CRDT relay not implemented

## [1.9.0] - 2026-06-05

### Added
- `POST /agent/architect` Python AI endpoint with structured plan generation
- `run_architect_agent_cmd` Rust Tauri command — read-only, no write path
- `ArchitectPlanResult` typed DTO (plan_id, summary, context_analysis, proposed_approach, affected_symbols, risk_assessment, dependency_warnings, acceptance_criteria, test_strategy, task_breakdown, adr_draft, confidence, schema_version)
- `SymbolRef`, `RiskItem`, `TaskBreakdownItem` DTOs for structured plan output
- `ArchitectAgentPanel.tsx` — display-only UI (no accept/reject patch buttons)
- Bounded ADR context (path, title, status, excerpt capped at 500 chars, max 10 ADRs)
- Architect context bundle: Agent Context v2 + symbol summaries + risk summary + existing ADRs
- 10 architect agent tests (structure, no-mutation, missing-service, schema, confidence, no-patch-fields, .Orqestra state unchanged)
- Manifest `architect_agent` section with 12 validator gates

### Changed
- Manifest `real_agents` now includes `architect-agent`
- AgentRunner architect mock removed — replaced with empty placeholder directing to Tauri command
- Architect route is React UI → Tauri command → Rust builds context → Python AI endpoint (no direct frontend-to-AI calls)

### Security
- Architect output has no patch-shaped fields (no before/after/edits)
- Architect plan cannot be passed to `apply_agent_patch_cmd` (structurally incompatible)
- Missing AI service returns structured error — no runtime fake plan
- Architect execution does not mutate repository files, .Orqestra runtime state, or audit files
- ADR content is bounded (metadata + 500-char excerpt only)

### Known Limitations
- Architect output is display-only in v1.9.0 (no export/save to Markdown yet)
- ADR draft is optional — only produced when applicable
- Requires ZAI_API_KEY set in Python AI service environment

## [1.8.0] - 2026-06-05

### Added
- `crates/code-intel/` — pure Rust crate with zero Tauri/git-bridge dependency
- Tree-sitter integration for Rust and TypeScript symbol extraction
- `SymbolSummary` DTO: path, language, symbols, parse_status, parse_latency_ms
- `Symbol` DTO: name, kind, line_start, line_end, is_public, parent — no source bodies
- `SymbolKind` enum: Function, Method, Struct, Enum, Trait, Impl, TypeAlias, Interface, Class, Module, Import, Constant, Variable
- `ParseStatus` enum: Success, ParseError, Excluded, TooLarge, Binary, Secret
- `CodeLanguage` detection by extension: .rs, .ts, .tsx, .js, .jsx
- Deterministic symbol ordering (line_start → line_end → kind → name → parent)
- File exclusion: binary, secret, generated/vendor dirs, >256 KiB, unsupported languages
- Parse error detection via ERROR/MISSING node ratio threshold (30%)
- `extract_symbols_cmd` Tauri command in `commands/code_intel.rs`
- `extract_symbols_batch_cmd` for multi-file extraction
- 29 code intelligence tests (Rust extraction, TypeScript extraction, exclusions, determinism, parse errors)
- Manifest `code_intelligence` section with 16 validator gates
- `docs/code-intelligence.md` documentation

### Changed
- Bugfix-agent context receives symbol summaries for changed files
- Semantic commit preparation references file-level affected symbols
- Docs-agent does not receive symbol context by default (`docs_agent_symbol_context: disabled-by-default`)

### Security
- Symbol extraction is read-only — never writes repository files
- Content-safe: outputs symbol names/kinds/ranges only, never source bodies
- Excluded files (binary, secret, generated, large) are never parsed
- `code-intel` crate has zero dependency on Tauri or git-bridge (no circular dependency)

### Known Limitations
- Only Rust and TypeScript supported (no other languages)
- Affected symbols are file-level, not hunk-level
- Parse error threshold is heuristic (30% ERROR ratio)

## [1.7.0] - 2026-06-05

### Added
- `PatchProposal` typed DTO with `proposal_id`, before/after content and checksums
- `PatchApplicationResult` typed DTO with durable statuses (proposed, rejected, apply_failed, applied)
- `AgentType` enum (docs, bugfix) for server-side policy enforcement
- `apply_agent_patch_cmd` Tauri command — validated, atomic, audited patch application
- `reject_agent_patch_cmd` Tauri command — records rejection without file modification
- `PatchApplicationGuard` in `security/patch_guard.rs` — governs all agent patch writes
- Server-side agent path policy — frontend may narrow but not widen scope
- Forbidden path enforcement: secret, workflow, binary, dependency locks, infrastructure config
- Before-content verification — patches rejected if file changed since proposal
- Atomic writes — temp-then-rename; failed writes leave original file unchanged
- Append-only JSONL audit trail at `.Orqestra/agents/{agent_type}/audit.jsonl`
- Post-apply checksum verification
- 15 patch governance tests (forbidden paths, valid patches, no auto-commit, audit records, server policy)
- Manifest `patch_governance` section with 16 validator gates
- `orqestra_desktop` lib target for integration test access

### Changed
- `AgentRunner.run()` auto_commit path removed — no direct file writes from agent runner
- Patch application must go through `apply_agent_patch_cmd`, never `write_file_cmd`
- Docs agent server policy restricts writes to README.md, docs/, roadmap/, CHANGELOG.md
- Bugfix agent server policy allows source files but enforces forbidden-path checks
- `write_file_cmd` reserved for workspace state persistence only

### Security
- Agent patches cannot modify forbidden paths (secret, workflow, binary, locks, CI config)
- Agent patches cannot silently alter files outside the proposal
- Rejected proposals leave no working-tree changes (test-verified)
- Failed validation leaves every file byte-identical to pre-command state
- Before-content verification prevents stale patches
- Frontend allowed_paths cannot widen server-side agent policy
- No auto-commit during patch application (test-verified)

### Known Limitations
- Patch governance applies to docs-agent and bugfix-agent only
- Architect agent not implemented
- Checksum uses DefaultHasher (not cryptographic SHA-256)

## [1.6.0] - 2026-06-05

### Added
- `GitProvider` enum — canonical provider labels (gix, gix-hybrid, git-cli-fallback, deterministic-heuristic, not-implemented)
- `GitOperationProvider` DTO with per-operation provider, native flag, read-only, mutates_repository, executed_in_diagnostics, latency_ms
- `GitProviderReport` DTO with per-operation diagnostics and repository validity
- `build_provider_report()` — read-only diagnostics builder (mutating ops registered but never executed)
- `RecentCommitsResult` response wrapper — carries provider even on empty commit lists
- `DiffStatResult` response wrapper — carries provider and latency
- `git_provider_diagnostics_cmd` Tauri command
- `git_recent_commits_with_provider_cmd` Tauri command
- `git_diff_stat_with_provider_cmd` Tauri command
- `GitProviderDiagnosticsPanel.tsx` — per-operation provider table with color-coded badges
- 17 provider diagnostics tests (completeness, accuracy, no-mutation guarantee, empty results, graceful degradation)
- Manifest `git_provider_diagnostics` section with 11 validator gates
- docs/native-git.md updated with v1.6.0 provider diagnostics section
- docs/product-readiness.md structured errors table now lists all 9 codes

### Changed
- Commit creation classified as `gix-hybrid` (not `gix`) — tree-from-index is CLI
- Push/pull/merge classified as `not-implemented` in provider registry
- `GitDiagnosticsPanel.tsx` provider badges now align with `GitProvider` enum values

### Security
- Provider diagnostics never mutate the repository (test-verified)
- Mutating operations show `executed_in_diagnostics: false` and `latency_ms: null`
- All new operations are read-only

### Known Limitations
- Provider diagnostics execute read-only operations only; mutating ops are reported from static registry
- Diff/stat and safe diff context remain CLI-only providers
- Push/pull/merge not implemented in git-bridge

## [1.5.1] - 2026-06-05

### Changed
- README rewritten from v1.0.12-era to v1.5.0 reality
- Added capability matrix, agent matrix, Git provider matrix, platform matrix, test trend table
- Updated download links to v1.5.0 release
- Fixed Known Limitations (Linux verified, Git hybrid not "shell-out only", bugfix agent real)
- Added documentation doctrine section
- docs/product-readiness.md rewritten from v1.1.0-era to v1.5.0-era
- Added sections for native Git, semantic commit preparation, agent context quality, safe diff context pilot
- Downgraded Shockwave merge labeling to "mock/prototype" in capability matrix
- Corrected vector search status: implemented in Python AI service, not "not found on disk"

### Fixed
- No current document says credentials are XOR-based
- No current document says bugfix agent is mock
- No current document says Linux is unverified
- No current document says "all Git operations shell out"
- Semantic commit preparation is described as deterministic and proposal-only
- Dashboard deployment status reflects CI reality

### Security
- No code changes — documentation/status alignment only
- 328 tests unchanged

### Known Limitations
- No new limitations introduced

## [1.5.0] - 2026-06-04

### Added
- Opt-in safe diff context pilot for docs-agent and bugfix-agent
- `SafeDiffContext` DTO with `enabled_source`, policy caps, per-file eligibility, exclusion reasons, hunks, and summary
- Eligibility gate with 11 exclusion reasons (secret-risk, binary, large, symlink, workflow-risk, file-limit, non-text, unsupported-status, read-error, absolute-path, disabled)
- Status policy: modified/staged/added/renamed eligible; deleted/untracked excluded
- Bounded diff hunk extraction via `git diff` (CLI-backed, provider: `git-cli-fallback`)
- Policy caps: max 5 files, max 80 lines/hunk, max 120 lines/file, max 250 total lines
- `SafeDiffContextPanel.tsx` diagnostics UI
- 23 safe diff context tests (default-off, eligibility, caps, payloads, forbidden scan, degradation, parsing)
- Manifest `safe_diff_context_pilot` section with 15+ validator gates
- `ORQESTRA_SAFE_DIFF_CONTEXT` environment variable for opt-in enablement
- v1.5.0 safe diff context evidence

### Changed
- Agent Context v2 now carries optional `safe_diff_context` metadata
- Agent diagnostics show diff-context status, caps, included/excluded files, and truncation
- `enabled_source` field records how context was enabled (`default-off` or `env:ORQESTRA_SAFE_DIFF_CONTEXT`)
- Renamed files preserve `original_path` metadata in diff context

### Security
- Safe diff context is disabled by default
- Secret-risk, binary, large, symlink, and absolute-path files are excluded
- Workflow-risk files are excluded by default
- Diff fields use `safe_diff_context`, `hunks`, and `lines`; raw `diff`/`raw_diff`/`patch` fields remain forbidden
- `SEMANTIC_PREP_DIFF_BODY_ENABLED` does not enable Agent Context safe diff context
- Legacy `read_safe_diff_body` annotated as not used for Agent Context v2
- Agents remain review-only and cannot stage files, create commits, push, pull, or auto-apply changes

### Known Limitations
- Safe diff context is a pilot
- Provider is CLI-backed in v1.5.0
- No native diff-body provider is claimed
- No subjective AI quality improvement is claimed without separate evaluation

## [1.4.1] - 2026-06-04

### Added
- Agent Context v2 payload regression fixtures for docs-agent and bugfix-agent (22 payload tests)
- Path-aware forbidden-field scan expanded from 6 to 10 forbidden keys (content, body, diff, patch, file_text, raw, token, authorization, secret_value, private_key)
- Graceful degradation tests for non-repo, deleted directory, .git-as-file, and path-points-to-file
- Forbidden-field scan scoped to `git_context` only (safe keys in task, context_files, agent response excluded from scan)
- Agent diagnostics UI shows error code when context unavailable, review-only status always visible
- Manifest `context_degradation` section (graceful, failure_blocks_agent: false)
- Manifest `stabilization` section (payload_fixtures, forbidden_field_scan: path-aware)
- Manifest `absolute_paths_displayed: false` in content policy

### Changed
- Agent context diagnostics now clearly distinguish available and unavailable context states
- Manifest validation now records context degradation guarantees
- Agent context documentation now explains path-aware forbidden-field scanning

### Security
- Agent Git context remains content-free
- Forbidden-field scan checks JSON keys inside `git_context` without false-positive substring matching
- Safe metadata keys (`secret_count`, `secret_contents_excluded`, `raw_diffs: false`) pass scan correctly
- Context failures do not enable auto-apply, auto-commit, path expansion, or repository writes
- Diagnostics continue to use repo-relative paths only

### Known Limitations
- v1.4.1 does not add new agent roles
- Agents remain review-only
- Agents do not stage files, create commits, push, pull, or perform autonomous operations
- Agent quality claims remain payload-structure-based unless separately evaluated

## [1.4.0] - 2026-06-04

### Added
- Agent Context v2 schema (`AgentContextV2`) for docs-agent and bugfix-agent
- `ProposalSummary` struct (summary-only, no `body` field)
- `ContentPolicy` struct explicitly declaring all content exclusions
- `build_agent_context_v2()` producing structured, schema-versioned context
- Agent context diagnostics UI (`AgentContextPanel`, `AgentDiagnosticsPanel`)
- 18 agent context integration tests: schema, content policy, 11 fixtures, agent payloads, forbidden-field scan, graceful degradation
- Manifest `agent_context_quality` section with no-autonomy and content-policy validator gates
- Explicit `git_context_status` and `git_context_error_code` in agent request payloads
- v1.4.0 agent context evidence

### Changed
- Docs-agent receives Agent Context v2: changed-file paths/statuses, risk summaries, commit groups, proposal summaries, recent commit subjects, diff/stat counts
- Bugfix-agent receives Agent Context v2: same structured context with user-selected allowed paths
- Both agents now send explicit `review_only: true`, `auto_commit: false`, `auto_apply: false` constraints
- Context build failure degrades gracefully with `unavailable` status and error code (does not block agent request)
- Product readiness docs now distinguish agent context quality from agent autonomy

### Security
- Git context remains content-free by construction
- Raw diffs, file contents, secrets, tokens, binary data, large file contents, and symlink targets are excluded
- Forbidden-field scan is path-aware: `secret_count`, `secret_contents_excluded`, `raw_diffs: false` are safe metadata keys
- `ProposalSummary` has no `body` field — summary-only
- Diagnostics UI displays repo-relative paths only
- Agents remain review-only with `auto_commit: false` and `auto_apply: false`

### Known Limitations
- Agent quality improvements are structural unless otherwise evidenced
- Agents do not autonomously apply patches or create commits
- New agent roles are not introduced in v1.4.0
- Diff body pilot remains disabled by default
- Native commit execution is not implemented

## [1.3.1] - 2026-06-04

### Added
- 29 semantic commit stabilization tests (proposal fixtures, no-write regression, grouping, agent context safety, diff body pilot)
- No-write regression tests proving HEAD, staging area, and worktree are unchanged after proposal generation
- Proposal quality fixtures for docs-only, test-only, Rust source, TS UI, mixed, release, workflow-risk, secret-risk, renamed, deleted, and multi-scope changes
- Agent context serialization test proving no `content`/`body`/`diff`/`patch` fields
- Diff body pilot workflow-risk exclusion test
- Determinism test proving identical input produces identical proposal
- Manifest `pushes: false`, `pulls: false`, `stabilization` sub-object, `runtime_toggle`, `release_verified_state`

### Changed
- Semantic commit UI now explicitly states "No files are staged. No commit is created. Copy/fill does not mutate the repository."
- Validator enforces `pushes`, `pulls`, and `release_verified_state` for semantic commit preparation

### Security
- No-write regression tests prove proposal generation never mutates repository state
- Agent context tests prove `.env`, key files, and symlinks do not leak content into JSON payloads
- Diff body pilot remains disabled by default; release state explicitly recorded as `disabled`

### Known Limitations
- Native commit execution is not implemented
- Push, pull, merge, and network Git operations remain on existing human-triggered flow
- AI-assisted commit message generation remains backlog
## [1.3.0] - 2026-06-04

### Added
- Proposal-only semantic commit preparation (`prepare_semantic_commit_cmd`)
- Deterministic commit title/body/scope/risk proposal builder (path-based heuristics, no AI dependency)
- Commit grouping suggestions (scope grouping + risk isolation)
- Semantic commit input model composing repository snapshot, changed files, diff/stat, and recent commits
- Content-free agent Git context injection for docs-agent and bugfix-agent
- `SemanticCommitPrepPanel` and `CommitGroupingPanel` UI components
- Manifest `semantic_commit_preparation` section with proposal-only enforcement
- Validator blocks `native_commit_execution`, `autonomous_commit`, `stages_files`, `writes_repository`
- 25 new tests: scope extraction, type heuristics, confidence, risk levels, grouping, diff body safety

### Changed
- Agent requests now include `git_context` with safe branch/HEAD/file/risk metadata
- Manifest validator enforces semantic commit preparation constraints
- `semantic_prep` module exports `prepare_semantic_commit`, `build_semantic_commit_input`, `build_agent_context`

### Security
- Secret-risk files are flagged by path only — contents never read
- Binary, large, and symlink files excluded from content analysis
- Agent context is content-free (paths + risk flags only, no file contents)
- Diff body pilot disabled by default; bounded to 256 KiB text files with normal risk
- Proposal always sets `write_operations: false` and `requires_review: true`

### Known Limitations
- Native commit execution is not implemented
- Push, pull, merge, and network Git operations remain on existing human flow
- AI-assisted commit message generation is backlog
- Diff body pilot remains disabled by default

## [1.2.1] - 2026-06-04

### Added
- Expanded native Git snapshot parity: staged+unstaged same file, added file, renamed file (with original_path), nested directories, ignored files, multiple simultaneous changes
- Hardened risk classification: `*.crt`, `*.cer`, `*_rsa`, `*_ed25519`, `secrets.*`, `credentials.*`, `.github/actions/**`
- Symlink detection: symlinks classified as `unknown` risk with explicit reason, never `normal`
- Large file detection: files > 10 MiB classified as `large` by metadata without content sampling
- Binary file detection: null-byte sampling verified for PNG files
- Merge commit support: commit metadata reads show all parents (not just first)
- Rename entry support: porcelain v2 `2 ` prefix parsed with `original_path` in DTO
- Untracked file listing: `-u` flag ensures individual files shown (not just directories)
- Commit metadata edge cases: multiline messages (title only), unicode authors
- Diff/stat robustness: files with spaces, binary files, deleted files, multiple files
- Manifest `risk_classification` sub-object with validator enforcement
- Diagnostics UI: risk counts, last refresh time, known limitations link

### Changed
- `GitChangedFile` DTO gains optional `original_path` field for renames
- `detect_file_kind` returns tuple `(file_kind, kind_reason)` for symlink reason propagation
- Manifest validator enforces `risk_classification.secret_paths === "path-only"`, `symlink_following === false`
- Native Git parity upgraded to `verified-expanded-cases`

### Security
- Certificate-like paths (`.crt`, `.cer`) classified conservatively with explicit risk_reason
- Symlinks never followed during classification; never classified as `normal` risk
- Secret-risk file contents never sampled
- Binary detection remains bounded to 8 KiB

### Known Limitations
- Push, pull, commit, and merge remain CLI/human-flow operations
- Diff/stat remains CLI-backed and labeled
- Native Git coverage remains limited to read-only verified cases

## [1.2.0] - 2026-06-04

### Added
- Repository snapshot command (`git_repository_snapshot_cmd`) — composite branch/HEAD/status/changed-files DTO
- Branch and HEAD metadata reads via gix (SHA, message, author, timestamp, detached detection)
- Changed-file summary with `file_kind` (text/binary/large/unknown) and `risk` (normal/secret/workflow/binary/large/unknown) classification
- Recent commit metadata reads with bounded limits (default 10, max 100) via gix traversal with CLI fallback
- Diff/stat read pilot (`git_diff_stat_cmd`) — CLI-backed, labeled, secret-safe
- Native Git diagnostics UI: `GitStatusPanel`, `GitDiagnosticsPanel`, `CommitSummaryPanel` components
- 29 new tests: snapshot, HEAD metadata, risk classification, commits, diff/stat, parity
- `docs/native-git.md` documentation
- Manifest `native_git` section with scope, operations, providers, fallback, parity, and secret safety

### Changed
- `git-bridge` crate gains `snapshot`, `commits`, and `diff` modules
- Manifest validator enforces `write_operations_migrated=false`, `network_operations_migrated=false`, `fallback_required=true`, `blocking=false`, `secret_safe=true`
- Provider renamed from `gix+cli` to `gix-hybrid` (v1.1.1 carry-forward)

### Security
- Secret-risk paths (.env, *.pem, *.key, id_rsa, id_ed25519) are flagged without reading contents
- Binary detection reads at most 8 KiB and never opens secret-risk files
- Symlinks are not followed during risk classification
- Diff/stat output never includes file contents

### Known Limitations
- Push, pull, commit, and merge operations remain CLI-backed
- Diff/stat is CLI-backed in v1.2.0 as a labeled read-only fallback
- Native Git coverage is limited to verified read-only cases

## [1.1.1] - 2026-06-04

### Added
- Bugfix-agent hardening: 7 new tests for disallowed paths, .env rejection, workflow rejection, empty response, auto-commit=false invariant, reject-no-change
- Native Git parity: `fallback_used` and `parity_status` fields in `NativeGitStatus` DTO
- Native Git parity: 4 new tests (branch/dirty parity, dirty repo, non-repo fallback, provider)
- Structured error coverage: 3 new tests (9 code check, .env redaction, auth header redaction)
- `credential_validation` manifest section with per-platform status and refined Linux wording
- `structured_errors` manifest section with code count and redaction status
- `native_git_pilot.parity` manifest field set to `verified-core-cases`

### Changed
- Native Git provider renamed from `gix+cli` to `gix-hybrid` (explicit about hybrid nature)
- Linux credential status uses `os-keychain-or-session-fallback` with environment notes
- Manifest validator enforces product-readiness values match evidence

### Fixed
- Native Git status DTO now includes `fallback_used` and `parity_status` for diagnostics

### Security
- Redaction tests verify .env content, Authorization headers, and long hex strings
- Token masking tests verify no raw PATs in DTO serialization

## [1.1.0] - 2026-06-03

### Added
- Product readiness manifest with credential provider, agents, and native Git pilot fields
- Manifest validator enforces product_readiness fields and rejects autonomous agent mode
- Native git status pilot: gix 0.84 branch detection + git CLI status counts (`git_status_cmd`)
- Structured error DTOs: 9 error codes with likely causes, suggested actions, and secret-safe guarantee
- First-run product guide (`FirstRunGuide.tsx`): checklist from repo open to optional AI mode
- AI mode status indicator (`AiModeStatus.tsx`): credential state, agent paths, review-only badge
- Bugfix agent review panel (`BugfixAgentPanel.tsx`): review-only diff with accept/reject
- `docs/product-readiness.md`: comprehensive product readiness documentation

### Changed
- Credential storage: validated OS-keychain path with 11 security tests (already existed, now tested)
- Manifest `product_readiness.credential_security_level` uses `platform-backed` (not `production-grade`)
- Native git status pilot is non-blocking by design (always falls back to git CLI)
- Bugfix agent proposals require human review; commits use normal Git flow (no auto-commit)

### Security
- Credential DTO tests verify no raw tokens in serialized output
- Token masking tests verify GitHub PAT patterns are redacted
- Error responses are secret-safe by design

### Known Limitations
- Windows installer unsigned (SmartScreen warnings expected)
- Linux tested on Ubuntu 24.04 only
- macOS has no bundled artifact
- Native Git status pilot: staged/unstaged counts come from CLI (gix lacks status API)
- Agents are review-only, not autonomous

## [1.0.12] - 2026-06-03

### Added
- Native Linux contributor smoke guide (`docs/linux-native-smoke-guide.md`)
- Linux smoke evidence template (`docs/linux-smoke-evidence-template.md`)
- Linux smoke report GitHub issue template (`.github/ISSUE_TEMPLATE/linux_smoke_report.yml`)
- `native-smoke-blocked` status in manifest validator
- `contributor_smoke_kit` manifest field linking guide, template, and issue form
- `previous_runtime_evidence` field preserving WSLg evidence

### Changed
- Linux status: `runtime-evidence-wslg` -> `tested` (native Ubuntu 24.04 GNOME smoke pass)
- Platform confidence documentation updated with contributor smoke path
- README links to Linux smoke guide for contributors

### Security
- SHA256 verification required for Linux AppImage
- Issue templates warn users not to share API keys, tokens, `.env` files, or secrets

### Known Limitations
- Linux AppImage tested on native Ubuntu 24.04 GNOME (QEMU/Proxmox)
- Linux screenshot blocked by Wayland rootless compositor; process+window evidence recorded
- Windows installer remains unsigned
- macOS remains build-feasibility-only

## [1.0.11] - 2026-06-03

### Added
- Linux AppImage runtime verification under WSLg (Windows Subsystem for Linux GUI)
- Linux runtime environment recording: Ubuntu 24.04 + WSLg (XWayland + Wayland)
- Linux WSLg smoke evidence: main window 1280x720, WebKit processes running, 384MB stable
- Screenshot of Orqestra running under WSLg
- `runtime-evidence-wslg` status in manifest validator
- `native-runtime-blocked`, `native-smoke-failed`, `native-smoke-blocked` statuses in validator
- `runtime_result`, `native_desktop_smoke` manifest fields
- Validator rules: WSLg status requires `smoke_tested: false`, `native_desktop_smoke: false`, `promotion_blocker`

### Changed
- Linux status: `runtime-blocked` -> `runtime-evidence-wslg` (WSLg runtime pass, not promoted)
- Platform confidence documentation updated with WSLg runtime evidence
- Linux troubleshooting updated with WSLg/GTK guidance

### Security
- SHA256 verification required for Linux AppImage
- Issue templates continue warning users not to share API keys, tokens, `.env` files, or secrets

### Known Limitations
- Linux AppImage runtime-evidence-wslg: passes under WSLg, not promoted
- Linux not promoted without native desktop smoke
- Windows installer remains unsigned
- macOS remains build-feasibility-only

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
