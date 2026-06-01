# Orqestra — Technical Specification & Implementation Guide
**Version:** 0.5.1-alpha  
**Date:** 2026-06-01  
**Status:** FINAL DRAFT — v0.5.1 implementation-ready. No further review cycles required.  

---

## Revision Notes — v0.5.1

This revision finalizes the Tauri-first desktop integration path. It explicitly treats in-process Tauri commands as the Phase 0–2 desktop boundary, keeps pure Rust crates free of Tauri dependencies, preserves a future sidecar/gRPC adapter path, and adds implementation guardrails for scaffold path verification and one-way command error serialization.

---

## 1. Executive Summary

Orqestra is a local-first, AI-native development environment where a Git repository is simultaneously:
- A **project management system** (Markdown-native Gantt/Kanban/roadmap)
- A **semantic knowledge graph** (every commit carries AI-generated intent and reasoning traces)
- An **autonomous agent workforce** (isolated AI workspaces that read the roadmap, edit code, and commit changes)

The system is built as a **polyglot architecture**: Rust powers the sync and storage core, TypeScript drives the desktop IDE and agent orchestration, Python handles LLM reasoning and embeddings, and Rust/WASM runs on the Cloudflare edge.

**Key architectural invariant:** The Git commit itself is never blocked by AI inference. The repository remains a valid Git repo at all times, even if the semantic layer is stale.

---

## 2. System Architecture

### 2.1 High-Level Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              DEVELOPER WORKSTATION                           │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         Orqestra DESKTOP (Tauri)                    │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │    │
│  │  │ Code Editor  │  │ PM Views     │  │ Agent Sidebar            │  │    │
│  │  │ (Monaco)     │  │ (Gantt/Kanban│  │ (Chat + Skills + Secrets)│  │    │
│  │  └──────┬───────┘  └──────┬───────┘  └───────────┬──────────────┘  │    │
│  │         │                 │                      │                 │    │
│  │         └─────────────────┼──────────────────────┘                 │    │
│  │                           ▼                                        │    │
│  │              ┌────────────────────────────┐                        │    │
│  │              │ TypeScript Agent Host       │                        │    │
│  │              │ + Tauri invoke() client     │                        │    │
│  │              └────────────┬───────────────┘                        │    │
│  └───────────────────────────┼────────────────────────────────────────┘    │
│                              │ Tauri commands                              │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │             TAURI RUST COMMAND LAYER + PURE CORE CRATES              │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐ │    │
│  │  │ CRDT Engine │  │ Git Bridge  │  │ Markdown Indexer / Graph    │ │    │
│  │  │ (Loro)      │  │ (gitoxide)  │  │ (pulldown-cmark + serde)    │ │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────────┘ │    │
│  │                                                                     │    │
│  │  Phase 0–2: in-process Tauri commands                               │    │
│  │  Phase 3+: optional sidecar/gRPC adapter for headless execution      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                              │                                              │
│                              ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                     PYTHON AI SERVICE (Local/Remote)                 │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐ │    │
│  │  │ ML-Master   │  │ Embeddings  │  │ Agent Reasoning API         │ │    │
│  │  │ (Explorer)  │  │ (sentence-  │  │ (FastAPI / gRPC)            │ │    │
│  │  │             │  │ transformers)│  │                             │ │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────────┘ │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │  CRDT Sync + Semantic Commits
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLOUDFLARE EDGE                                 │
│  ┌─────────────────────────────┐  ┌─────────────────────────────────────┐   │
│  │  Durable Objects (CRDT Relay)│  │  Pages: Public PM Dashboard         │   │
│  │  - Workspace state sync      │  │  - Gantt/Kanban from roadmap/      │   │
│  │  - Token-based access control│  │  - Read-only stakeholder view      │   │
│  └─────────────────────────────┘  └─────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │  WASM Workers (Rust)                                                   ││
│  │  - Semantic Query API: POST /query                                     ││
│  │  - Vector search (Vectorize)                                           ││
│  │  - Intent extraction from CRDT snapshots                               ││
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Communication Flow

| From | To | Protocol | Data |
|---|---|---|---|
| React renderer | Tauri command layer | `invoke()` | Roadmap indexing, task lookup, project open/close, Git actions |
| Tauri command layer | Rust core crates | In-process Rust calls | File CRUD, CRDT ops, Git commands, graph queries |
| Rust core crates | Python AI | HTTP (localhost) or gRPC | Exploration requests, embedding generation, intent extraction |
| Rust core crates | GitHub | HTTPS (`gitoxide`; libgit2 only if required) | Push, pull, PR creation |
| Desktop UI | Cloudflare | WebSocket / HTTPS | Real-time sync, dashboard queries |
| Rust core crates | Cloudflare | WebSocket | CRDT delta sync |
| Python AI | Cloudflare | HTTPS | Embedding upload, semantic commit indexing |
| Future sidecar adapter | Desktop / external clients | gRPC over Unix socket or named pipe | Headless runner, daemon mode, CI agent access |

### 2.2.1 Desktop Integration Decision: Tauri In-Process First, Sidecar-Ready

Because both the desktop shell and the core are Rust-capable, Orqestra has two viable desktop integration modes:

**Option A — Rust core as a Tauri sidecar.** The core runs as a separate process, Tauri spawns it on startup, and the TypeScript renderer talks to it over gRPC on a local socket. This is the right model when the same core must run headless in a GitHub Action, daemon, or alternate frontend.

**Option B — Rust core compiled directly into the Tauri app.** The core crates are linked into the Tauri app crate, and the TypeScript renderer calls Tauri commands, which call Rust functions directly in-process. This removes the Phase 0–2 gRPC dependency, reduces latency, and simplifies debugging.

**Decision:** Start with **Option B**, but design every core crate so it can later be wrapped by **Option A** without rewriting business logic. `md-indexer`, `graph-store`, `loro-engine`, and `git-bridge` must remain pure Rust libraries with zero Tauri dependencies. The Tauri app crate owns the desktop command layer. A future `rpc-server` crate can reuse the same core functions behind tonic/protobuf when headless execution becomes necessary.

### 2.3 Async Semantic Commit Pipeline (Critical Invariant)

The semantic commit object requires AI inference (intent extraction, risk assessment, embedding generation). This cannot block the Git commit. The pipeline is **asynchronous with optimistic stubs**:

```
Agent completes edits
  → Rust core stages changes (git add)
    → gitoxide writes STANDARD Git commit IMMEDIATELY
      → Semantic stub written: { hash, semantic: null, status: "pending_indexing" }
        → UI shows commit as "Indexing..."
          → Python AI receives diff (async queue / background job)
            → Generates intent, APIs, risk, confidence, embedding
              → Rust core backfills .Orqestra/graph/commits/{hash}.json
                → Triple store updated (content-addressed files)
                  → UI updates to "Indexed" 
                    → IF confidence < 0.70: UI flags commit for human review
```

**Commit latency target:** < 200ms for the Git operation. AI backfill target: < 5s local, < 15s cloud.

---

## 3. Data Models

### 3.1 Repository Layout

Every Orqestra has this mandatory structure:

```
my-project/
├── src/                          # Source code (language agnostic)
├── roadmap/                      # PM data — THE PROJECT DATABASE
│   ├── _index.md                 # Sprint/epic registry (coordinator)
│   ├── _team.md                  # Global team roster
│   ├── sprint-14.md              # Sprint definition
│   ├── epic-security.md          # Epic definition
│   ├── TASK-2026-042.md          # Individual task
│   └── ADR-011.md               # Architecture Decision Record
├── .Orqestra/                    # System directory
│   ├── .gitignore                # See Section 3.1.1 for rules
│   ├── crdt/                     # Loro CRDT snapshots (NOT committed)
│   ├── graph/                    # Knowledge graph (COMMITTED)
│   │   ├── commits/              # Semantic commit objects (one per hash)
│   │   └── triples/              # Content-addressed triple files
│   ├── embeddings/               # Vector cache (NOT committed)
│   └── agents/                   # Workspace state (NOT committed)
├── .github/
│   └── workflows/
│       └── Orqestra-agents.yml   # GitHub Actions agent triggers
└── Orqestra.toml                 # Repo configuration
```

#### 3.1.1 `.Orqestra/.gitignore` — Explicit Rules

```gitignore
# NEVER commit — binary, large, regeneratable, or local-only
crdt/
embeddings/
agents/
audit/

# ALWAYS commit — source of truth, queryable history
!graph/
!graph/commits/
!graph/triples/

# Config is committed
!Orqestra.toml
```

### 3.2 Task Schema (Markdown + YAML)

Every task is a `.md` file in `roadmap/`. This is the single source of truth.

```yaml
---
pm-task: true
id: TASK-2026-042
title: "Refactor auth middleware to use JWT"
type: Task           # Task | Subtask | Milestone | Epic
status: in-progress  # backlog | ready | in-progress | in-review | done | cancelled
priority: Critical   # Critical | High | Medium | Low
sprint: "Sprint 14"
epic: "Security Hardening"
assignee: "agent-architect"   # Human username or agent role
created: "2026-05-28T09:00:00Z"
updated: "2026-06-01T14:30:00Z"
due_date: "2026-06-15"
start_date: "2026-06-01"
time_estimate: 8h
time_logged: 3h
progress: 37          # 0-100
dependencies:
  - TASK-2026-038     # "Refactor DB layer"
  - TASK-2026-040     # "Add caching"
blocks:
  - TASK-2026-045     # "Update API docs"
labels:
  - backend
  - auth
  - refactor
---

## Context
The current session-based auth breaks in serverless contexts...

## Acceptance Criteria
- [ ] All routes protected by JWT middleware
- [ ] Backward compatibility for legacy sessions (2-week grace)
- [ ] Benchmark: <5ms overhead per request

## Agent Notes
> ML-Master: Explored 3 patterns. Selected "stateless JWT + refresh rotation" 
> based on ADR-011. See `/docs/adrs/011-auth-refactor.md`.
```

### 3.3 Epic & Sprint Schema

```yaml
---
pm-epic: true
id: epic-security
title: "Security Hardening"
status: in-progress
theme: "Q2 Reliability"
owner: "alice"
target_date: "2026-06-30"
---

## Goals
- Eliminate session-based auth
- Add rate limiting
- Pass SOC2 audit
```

```yaml
---
pm-sprint: true
id: "Sprint 14"
start_date: "2026-06-01"
end_date: "2026-06-14"
goal: "Ship auth refactor and caching layer"
velocity_target: 40h
---
```

### 3.4 Semantic Commit Object

Stored in `.Orqestra/graph/commits/{hash}.json`. One file per commit hash — never conflicts.

```json
{
  "hash": "a1b2c3d4e5f6",
  "parent_hashes": ["def456..."],
  "author": {
    "name": "agent-architect",
    "type": "agent",
    "workspace_id": "workspace/architect"
  },
  "timestamp": "2026-06-01T14:30:00Z",
  "conventional_message": "feat(auth): add JWT refresh rotation",
  "semantic": {
    "intent_summary": "Improve security by replacing stateful sessions with stateless JWTs",
    "reasoning_trace_id": "reasoning-uuid-123",
    "affected_concepts": ["authentication", "session-management", "security"],
    "affected_apis": ["POST /api/login", "POST /api/refresh", "GET /api/me"],
    "affected_tasks": ["TASK-2026-042"],
    "risk_assessment": {
      "breaking_change": true,
      "migration_required": "legacy_session_grace_period",
      "rollback_complexity": "low"
    },
    "confidence": 0.94,
    "vector_embedding": [0.023, -0.145, 0.892]
  },
  "crdt_snapshot_ref": ".Orqestra/crdt/snapshots/a1b2c3d.bin",
  "git_compatible": true
}
```

**Stub state (before AI backfill):**
```json
{
  "hash": "a1b2c3d4e5f6",
  "parent_hashes": ["def456..."],
  "author": { "name": "agent-architect", "type": "agent", "workspace_id": "workspace/architect" },
  "timestamp": "2026-06-01T14:30:00Z",
  "conventional_message": "feat(auth): add JWT refresh rotation",
  "semantic": null,
  "indexing_status": "pending",
  "git_compatible": true
}
```

### 3.5 Knowledge Graph Triples (Content-Addressed)

Stored as individual files in `.Orqestra/graph/triples/` — one file per write operation, keyed by UUID. This eliminates merge conflicts entirely because no two agents write to the same filename.

**Directory structure:**
```
.Orqestra/graph/triples/
├── 550e8400-e29b-41d4-a716-446655440000.json
├── 6ba7b810-9dad-11d1-80b4-00c04fd430c8.json
└── ...
```

**File format (one triple per file):**
```json
{
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "subject": "TASK-2026-042",
  "predicate": "implements",
  "object": "ADR-011",
  "commit": "a1b2c3d",
  "timestamp": "2026-06-01T14:30:00Z"
}
```

**Query layer:** The Rust graph-store crate reads all files in `triples/` at startup, deduplicates on `(subject, predicate, object, commit)`, and maintains an in-memory index. Writes are append-only (create new UUID file). Deletes are soft (write a `retracted` triple with the same UUID reference).

**Multi-process staleness:** The `TripleStore` uses a filesystem watcher on `triples/` to trigger index updates when new files appear. The watcher must be kept alive for the duration of the process. See Appendix B for implementation.

**Why content-addressed files:**
- No Git merge conflicts ever (different filenames)
- Parallel writes from multiple agents are safe
- Atomic: a triple either exists as a complete file or it doesn't
- Easy to audit: every triple has a creation timestamp and provenance

### 3.6 CRDT Document Scope & Coordinator Schema

**Decision:** One Loro CRDT document per `.md` file in `roadmap/`, plus one lightweight coordinator document for `_index.md`.

**Rationale:**
- Task files are edited independently 95% of the time
- Cross-file operations (reassigning a task to a different sprint) are rare and can use two-phase coordination via the coordinator document
- This avoids building a virtual filesystem on top of Loro, which is significant engineering

#### 3.6.1 `_index.md` Coordinator Schema

This is the only file where cross-file references are authoritative. The `sprint` and `epic` fields in individual task files are denormalized copies — convenient for reading a single file, but `_index.md` wins on conflict.

```yaml
---
pm-index: true
version: 1
sprints:
  - id: "Sprint 14"
    title: "Auth & Caching"
    tasks:
      - "TASK-2026-038"
      - "TASK-2026-040"
      - "TASK-2026-042"
    start_date: "2026-06-01"
    end_date: "2026-06-14"
    status: "active"
epics:
  - id: "epic-security"
    title: "Security Hardening"
    tasks:
      - "TASK-2026-042"
      - "TASK-2026-045"
    status: "in-progress"
    theme: "Q2 Reliability"
team:
  - id: "alice"
    role: "tech-lead"
  - id: "bob"
    role: "backend"
---
```

**Conflict resolution rule:** If a task file says `sprint: "Sprint 13"` but `_index.md` lists it under `Sprint 14`, the coordinator document is authoritative. The UI displays the coordinator's view and offers a one-click "sync from task file" action for manual reconciliation.

**Orphan rule:** Tasks found in `roadmap/` but absent from `_index.md` are displayed with status "unassigned" and surfaced in a dedicated "Unassigned Tasks" view. They are never hidden.

**Dangling reference rule:** Task IDs in `_index.md` with no corresponding `.md` file are a validation error. The indexer includes them in `IndexResult.errors`, and the UI shows a "Missing task file" warning.

**gRPC API impact:**
```protobuf
message FileId { string path = 1; }  // "roadmap/TASK-2026-042.md"

service OrquestraCore {
  rpc OpenFileCrdt(FileId) returns (CrdtDocument);
  rpc ApplyFileDelta(FileDelta) returns (Ack);
  rpc GetCoordinatorState(Empty) returns (IndexDocument);  // _index.md only
}
```

---

## 4. Component Specifications

### 4.1 Rust Core Crates (`md-indexer`, `graph-store`, `loro-engine`, `git-bridge`)

**Responsibilities:**
- CRDT document management (Loro)
- Git operations (gitoxide)
- Markdown indexing and graph construction
- Local sync server primitives (WebSocket-ready)
- Semantic commit export and graph persistence
- Reusable business logic that can be called by either Tauri commands or a future gRPC sidecar

**Boundary rule:** Core crates must have **zero Tauri dependencies**. They expose normal Rust APIs and serializable types. The Tauri app crate in `apps/desktop/src-tauri` depends on these crates; the crates never depend on the desktop shell.

**Crate Structure:**
```
orqestra/
├── crates/
│   ├── md-indexer/        # Pure Rust — pulldown-cmark + serde_yaml parsing
│   ├── graph-store/       # Pure Rust — content-addressed triples + SQLite cache
│   ├── loro-engine/       # Pure Rust — loro-crdt wrappers and workspace isolation
│   ├── git-bridge/        # Pure Rust — gitoxide wrappers, semantic commit export
│   └── rpc-server/        # Future adapter — tonic/protobuf sidecar server
└── apps/
    └── desktop/
        └── src-tauri/     # Tauri app crate; depends on the pure Rust crates
```

**Primary Phase 0–2 API:** Tauri commands in `apps/desktop/src-tauri/src/commands/*` call the pure Rust crates directly and return JSON-serializable DTOs to the TypeScript renderer.

**Future headless API (Phase 3+):** The same Rust functions are wrapped by `rpc-server` for gRPC:
```protobuf
service OrquestraCore {
  // CRDT
  rpc OpenWorkspace(OpenWorkspaceReq) returns (WorkspaceState);
  rpc ApplyLocalDelta(Delta) returns (Ack);
  rpc GetSnapshot(WorkspaceId) returns (Snapshot);

  // Git
  rpc SemanticCommit(SemanticCommitReq) returns (CommitResult);
  rpc CheckoutCommit(CommitHash) returns (WorkspaceState);
  rpc GetCommitGraph(CommitRange) returns (stream CommitNode);

  // Index
  rpc IndexRoadmap(IndexReq) returns (IndexResult);
  rpc QueryGraph(GraphQuery) returns (stream Triple);
  rpc SearchSemantic(SemanticQuery) returns (SearchResult);
}
```

**Build:**
```bash
cargo test --workspace
cargo build --release
```

### 4.2 TypeScript Frontend (`Orqestra-desktop`)

**Responsibilities:**
- Tauri desktop shell
- Monaco code editor
- PM view renderers (Gantt, Kanban, Table)
- Agent chat interface
- Skill/secret management UI
- GitHub PAT storage (encrypted at rest)
- Typed wrappers around Tauri `invoke()` commands

**Stack:**
- **Shell:** Tauri 2
- **Renderer:** React 19 + Vite + Tailwind CSS
- **State:** Zustand (local) + Loro bindings (sync)
- **Charts:** Custom Canvas Gantt + `@dnd-kit` Kanban
- **Native capabilities:** Tauri plugins, beginning with `@tauri-apps/plugin-dialog`

**Key Modules:**
```
apps/desktop/
├── package.json                 # Vite + React frontend
├── vite.config.ts
├── src/
│   ├── editor/                  # Monaco + file tree
│   ├── pm/
│   │   ├── GanttView.tsx
│   │   ├── KanbanView.tsx
│   │   ├── TableView.tsx
│   │   └── SmartScheduler.ts    # Dependency resolution, auto-schedule
│   ├── agent/
│   │   ├── AgentSidebar.tsx
│   │   ├── SkillManager.tsx
│   │   ├── SecretVault.tsx      # Encrypted API key storage
│   │   └── ConfidenceGate.ts    # See Section 4.5
│   ├── sync/
│   │   └── LoroProvider.tsx     # CRDT sync context
│   ├── bridge/
│   │   └── CoreClient.ts        # Future gRPC client, not needed in Phase 0–2
│   └── lib/
│       └── orqestra.ts          # Typed invoke() wrappers for Tauri commands
└── src-tauri/
    ├── Cargo.toml               # Tauri app crate; depends on md-indexer, etc.
    ├── tauri.conf.json
    └── src/
        ├── main.rs
        └── commands/
            ├── mod.rs
            └── roadmap.rs       # Tauri commands wrapping md-indexer
```

**First demoable UI:** `TaskTable.tsx` calls `indexRoadmap(projectRoot)`, renders `Vec<Task>` as a table, and displays parse warnings without failing the entire view. The UI must pin the Rust-to-TypeScript serialization contract with a Rust JSON-shape test before additional frontend logic depends on fields such as `time_estimate` or `time_logged`.

### 4.3 Python AI Service (`Orqestra-ai`)

**Responsibilities:**
- LLM reasoning (ML-Master exploration)
- Semantic intent extraction from diffs
- Embedding generation
- ADR (Architecture Decision Record) drafting

**Stack:**
- **Framework:** FastAPI + uvicorn
- **LLM:** OpenAI/Anthropic APIs (bring-your-own-key) + optional local vLLM
- **Embeddings:** `sentence-transformers` (all-MiniLM-L6-v2 for dev, larger for prod)
- **Exploration:** Custom ML-Master loop (iterative codebase search + reasoning)

**API Endpoints:**
```python
@app.post("/explore")
async def explore(request: ExploreRequest) -> ExplorationResult:
    # ML-Master long-horizon reasoning.
    # Input: task description + codebase snapshot
    # Output: plan, ADR draft, affected files, confidence score
    pass

@app.post("/extract-intent")
async def extract_intent(request: DiffRequest) -> SemanticIntent:
    # Given a diff + commit message draft, generate:
    # - affected_concepts, affected_apis, risk_assessment, confidence
    pass

@app.post("/embed")
async def embed(request: EmbedRequest) -> EmbeddingVector:
    # Generate vector for semantic search.
    pass

@app.post("/query-history")
async def query_history(request: HistoryQuery) -> HistoryAnswer:
    # Natural language query over commit history.
    # Uses vector search + graph traversal.
    pass
```

**Deployment:**
- Local: `python -m orqestra_ai` (runs on localhost:8000)
- Remote: Docker container with GPU access for local LLMs
- The Desktop app auto-detects local service; falls back to cloud endpoint

### 4.4 Agent Workspaces (`Orqestra-agents`)

**Responsibilities:**
- Isolate agent contexts (skills, memory, tools)
- Route tasks to appropriate agent personalities
- Execute tool calls (file edit, test run, git commit)

**Workspace Definition (YAML):**
```yaml
# .Orqestra/agents/bugfix/workspace.yml
id: agent-bugfix
personality: "You are a meticulous bug-fixing engineer."
model: "claude-sonnet-4"
skills:
  - ./skills/debugging/SKILL.md
  - ./skills/testing/SKILL.md
tools:
  - file_read
  - file_write
  - test_run
  - git_commit
  - semantic_commit
memory:
  type: session       # session | persistent | episodic
  max_tokens: 16000
secrets:
  - GITHUB_TOKEN
  - ANTHROPIC_API_KEY
confidence_gate:
  auto_commit: 0.85      # lower than default — bug fixes are bounded in scope
  propose:     0.65
  flag:        0.40
  breaking_change_override: always_propose  # overrides auto_commit for breaking changes
```

**Agent Router Logic:**
```typescript
// TypeScript orchestrator inside Desktop
class AgentRouter {
  async route(task: TaskFile): Promise<AgentResult> {
    const workspace = await this.loadWorkspace(task.labels);
    const context = await this.gatherContext(task);

    // Run agent loop
    const result = await workspace.agent.run({
      task: task.title,
      context,
      tools: workspace.config.tools,
    });

    // Resolve action using workspace-specific confidence gate
    const gate = new ConfidenceGate(workspace.config.confidence_gate);
    const action = gate.resolve(result.confidence, result.hasBreakingChange);

    if (action.type === 'auto_commit') {
      await this.semanticCommit(result.changes, task.id);
    } else if (action.type === 'propose') {
      await this.stageForReview(result.changes, task.id);
    } else if (action.type === 'flag') {
      await this.flagForHuman(task.id, action);
    } else {
      await this.abortAndAlert(task.id, action);
    }

    return result;
  }
}
```

### 4.5 Confidence Threshold Gate (`ConfidenceGate.ts`)

**Purpose:** Prevent agents with low confidence from auto-committing dangerous changes. Thresholds are **per-workspace with global defaults**.

```typescript
// apps/desktop/src/agent/ConfidenceGate.ts

export interface ConfidenceGateConfig {
  auto_commit: number;      // Commit immediately, notify human asynchronously
  propose: number;          // Stage commit, show diff in UI, require human approval
  flag: number;            // Log concern, assign back to human, do not stage
  breaking_change_override: 'always_propose' | 'always_flag' | 'respect_thresholds';
}

export const DEFAULT_GATE: ConfidenceGateConfig = {
  auto_commit:  0.90,
  propose:      0.70,
  flag:         0.50,
  breaking_change_override: 'always_propose',
};

export class ConfidenceGate {
  constructor(private config: Partial<ConfidenceGateConfig> = {}) {
    this.config = { ...DEFAULT_GATE, ...config };
  }

  resolve(confidence: number, hasBreakingChange: boolean): Action {
    // Breaking change override
    if (hasBreakingChange && this.config.breaking_change_override === 'always_propose') {
      return { type: 'propose', ui: 'diff_review_modal', reason: 'breaking_change' };
    }
    if (hasBreakingChange && this.config.breaking_change_override === 'always_flag') {
      return { type: 'flag', assignee: 'human_fallback', reason: 'breaking_change' };
    }

    // Normal threshold resolution
    if (confidence >= this.config.auto_commit!) {
      return { type: 'auto_commit', notify: 'async' };
    }
    if (confidence >= this.config.propose!) {
      return { type: 'propose', ui: 'diff_review_modal' };
    }
    if (confidence >= this.config.flag!) {
      return { type: 'flag', assignee: 'human_fallback' };
    }
    return { type: 'abort', alert: 'immediate' };
  }
}
```

**Rules:**
- Breaking changes NEVER auto-commit, regardless of confidence (unless `breaking_change_override: respect_thresholds`, which is NOT recommended).
- If the AI service fails to return a confidence score (network error, timeout), default to `propose`.
- All gate decisions are logged to the audit trail with the workspace ID and threshold values used.
- Workspace configs override global defaults. A `docs` agent may auto-commit at 0.80; an `architect` agent may require review even at 0.95.

### 4.6 Cloudflare Edge (`Orqestra-edge`)

**Responsibilities:**
- CRDT sync relay (Durable Objects)
- Public dashboard hosting (Pages)
- Semantic query API (WASM Workers)
- Authentication & token management

**WASM Worker (Rust):**
```rust
// workers-rs
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    match req.path().as_str() {
        "/query" => handle_semantic_query(req, env).await,
        "/sync" => handle_crdt_sync(req, env).await,
        _ => Response::error("Not found", 404),
    }
}
```

**Pages Dashboard:**
- Static site generated from `roadmap/` via GitHub Action
- Hosted on `https://{repo}-dashboard.pages.dev`
- Read-only for stakeholders; edit-capable for team members (via Worker auth)

---

## 5. APIs & Interfaces

### 5.1 Desktop Boundary: Tauri Commands (Phase 0–2)

The renderer calls Rust through Tauri `invoke()`. Commands live under `apps/desktop/src-tauri/src/commands/` and must return JSON-serializable DTOs, not raw internal error enums.

**Initial commands:**

| Command | Parameters | Returns | Purpose |
|---|---|---|---|
| `index_roadmap_cmd` | `{ projectRoot: string }` | `{ tasks: Task[], warnings: string[] }` | Parse `roadmap/` and return task records plus non-fatal parse warnings |
| `get_task` | `{ projectRoot: string, taskId: string }` | `Task \| null` | Fetch a single task by ID from the indexed roadmap |
| `semantic_commit` | `{ projectRoot, taskId, message }` | `CommitResult` | Create a normal Git commit plus semantic stub |
| `query_graph` | `{ projectRoot, subject?, predicate?, object? }` | `Triple[]` | Query content-addressed graph triples |

**Error contract:** Commands return `Result<T, CommandError>`, where `CommandError` has a stable `code` and human-readable `message`. `CommandError` must derive `Serialize` and should not derive `Deserialize`; errors flow one way, from Rust to the TypeScript rejected promise handler. Do not expose internal Rust enum variants directly to TypeScript.

### 5.2 Inter-Process: gRPC (Future Sidecar / Headless Mode)

The gRPC service is deferred until Orqestra needs a headless runner, daemon, GitHub Action integration, or alternate frontend. When implemented, it must wrap the same pure Rust core crate APIs used by Tauri commands.

**Port:** 50051 (local default)  
**Transport:** Unix socket (macOS/Linux) or named pipe (Windows)  
**Definition:** `proto/Orqestra.proto`  
**Service name:** `OrquestraCore` (consistent across all languages)

### 5.3 AI Service: HTTP/REST (Rust Core ↔ Python)

**Base URL:** `http://localhost:8000`  
**Auth:** None for local-only development; HMAC-signed requests for remote deployments.

### 5.4 Edge: Cloudflare Workers

**Endpoints:**

| Method | Path | Description | Auth |
|---|---|---|---|
| POST | `/sync/workspace/{id}` | CRDT delta relay | Bearer token |
| POST | `/query` | Semantic code history query | Bearer token |
| GET | `/dashboard/{repo}` | Public Gantt/Kanban | Optional (read-only) |
| POST | `/token` | Generate access token | Master token |

---

## 6. Implementation Roadmap

### Phase 0: Foundation (Weeks 1–3)
**Goal:** A Markdown-native PM system that runs locally inside a Tauri desktop shell.

**Revised build order (strict dependency order):**

| Order | Task | Duration | Output |
|---|---|---|---|
| 1 | `md-indexer` Rust crate: `roadmap/*.md` → `Vec<Task>` | 2–3 days | Validated data model |
| 2 | CLI tool: `orqestra deps --format=dot` prints dependency graph | 1–2 days | Forces cross-file graph resolution |
| 3 | Scaffold `apps/desktop` with Tauri 2 + React + Vite | 1 day | Native shell, renderer, and `src-tauri` crate wired into workspace |
| 4 | Tauri command layer: `index_roadmap_cmd` + `get_task` | 1 day | Typed desktop bridge without gRPC |
| 5 | Table view from `indexRoadmap(projectRoot)` | 1–2 days | First demoable milestone |
| 6 | Git sync: Shockwave-style PAT push/pull for `roadmap/` | 2 days | "Free" multi-device sync |
| 7 | Add Loro CRDT per-file for real-time collaboration | 3–5 days | Offline editing + merge |

- [ ] Set up monorepo with Rust + TypeScript + Python
- [ ] Implement `md-indexer` (Rust): parse `roadmap/*.md` into structured graph
- [ ] Scaffold the Tauri desktop app under `apps/desktop`
- [ ] Register `apps/desktop/src-tauri` as a Rust workspace member
- [ ] Implement Tauri commands that call `md-indexer` directly
- [ ] Pin the Rust-to-TypeScript task JSON shape with a serialization test
- [ ] Implement basic CRDT document for `roadmap/` (Loro) — one doc per file
- [ ] Build minimal Desktop UI: project picker, file tree, Markdown editor, and Table view
- [ ] Git sync: standard Git push/pull for `roadmap/` (Shockwave-style)

**Deliverable:** You can open a repo in the Tauri app, see tasks in a table, edit them, and push to GitHub.

### Phase 1: Intelligence (Weeks 4–6)
**Goal:** Add AI agent and semantic understanding.

- [ ] Integrate Python AI service (FastAPI) with local LLM calls
- [ ] Build Agent Sidebar in Desktop: chat interface, BYO API key
- [ ] Implement `extract-intent` endpoint: diff → semantic metadata
- [ ] Implement async semantic commit pipeline (stub → backfill)
- [ ] Implement ConfidenceGate module with per-workspace thresholds
- [ ] Agent can read task files and edit source code via tool calls
- [ ] First semantic commit: agent commits code + auto-updates task status

**Deliverable:** You can ask the agent to "fix the auth bug" and it edits code, writes a semantic commit, and marks the task done.

### Phase 2: Project Management (Weeks 7–9)
**Goal:** Full PM views with dependency tracking.

- [ ] Implement Gantt view (Canvas renderer) from task dates
- [ ] Implement Kanban view with drag-and-drop
- [ ] Smart scheduler: auto-adjust dependent task dates when blocker moves
- [ ] Dependency visualization in Gantt (arrows between tasks)
- [ ] Time tracking: estimate vs logged vs burndown

**Deliverable:** A project manager can plan a sprint, see Gantt charts, and watch the AI workforce execute it.

### Phase 3: Multi-Agent & Workspaces (Weeks 10–12)
**Goal:** Isolate agents, route tasks, prevent context corruption.

- [ ] Implement PilotDeck-style workspace isolation
- [ ] Agent router: label-based task routing (`bug` → bugfix agent, `docs` → docs agent)
- [ ] Workspace state persistence in `.Orqestra/agents/`
- [ ] Skill system: load `SKILL.md` files per workspace
- [ ] Multi-agent coordination: architect agent writes ADR, bugfix agent implements it

**Deliverable:** Three agents work on different tasks simultaneously without interfering.

### Phase 4: Semantic Git & History (Weeks 13–15)
**Goal:** Query the repository like a database.

- [ ] Build knowledge graph triple store (content-addressed files in Rust)
- [ ] Index every commit with semantic metadata
- [ ] Implement `query-history` in Python (vector + graph)
- [ ] Shockwave merge conflict UI: show intent + AI resolution proposal
- [ ] Semantic diff view: "Alice refactored for performance, Bob added OAuth"

**Deliverable:** You can ask "When did we introduce the memory leak?" and get a commit hash with reasoning trace.

### Phase 5: Cloud & Sync (Weeks 16–18)
**Goal:** Real-time collaboration and public dashboards.

- [ ] Deploy Cloudflare Durable Objects for CRDT relay
- [ ] Deploy WASM Workers for semantic query API
- [ ] Build Cloudflare Pages dashboard (Gantt/Kanban from `roadmap/`)
- [ ] Implement token-based access control (master/access tokens)
- [ ] Offline-first: full functionality without internet, sync on reconnect

**Deliverable:** A team of 5 can work offline on the same repo, sync via Cloudflare, and stakeholders view progress on a public URL.

### Phase 6: Self-Hosting (Weeks 19–20)
**Goal:** Dogfood. Orqestra manages its own development.

- [ ] Migrate project planning to `roadmap/` inside this repo
- [ ] Configure GitHub Actions to run agent fleet on issue creation
- [ ] Cloudflare dashboard public for community tracking
- [ ] Release v1.0

**Deliverable:** The repository that builds Orqestra is itself a Orqestra.

---

## 7. Development Environment Setup

### Prerequisites
- Rust 1.80+ (`rustup`)
- Node.js 22+ (`nvm`)
- Python 3.11+ (`pyenv`)
- Git 2.40+
- Cloudflare CLI (`wrangler`)

### Monorepo Structure
```
Orqestra-repo/
├── Cargo.toml                    # Rust workspace — see Appendix C
├── package.json                  # Root scripts (Turborepo or plain npm)
├── pyproject.toml                # Python workspace (uv/poetry)
├── proto/
│   └── Orqestra.proto            # Future gRPC definitions
├── crates/
│   ├── md-indexer/               # Pure Rust — no Tauri dependency
│   ├── graph-store/              # Pure Rust — no Tauri dependency
│   ├── loro-engine/              # Pure Rust — no Tauri dependency
│   ├── git-bridge/               # Pure Rust — no Tauri dependency
│   └── rpc-server/               # Future sidecar/headless adapter
├── apps/
│   ├── desktop/
│   │   ├── package.json          # Vite + React frontend
│   │   ├── src/                  # TypeScript/React source
│   │   ├── src-tauri/            # Tauri app crate; owns command layer
│   │   │   ├── Cargo.toml
│   │   │   ├── tauri.conf.json
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       └── commands/
│   │   │           ├── mod.rs
│   │   │           └── roadmap.rs
│   │   └── vite.config.ts
│   ├── dashboard/                # Cloudflare Pages site
│   └── edge-worker/              # WASM Worker (Rust)
├── services/
│   └── ai/                       # Python FastAPI service
└── agents/
    ├── skills/                   # Shared SKILL.md templates
    └── workspaces/               # Default workspace configs
```

### Quick Start
```bash
# 1. Clone
git clone https://github.com/your-org/Orqestra-repo.git
cd Orqestra-repo

# 2. Install Rust deps
cargo fetch

# 3. Install JS deps
npm install

# 4. Install Python deps
uv sync  # or: poetry install

# 5. Build everything
npm run build:all

# 6. Run locally
npm run dev
# Starts: Tauri desktop, Python AI service (localhost:8000), and any local sync workers
```

**Tauri desktop development:**
```bash
cd apps/desktop
npm run tauri dev

# From repo root — runs all Rust tests including md-indexer
cargo test --workspace

# Build release binary
cd apps/desktop
npm run tauri build
```

---

## 8. Testing Strategy

| Layer | Strategy | Tools |
|---|---|---|
| Rust Core | Unit + integration + property-based | `cargo test`, `proptest` |
| CRDT | Fuzz testing (concurrent edits) | `loom`, custom fuzzer |
| Git Bridge | Round-trip tests (commit → parse → commit) | `gitoxide` test fixtures |
| TypeScript UI | Component + E2E | `vitest`, `playwright` |
| Python AI | LLM output evals + regression | `pytest`, custom eval harness |
| Integration | Full stack: open repo → edit → sync → query | Docker Compose + `pytest` |

**Critical Test:** The "offline merge" test. Two clients edit the same task offline. They reconnect. The CRDT must converge to a valid state with no data loss.

---

## 9. Security & Privacy

- **API Keys:** Stored in OS keychain (macOS Keychain, Windows DPAPI, Linux Secret Service). Never in plain text.
- **Agent Isolation:** Each workspace runs in a separate process with filesystem sandboxing (Tauri isolation or OS-level).
- **Cloudflare Tokens:** Master token generates revocable access tokens. No long-lived credentials.
- **CRDT Privacy:** Workspace data encrypted at rest (AES-256) before sync to Cloudflare.
- **Audit:** Every agent action logged in `.Orqestra/audit/{timestamp}.jsonl`.

### 9.1 Enhanced Audit Log Format

```jsonl
{"ts":"2026-06-01T14:30:00Z","agent":"agent-architect","workspace":"workspace/architect","action":"file_write","file":"src/auth/middleware.ts","task":"TASK-2026-042","confidence":0.94,"model":"claude-sonnet-4","latency_ms":2340}
```

**Fields:**
- `ts`: ISO 8601 timestamp
- `agent`: Agent ID
- `workspace`: Workspace ID
- `action`: Operation type (file_write, git_commit, semantic_index, etc.)
- `file`: Affected file path (if applicable)
- `task`: Linked task ID (if applicable)
- `confidence`: Agent confidence score (if applicable)
- `model`: LLM model used (if applicable)
- `latency_ms`: Inference latency (if applicable)

---

## 10. Glossary

| Term | Definition |
|---|---|
| **CRDT** | Conflict-free Replicated Data Type. Guarantees convergence without coordination. |
| **Semantic Commit** | A commit object enriched with AI-generated intent, affected APIs, and reasoning traces. |
| **Workspace** | An isolated agent context with its own skills, memory, and tool permissions. |
| **ADR** | Architecture Decision Record. A Markdown file documenting a significant design choice. |
| **Loro** | The Rust CRDT library used for real-time sync. |
| **PilotDeck** | Pattern of isolating agents into separate workspaces to prevent context corruption. |
| **ML-Master** | Long-horizon reasoning agent that explores codebases and generates architectural plans. |
| **Confidence Gate** | The threshold system that prevents low-confidence agent actions from auto-executing. |
| **Content-Addressed Triples** | Individual triple files keyed by UUID to eliminate Git merge conflicts. |

---

## 11. Next Steps

1. **Create the monorepo** with the structure above.
2. **Implement `md-indexer`** (Rust) — this is the lowest-risk, highest-value first step.
3. **Build a proof-of-concept Desktop app** that can render a Gantt chart from `roadmap/*.md` files.
4. **Integrate one agent** (e.g., a documentation agent) that can read a task and update README.md.
5. **Write the first ADR** for the project: "Why we chose Rust + TS + Python."

---

## Appendix A: Async Semantic Commit Pipeline — Sequence Diagram

```
┌─────────┐     ┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌──────────┐
│  Agent  │     │ Rust Core   │     │  gitoxide    │     │ Python AI   │     │   UI     │
└────┬────┘     └──────┬──────┘     └──────┬───────┘     └──────┬──────┘     └────┬─────┘
     │                 │                   │                   │               │
     │ Completes edits │                   │                   │               │
     │────────────────>│                   │                   │               │
     │                 │ git add           │                   │               │
     │                 │──────────────────>│                   │               │
     │                 │                   │                   │               │
     │                 │ git commit        │                   │               │
     │                 │──────────────────>│                   │               │
     │                 │                   │                   │               │
     │                 │ Write stub        │                   │               │
     │                 │ .Orqestra/graph/  │                   │               │
     │                 │ commits/{hash}.json│                  │               │
     │                 │ (status: pending) │                   │               │
     │                 │────────────────────────────────────────>│               │
     │                 │                   │                   │               │
     │                 │ Return hash       │                   │               │
     │                 │<──────────────────│                   │               │
     │                 │                   │                   │               │
     │                 │                   │                   │               │ Show "Indexing..."
     │                 │────────────────────────────────────────────────────────>│
     │                 │                   │                   │               │
     │                 │                   │   Async queue     │               │
     │                 │────────────────────────────────────────>│               │
     │                 │                   │                   │               │
     │                 │                   │   Extract intent  │               │
     │                 │                   │   Generate embedding│               │
     │                 │                   │   Assess risk     │               │
     │                 │                   │<──────────────────│               │
     │                 │                   │                   │               │
     │                 │   Backfill        │                   │               │
     │                 │   .Orqestra/graph/│                   │               │
     │                 │   commits/{hash}.json│                  │               │
     │                 │   (full semantic) │                   │               │
     │                 │<────────────────────────────────────────│               │
     │                 │                   │                   │               │
     │                 │ Update triple store│                  │               │
     │                 │ (content-addressed)│                  │               │
     │                 │                   │                   │               │
     │                 │                   │                   │               │ Show "Indexed"
     │                 │────────────────────────────────────────────────────────>│
     │                 │                   │                   │               │
     │                 │                   │                   │               │ IF confidence < 0.70
     │                 │                   │                   │               │ Show "Review Required"
     │                 │                   │                   │               │
```

**Latency Budget:**
- Git commit: < 200ms (synchronous)
- AI backfill (local LLM): < 5s (asynchronous)
- AI backfill (cloud LLM): < 15s (asynchronous)
- UI update: immediate (optimistic stub) + async refresh

---

## Appendix B: Content-Addressed Triple Store Design

### Why not a single `triples.ndjson` file?

A single append-only file causes Git merge conflicts when two branches append to the same file. Git's line-based diff algorithm cannot merge two independent appends cleanly.

### Content-addressed solution

```
.Orqestra/graph/triples/
├── 550e8400-e29b-41d4-a716-446655440000.json
├── 6ba7b810-9dad-11d1-80b4-00c04fd430c8.json
└── ...
```

**Properties:**
- **No merge conflicts:** Different agents always write different UUID filenames
- **Atomic writes:** A triple either exists as a complete file or it doesn't
- **Parallel-safe:** Multiple processes can write simultaneously
- **Deduplication:** Query layer deduplicates on `(subject, predicate, object, commit)`
- **Soft deletes:** Write a retraction triple rather than deleting files

### TripleStore Implementation (with filesystem watcher)

```rust
// graph-store/src/lib.rs
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use dashmap::DashMap;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triple {
    pub uuid: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub commit: String,
    pub timestamp: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GraphStoreError {
    #[error("Serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("IO error on {0}: {1}")]
    Io(PathBuf, #[source] std::io::Error),
}

pub struct TripleStore {
    root: PathBuf,
    index: DashMap<(String, String, String), Vec<Triple>>,
    _watcher: Option<RecommendedWatcher>, // kept alive to keep watching
}

impl TripleStore {
    /// Load from filesystem, building in-memory index.
    /// Gracefully skips corrupted files rather than panicking.
    pub fn load(root: PathBuf) -> Self {
        let index: DashMap<(String, String, String), Vec<Triple>> = DashMap::new();

        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<Triple>(&content) {
                        Ok(triple) => {
                            let key = (triple.subject.clone(), triple.predicate.clone(), triple.object.clone());
                            index.entry(key).or_default().push(triple);
                        }
                        Err(e) => {
                            tracing::warn!("Skipping corrupted triple file {:?}: {}", path, e);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Cannot read triple file {:?}: {}", path, e);
                    }
                }
            }
        }

        Self { root, index, _watcher: None }
    }

    /// Load with filesystem watcher for multi-process staleness detection.
    /// The watcher must be kept alive for the duration of the process.
    pub fn load_with_watch(root: PathBuf) -> (Self, RecommendedWatcher) {
        let store = Self::load(root.clone());
        let index = store.index.clone();

        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(notify::Event { kind: notify::EventKind::Create(_), paths, .. }) = res {
                for path in paths {
                    if path.extension().and_then(|s| s.to_str()) != Some("json") {
                        continue;
                    }
                    match fs::read_to_string(&path) {
                        Ok(content) => match serde_json::from_str::<Triple>(&content) {
                            Ok(triple) => {
                                let key = (triple.subject.clone(), triple.predicate.clone(), triple.object.clone());
                                index.entry(key).or_default().push(triple);
                                tracing::debug!("Indexed new triple from {:?}", path);
                            }
                            Err(e) => tracing::warn!("New triple file {:?} is corrupted: {}", path, e),
                        }
                        Err(e) => tracing::warn!("Cannot read new triple file {:?}: {}", path, e),
                    }
                }
            }
        }).expect("Failed to create filesystem watcher");

        watcher.watch(&root, RecursiveMode::NonRecursive)
            .expect("Failed to watch triples directory");

        (store, watcher)
    }

    /// Insert a new triple. Writes to disk atomically (write-then-rename) and updates in-memory index.
    /// Returns Result so callers can propagate failures to the audit log.
    pub fn insert(&self, triple: Triple) -> Result<(), GraphStoreError> {
        let final_path = self.root.join(format!("{}.json", triple.uuid));
        let tmp_path = self.root.join(format!(".tmp-{}.json", triple.uuid));

        let content = serde_json::to_string_pretty(&triple)
            .map_err(GraphStoreError::Serialize)?;

        // Write to temp file first (atomic on all POSIX + Windows with MoveFileEx)
        fs::write(&tmp_path, &content)
            .map_err(|e| GraphStoreError::Io(tmp_path.clone(), e))?;

        // Atomic rename into place
        fs::rename(&tmp_path, &final_path)
            .map_err(|e| GraphStoreError::Io(final_path.clone(), e))?;

        // Only update in-memory index after successful disk write
        let key = (triple.subject.clone(), triple.predicate.clone(), triple.object.clone());
        self.index.entry(key).or_default().push(triple);

        Ok(())
    }

    /// Query by subject, predicate, and/or object. Uses DashMap index for O(1) lookups.
    pub fn query(
        &self,
        subject: Option<&str>,
        predicate: Option<&str>,
        object: Option<&str>,
    ) -> Vec<Triple> {
        // If all three are specified, use direct index lookup
        if let (Some(s), Some(p), Some(o)) = (subject, predicate, object) {
            return self.index
                .get(&(s.to_string(), p.to_string(), o.to_string()))
                .map(|v| v.clone())
                .unwrap_or_default();
        }

        // Otherwise, scan the index (still fast for reasonable sizes)
        let mut results = Vec::new();
        for entry in self.index.iter() {
            let (k, v) = entry.pair();
            let (s, p, o) = k;
            if subject.map_or(true, |q| s == q)
                && predicate.map_or(true, |q| p == q)
                && object.map_or(true, |q| o == q)
            {
                results.extend(v.iter().cloned());
            }
        }
        results
    }
}
```

**Key implementation notes:**
- `insert()` uses write-then-rename for atomicity across all POSIX systems and Windows
- `GraphStoreError` is a typed error enum with `thiserror` for clean error propagation
- The filesystem watcher sees the `rename` as a `Create` event on `final_path` and indexes it correctly
- `.tmp-*.json` files are ignored by the `.json` extension filter — the watcher never sees them
- Graceful error handling: corrupted files are skipped with `tracing::warn!` rather than panicking

**Startup cost:** Reading ~10,000 triple files takes < 100ms on SSD. For larger repos, add an SQLite cache that mirrors the in-memory index and is invalidated on file mtime change.

---

## Appendix C: Workspace Cargo.toml

```toml
# Cargo.toml (repo root)
[workspace]
members = [
    "crates/md-indexer",
    "crates/graph-store",
    "crates/loro-engine",
    "crates/git-bridge",
    "crates/rpc-server",
    "apps/desktop/src-tauri",
    "apps/edge-worker",
]
resolver = "2"

[workspace.dependencies]
# Pin shared deps here so all crates use the same versions
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
serde_yaml  = "0.9"
tokio       = { version = "1", features = ["full"] }
thiserror   = "1"
tracing     = "0.1"
uuid        = { version = "1", features = ["v4"] }
dashmap     = "6"
notify      = "6"

[profile.release]
lto = true
codegen-units = 1
```

**Scaffold path check:** `apps/desktop/src-tauri` is the expected Tauri 2.x scaffold output when running `npm create tauri-app@latest desktop` from `apps/`. Verify the generated directory before running `cargo check`. If the CLI creates a different layout, update the workspace member path and the Tauri crate path dependencies to match the actual scaffold.

```toml
# crates/md-indexer/Cargo.toml
[package]
name = "md-indexer"
version = "0.1.0"
edition = "2021"

[dependencies]
serde      = { workspace = true }
serde_yaml = { workspace = true }
thiserror  = { workspace = true }
pulldown-cmark = "0.11"
walkdir    = "2"
chrono     = { version = "0.4", features = ["serde"] }
tracing    = { workspace = true }
```

```toml
# crates/graph-store/Cargo.toml
[package]
name = "graph-store"
version = "0.1.0"
edition = "2021"

[dependencies]
serde      = { workspace = true }
serde_json = { workspace = true }
uuid       = { workspace = true }
dashmap    = { workspace = true }
notify     = { workspace = true }
tracing    = { workspace = true }
thiserror  = { workspace = true }
```

```toml
# apps/desktop/src-tauri/Cargo.toml
[package]
name = "orqestra-desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri      = { version = "2", features = [] }
serde      = { workspace = true }
serde_json = { workspace = true }
tracing    = { workspace = true }
thiserror  = { workspace = true }

# Core crates — the bridge between Tauri commands and business logic
md-indexer = { path = "../../../crates/md-indexer" }
# graph-store = { path = "../../../crates/graph-store" }   # uncomment in Phase 4
# loro-engine = { path = "../../../crates/loro-engine" }   # uncomment in Phase 0.5

# Native desktop capabilities
tauri-plugin-dialog = "2"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

**Workspace benefits:**
- `cargo test --workspace` runs all crates' tests in one command
- `cargo build -p md-indexer` builds only the indexer
- Shared dependency versions prevent unification bugs between crates using different feature sets of the same library
- `resolver = "2"` is required for correct feature unification in workspaces

---

## Appendix D: Implementation Sequence — This Week

**Day 1:** Paste Appendix C's `Cargo.toml` into repo root. Run `cargo new --lib crates/md-indexer`.

**Day 2:** Define the `Task`, `Sprint`, `Epic`, and `IndexResult` structs in `crates/md-indexer/src/lib.rs`. Implement YAML frontmatter extraction with `serde_yaml`.

**Day 3:** Implement the `walkdir`-based directory scanner. Collect all `.md` files in `roadmap/`, parse frontmatter, return `Vec<Task>`.

**Day 4:** Write unit tests against the sample task from Section 3.2. Assert that `TASK-2026-042` parses with the correct `dependencies`, `blocks`, and `time_estimate`.

**Day 5:** Add the CLI binary target: `orqestra deps --format=dot` that reads `roadmap/` and prints a DOT graph of task dependencies. This validates cross-file graph resolution.

**Day 6:** Run `cargo test -p md-indexer`. When green, the foundation is solid. The spec has earned its keep.

**Day 7:** Write the first ADR: `roadmap/ADR-001.md` — "Why we chose Rust + TS + Python for Orqestra." Commit it. The repository that builds Orqestra is now, in a small way, already a Orqestra.


## Appendix E: Tauri Desktop Integration — Phase 0 Implementation Pattern

### E.1 Day 1 Scaffold

Install the Tauri CLI:

```bash
cargo install tauri-cli --version "^2"
# or via npm:
npm install -g @tauri-apps/cli@next
```

Scaffold the desktop app:

```bash
cd apps
npm create tauri-app@latest desktop -- \
  --template react-ts \
  --manager npm \
  --tauri

cd desktop
npm install
```

Before editing `Cargo.toml`, confirm the generated Rust app path. For Tauri CLI 2.x, the expected path is `apps/desktop/src-tauri/`. If the scaffold differs, treat the actual generated folder as authoritative and update the workspace `members` entry plus all relative crate paths accordingly.

Wire the Tauri crate into the Rust workspace by adding `apps/desktop/src-tauri` to the root `Cargo.toml` workspace members. Then add the pure core crates, beginning with `md-indexer`, as path dependencies of the Tauri app crate.

### E.2 Tauri Command Layer

Tauri commands are the integration boundary. They call pure Rust library functions and return stable JSON DTOs to TypeScript.

```rust
// apps/desktop/src-tauri/src/commands/roadmap.rs
use md_indexer::{index_roadmap, Task, IndexerError};
use serde::Serialize;
use std::path::PathBuf;
use tauri::command;

/// Serializable error for the frontend.
/// Never expose internal IndexerError variants directly to TypeScript.
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

// `&'static str` serializes to a JSON string for the TypeScript catch handler.
// Keep this type Rust-to-TypeScript only; do not derive Deserialize.

impl From<IndexerError> for CommandError {
    fn from(e: IndexerError) -> Self {
        match e {
            IndexerError::DirectoryNotFound(_) => CommandError {
                code: "ROADMAP_NOT_FOUND",
                message: e.to_string(),
            },
            IndexerError::Io(_, _) => CommandError {
                code: "IO_ERROR",
                message: e.to_string(),
            },
            _ => CommandError {
                code: "PARSE_ERROR",
                message: e.to_string(),
            },
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Serialize)]
pub struct IndexRoadmapResult {
    pub tasks: Vec<Task>,
    pub warnings: Vec<String>,
}

/// Index the roadmap/ directory relative to the given project root.
/// Called from TypeScript as: invoke('index_roadmap_cmd', { projectRoot: '/path/to/project' })
#[command]
pub fn index_roadmap_cmd(project_root: String) -> CommandResult<IndexRoadmapResult> {
    let roadmap_dir = PathBuf::from(&project_root).join("roadmap");
    let result = index_roadmap(&roadmap_dir).map_err(CommandError::from)?;

    let warnings = result
        .errors
        .iter()
        .map(|(path, err)| format!("{}: {}", path.display(), err))
        .collect();

    Ok(IndexRoadmapResult {
        tasks: result.tasks,
        warnings,
    })
}

#[command]
pub fn get_task(project_root: String, task_id: String) -> CommandResult<Option<Task>> {
    let roadmap_dir = PathBuf::from(&project_root).join("roadmap");
    let result = index_roadmap(&roadmap_dir).map_err(CommandError::from)?;
    let task = result.tasks.into_iter().find(|t| t.frontmatter.id == task_id);
    Ok(task)
}
```

Register the commands:

```rust
// apps/desktop/src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::roadmap::index_roadmap_cmd,
            commands::roadmap::get_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

```rust
// apps/desktop/src-tauri/src/commands/mod.rs
pub mod roadmap;
```

### E.3 TypeScript Invoke Wrapper

```typescript
// apps/desktop/src/lib/orqestra.ts
import { invoke } from '@tauri-apps/api/core';

export type TaskStatus =
  | 'backlog' | 'ready' | 'in-progress'
  | 'in-review' | 'done' | 'cancelled';

export type Priority = 'Critical' | 'High' | 'Medium' | 'Low';

export interface TaskFrontmatter {
  id: string;
  title: string;
  status: TaskStatus;
  priority: Priority;
  sprint: string | null;
  epic: string | null;
  assignee: string | null;
  progress: number;
  dependencies: string[];
  blocks: string[];
  labels: string[];
  time_estimate: number | null;
  time_logged: number | null;
  due_date: string | null;
  start_date: string | null;
  created: string;
  updated: string;
}

export interface Task {
  frontmatter: TaskFrontmatter;
  body: {
    context: string | null;
    acceptance_criteria: Array<{ text: string; completed: boolean }>;
    agent_notes: string | null;
    raw: string;
  };
  source_path: string;
}

export interface IndexRoadmapResult {
  tasks: Task[];
  warnings: string[];
}

export interface CommandError {
  code: string;
  message: string;
}

export async function indexRoadmap(projectRoot: string): Promise<IndexRoadmapResult> {
  return invoke<IndexRoadmapResult>('index_roadmap_cmd', { projectRoot });
}

export async function getTask(projectRoot: string, taskId: string): Promise<Task | null> {
  return invoke<Task | null>('get_task', { projectRoot, taskId });
}
```

**Serialization warning:** A Rust tuple struct such as `Duration(480)` may serialize as a plain integer or tuple-like value depending on its serde implementation. Pin the exact shape with a Rust test before TypeScript logic depends on it.

### E.4 First Table View

```typescript
// apps/desktop/src/components/TaskTable.tsx
import { useEffect, useState } from 'react';
import { indexRoadmap, Task } from '../lib/orqestra';

const STATUS_COLORS: Record<string, string> = {
  'backlog': '#888',
  'ready': '#3b82f6',
  'in-progress': '#f59e0b',
  'in-review': '#8b5cf6',
  'done': '#10b981',
  'cancelled': '#ef4444',
};

export function TaskTable({ projectRoot }: { projectRoot: string }) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [warnings, setWarnings] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    indexRoadmap(projectRoot)
      .then(result => {
        setTasks(result.tasks);
        setWarnings(result.warnings);
      })
      .catch(err => setError(err.message ?? String(err)));
  }, [projectRoot]);

  if (error) return <div className="error">Error: {error}</div>;

  return (
    <div>
      {warnings.map((w, i) => (
        <div key={i} className="warning">⚠ {w}</div>
      ))}
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>Title</th>
            <th>Status</th>
            <th>Priority</th>
            <th>Sprint</th>
            <th>Assignee</th>
            <th>Progress</th>
          </tr>
        </thead>
        <tbody>
          {tasks.map(task => {
            const fm = task.frontmatter;
            return (
              <tr key={fm.id}>
                <td><code>{fm.id}</code></td>
                <td>{fm.title}</td>
                <td>
                  <span style={{ color: STATUS_COLORS[fm.status] }}>
                    {fm.status}
                  </span>
                </td>
                <td>{fm.priority}</td>
                <td>{fm.sprint ?? '—'}</td>
                <td>{fm.assignee ?? '—'}</td>
                <td>
                  <progress value={fm.progress} max={100} />
                  {' '}{fm.progress}%
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
```

### E.5 Project Picker and Dialog Plugin

```typescript
// apps/desktop/src/App.tsx
import { useState } from 'react';
import { TaskTable } from './components/TaskTable';
import { open } from '@tauri-apps/plugin-dialog';

export default function App() {
  const [projectRoot, setProjectRoot] = useState<string | null>(null);

  async function openProject() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === 'string') setProjectRoot(selected);
  }

  return (
    <div style={{ padding: '1rem' }}>
      {!projectRoot ? (
        <button onClick={openProject}>Open project folder</button>
      ) : (
        <>
          <div style={{ marginBottom: '0.5rem', color: '#666' }}>
            {projectRoot}
            <button onClick={() => setProjectRoot(null)} style={{ marginLeft: '1rem' }}>
              Close
            </button>
          </div>
          <TaskTable projectRoot={projectRoot} />
        </>
      )}
    </div>
  );
}
```

Install and register the plugin:

```bash
cd apps/desktop
npm install @tauri-apps/plugin-dialog
```

```toml
# apps/desktop/src-tauri/Cargo.toml
tauri-plugin-dialog = "2"
```

### E.6 Serialization Contract Test

```rust
// crates/md-indexer/src/parser.rs — tests module
#[test]
fn serializes_to_expected_json_shape() {
    let task = parse_task_content(TASK_2026_042, Path::new("test.md")).unwrap();
    let json = serde_json::to_value(&task).unwrap();

    assert_eq!(json["frontmatter"]["id"], "TASK-2026-042");
    assert_eq!(json["frontmatter"]["status"], "in-progress");
    assert_eq!(json["frontmatter"]["time_estimate"], 480);
    assert!(json["frontmatter"]["dependencies"].is_array());
}
```

If `time_estimate` serializes as `480`, TypeScript should use `number | null`. If it serializes as `[480]` or `{ minutes: 480 }`, adjust the TypeScript type to match the tested contract. This test is mandatory before building UI logic that depends on duration values.

---

*This specification is a living document. As implementation progresses, update sections with actual API schemas, performance benchmarks, and revised timelines.*
