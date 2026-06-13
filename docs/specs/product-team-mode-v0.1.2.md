> **⚠️ SUPERSEDED** — This spec is absorbed and expanded by `orqestra-lifecycle-v0.2.0.md`.
> Retained as historical reference. All new work follows the lifecycle spec.

# Orqestra Product Team Mode Specification

**Version:** 0.1.2-draft  
**Status:** Superseded by lifecycle v0.2.0 — retained for reference
**Date:** 2026-06-13  
**Status:** Design baseline — implementation blocked until v2.15.0 external evidence gate is sealed  
**Scope:** Defines how Orqestra will support governed multi-agent product team workflows with an integrated engineering skill system  
**Supersedes:** Earlier exploratory auto-commit language. Product Team Mode never auto-commits. All source mutation flows remain proposal-first and human-approved.
**Source inspiration:** Timovi (product team orchestration), addyosmani/agent-skills (engineering discipline)

---

## 1. Thesis

Orqestra will perform the same product-team functions as skill-based frameworks (planning, specification, decomposition, implementation, verification), but with **enforceable repo-native governance, typed artifacts, human approval gates, and evidence-backed delivery**.

The product is not "AI coding governance." It is **"governed AI product delivery."**

Product Team Mode is the delivery orchestration. The Engineering Skill System is the engineering discipline layer used inside the delivery loop.

```
Product Team Mode = product delivery orchestration
Engineering Skill System = engineering discipline modules
```

---

## 2. Product Team Mode Overview

Product Team Mode is an Orqestra operating mode that turns a repository into a structured product delivery loop.

### Pipeline

```
Intake → Plan → Design → Patch → Verify
```

| Stage | Purpose | Produced |
|-------|---------|----------|
| **Intake** | Bootstrap project knowledge, accept feature request | Project Knowledge Pack, feature intake record |
| **Plan** | Multi-role planning with cross-examination | Plan notes, domain mapping, rationale |
| **Design** | PRD with approval gate | PRD, issue graph, QA plan |
| **Patch** | Role-bounded patch proposals | Patch proposals (existing governed mechanics) |
| **Verify** | QA evidence, review report, status update | Review report, evidence record |

### Entry conditions

- User activates Product Team Mode from the desktop control plane
- A repository is open and indexed
- Orqestra has a configured AI service (or degrades gracefully)

### Exit conditions

- Feature reaches `verified` or `paused` state
- Human explicitly closes the feature cycle
- All artifacts are stored in repo-native state

---

## 3. Role Portfolio

### 3.1 Minimum initial team (MVP)

| Role | Can | Cannot |
|------|-----|--------|
| **Product Manager** | Generate PRD, user stories, acceptance criteria, prioritize | Edit source code, change release manifest, claim validation |
| **UX Designer** | Generate user flows, personas, wireframe descriptions | Edit source code, modify roadmap/issue files |
| **Architect** | Map modules, propose ADRs, assess feasibility | Apply patches without approval, modify project state |
| **Implementation Agent** | Propose patches within allowed file scope | Commit, merge, publish, edit evidence claims, modify issue state |
| **QA Agent** | Generate test plan, run/check tests, produce QA report | Mark release externally validated, modify source code |
| **Release/Evidence Agent** | Assemble release/evidence metadata | Claim external beta evidence without accepted external bundle |

### 3.2 Extended team (post-MVP)

| Role | Added capabilities |
|------|--------------------|
| **Frontend Agent** | Scoped to UI component files only |
| **Backend Agent** | Scoped to API/service files only |
| **DB Agent** | Scoped to schema/migration files only |
| **DevOps Agent** | Scoped to CI/CD and infra config |
| **Security Reviewer** | Read-only analysis of security posture |
| **Marketing/Positioning Agent** | Scoped to marketing docs only |

### 3.3 Role enforcement model

Every role has a typed permission declaration:

```json
{
  "role_id": "product-manager",
  "allowed_inputs": ["project-knowledge-pack", "feature-intake", "user-feedback"],
  "allowed_outputs": ["prd", "user-stories", "acceptance-criteria"],
  "allowed_file_scopes": {
    "read": [".Orqestra/product-team/team/", ".Orqestra/product-team/features/", "docs/"],
    "write": [".Orqestra/product-team/features/<id>/prd.md", ".Orqestra/product-team/features/<id>/acceptance.md"],
    "propose": []
  },
  "disallowed_actions": ["edit_source", "modify_release_manifest", "claim_validation"],
  "required_approval_gates": ["prd.approval"],
  "evidence_requirements": []
}
```

This is enforced by the Rust backend via the existing `guard_path()` mechanism, extended with role-aware scope validation.

### 3.4 Key governance difference from skill-based approaches

Skill-based frameworks rely on prompt instructions like "always load Layer 0" and "never modify template files." Orqestra enforces:

- Which files a role may **read**
- Which files a role may **propose changes to**
- Which actions require **human approval**
- Which **artifacts are valid** (typed schema)
- Which **claims can appear on dashboard**
- Which **evidence is required** before status changes

Prompt discipline is a fallback, not the primary boundary.

### 3.5 Deny-by-default semantics

All role permission evaluation follows deny-by-default:

- If a role permission is **absent**, the operation is **denied**.
- If a file path does not match an explicit scope, the operation is **denied**.
- If schema validation fails, the artifact/event is **rejected**.
- If an approval gate is missing, the stage transition is **rejected**.

No implicit grants. No fallback allows. Every permitted operation is explicitly declared.

---

## 4. Typed Artifact Model

### 4.1 Directory structure

Product Team Mode state lives in `.Orqestra/product-team/` within the repository, using the existing Orqestra system directory convention (PascalCase `.Orqestra/`). This avoids introducing a second hidden state root and prevents case-sensitivity issues across macOS/Windows/Linux.

```
.Orqestra/product-team/
├── team/
│   ├── project-profile.json      ← typed project metadata
│   ├── domain-language.md        ← canonical domain terms
│   ├── architecture-map.md       ← module structure and dependencies
│   ├── stack-map.md              ← technology stack and versions
│   ├── conventions.md            ← coding and design conventions
│   └── team-config.json          ← active roles, role permissions
│
├── features/
│   └── <feature-id>/
│       ├── intake.md             ← feature request and context
│       ├── plan.md               ← planning notes with rationale
│       ├── prd.md                ← product requirements document
│       ├── issue-graph.json      ← typed dependency graph of issues
│       ├── qa-plan.md            ← test plan and acceptance criteria
│       ├── review-report.md      ← verification results
│       ├── evidence.json         ← typed evidence record
│       └── events.jsonl          ← append-only event log
│
├── memory/
│   ├── decisions/                ← ADRs and decision records
│   └── retrospectives/           ← feature retrospective notes
│
└── state.json                    ← current product team state
```

### 4.2 Persistence policy

Not all artifacts have the same persistence policy. Some are project truth committed to the repository; others are local-only traces.

**Committed to repository (project truth):**

| Artifact | Reason |
|----------|--------|
| `team/project-profile.json` | Canonical project metadata |
| `team/domain-language.md` | Shared domain vocabulary |
| `team/architecture-map.md` | Module structure reference |
| `team/stack-map.md` | Technology stack reference |
| `team/conventions.md` | Coding/design standards |
| `features/<id>/intake.md` | Feature request record |
| `features/<id>/plan.md` | Planning decisions |
| `features/<id>/prd.md` | Approved requirements |
| `features/<id>/issue-graph.json` | Approved decomposition |
| `features/<id>/qa-plan.md` | Test plan |
| `features/<id>/review-report.md` | Verification results |
| `features/<id>/evidence.json` | Feature evidence record |
| `features/<id>/events.jsonl` | Event log (if redacted and schema-clean) |

**Local-only or opt-in (not committed by default):**

| Artifact | Reason |
|----------|--------|
| `memory/` | May contain sensitive product information, retrospective details |
| Raw model traces | May contain prompts, partial outputs, internal reasoning |
| Private user notes | User-authored annotations not intended for commit |
| Temporary role scratchpads | Intermediate agent working state |
| Unredacted prompts/responses | May contain proprietary or sensitive context |

Local-only artifacts should be listed in `.gitignore` by default during bootstrap. Users may opt in to committing specific memory files if they choose.

### 4.3 Typed artifact schemas

**project-profile.json:**
```json
{
  "schema_version": 1,
  "project_name": "string",
  "description": "string",
  "repository_url": "string",
  "primary_language": "string",
  "frameworks": ["string"],
  "generated_at": "ISO 8601",
  "generated_from_commit": "40-char SHA"
}
```

**issue-graph.json:**
```json
{
  "schema_version": 1,
  "feature_id": "string",
  "issues": [
    {
      "id": "ISSUE-1",
      "title": "string",
      "status": "pending | in_progress | done | failed | blocked",
      "assigned_role": "string",
      "blocked_by": ["ISSUE-2"],
      "file_scope": ["path/to/allowed/files"],
      "approval_required": true,
      "created_at": "ISO 8601",
      "updated_at": "ISO 8601"
    }
  ],
  "rounds": [
    {
      "round": 1,
      "issues": ["ISSUE-1", "ISSUE-3"],
      "status": "pending | in_progress | done"
    }
  ]
}
```

**evidence.json:**
```json
{
  "schema_version": 1,
  "feature_id": "string",
  "pipeline_stage": "intake | plan | design | patch | verify",
  "artifacts_produced": ["prd.md", "issue-graph.json"],
  "tests_run": 0,
  "tests_passed": 0,
  "qa_status": "pending | passed | failed | partial",
  "external_validation": false,
  "review_status": "pending | approved | rejected | changes_requested",
  "human_approvals": [
    {
      "gate": "prd.approval",
      "actor": "human",
      "timestamp": "ISO 8601"
    }
  ]
}
```

### 4.3 Artifact validation

All typed artifacts must pass schema validation before being accepted by the Rust backend. This extends the existing `evidence_schema` module in `md-indexer`.

---

## 5. State Transition Model

### 5.1 Event-sourced state

State is derived from an append-only event log, not from agent-edited JSON.

**events.jsonl** (per feature):
```jsonl
{"event":"feature.intake.created","actor":"human","feature_id":"sync-auth","timestamp":"...","inputs":["user request"],"result":"intake_recorded"}
{"event":"feature.plan.started","actor":"orchestrator","feature_id":"sync-auth","timestamp":"...","roles":["pm","ux","architect"],"result":"planning_initiated"}
{"event":"feature.plan.completed","actor":"orchestrator","feature_id":"sync-auth","timestamp":"...","result":"plan_notes_saved"}
{"event":"feature.prd.generated","actor":"product-manager","feature_id":"sync-auth","timestamp":"...","result":"prd_draft"}
{"event":"feature.prd.approved","actor":"human","feature_id":"sync-auth","timestamp":"...","result":"approved"}
{"event":"feature.issue-graph.generated","actor":"tech-lead","feature_id":"sync-auth","timestamp":"...","issues":5,"result":"issue_graph_saved"}
{"event":"feature.issue-graph.approved","actor":"human","feature_id":"sync-auth","timestamp":"...","result":"approved"}
{"event":"feature.patch.proposed","actor":"implementation-agent","feature_id":"sync-auth","issue_id":"ISSUE-1","timestamp":"...","result":"patch_pending_review"}
{"event":"feature.patch.approved","actor":"human","feature_id":"sync-auth","issue_id":"ISSUE-1","timestamp":"...","result":"approved"}
```

### 5.2 Derived state

The current feature state is computed from events:

```json
{
  "feature_id": "sync-auth",
  "pipeline_stage": "patch",
  "current_round": 2,
  "issues_total": 5,
  "issues_done": 2,
  "issues_in_progress": 1,
  "issues_pending": 2,
  "prd_status": "approved",
  "qa_status": "pending",
  "last_event": "feature.patch.approved",
  "updated_at": "..."
}
```

Agents never mutate state directly. They emit events. The Rust backend validates events against the role permission model before appending.

### 5.3 Approval gates

| Gate | Required actor | Required before |
|------|---------------|-----------------|
| `intake.accepted` | human | Plan stage starts |
| `plan.completed` | orchestrator | Design stage starts |
| `prd.approval` | human | Issue graph generation |
| `issue-graph.approval` | human | Patch proposals |
| `patch.approval` | human | Patch application |
| `qa.report.accepted` | human | Verify stage completes |
| `feature.verified` | human | Feature closed |

---

## 6. File and Patch Permissions

### 6.1 Role file scopes

The existing `guard_path()` and `patch_guard.rs` modules are extended with role-aware scope validation:

```rust
fn role_allowed_path(role: &Role, operation: FileOp, path: &Path) -> bool {
    let scope = ROLE_SCOPES.get(role);
    match operation {
        FileOp::Read => scope.allowed_reads.matches(path),
        FileOp::Write => scope.allowed_writes.matches(path),
        FileOp::ProposePatch => scope.allowed_proposes.matches(path),
    }
}
```

### 6.2 Roadmap governance

The roadmap directory (`roadmap/`) is **never writable by any agent role**. Roadmap files are:

- Created by the issue graph decomposition (typed `issue-graph.json`)
- Updated only by human approval events
- Displayed in kanban/gantt/table views from the typed artifact

This directly addresses the self-referential scoreboard problem: agents cannot write their own tracking data.

### 6.3 Patch proposal constraints

Patch proposals use the existing `DiffReviewPanel` mechanics:

- Agents propose, humans review and accept/reject
- Accept/reject are labels, not mutations (v2.14.3+ principle)
- `guard_path()` enforces project-root boundary
- Role scope further constrains which files can appear in proposals

---

## 7. Dashboard Truth Surface

### 7.1 Product Team evidence on dashboard

**No Product Team Mode data appears on the public dashboard until all of the following are true:**

1. Events are schema-validated
2. Derived state is reproducible
3. Evidence records are generated mechanically
4. Human approval gates are recorded
5. Forbidden dashboard claims are tested

This invariant is non-negotiable and supersedes any version roadmap timing.

When the above conditions are met, the public dashboard extends to show Product Team Mode data:

| Panel | Data source | Truth boundary |
|-------|------------|----------------|
| **Feature Progress** | `.Orqestra/product-team/features/*/evidence.json` | Derived from events, not agent-edited |
| **Team Activity** | `.Orqestra/product-team/features/*/events.jsonl` | Append-only, immutable after write |
| **QA Status** | `.Orqestra/product-team/features/*/evidence.json` | Requires human approval gate |
| **Review Status** | `.Orqestra/product-team/features/*/evidence.json` | Requires human approval gate |

### 7.2 Dashboard non-claims

The dashboard must **never** claim:

- External validation that hasn't occurred
- Test results that weren't mechanically verified
- Feature completion without human approval gate
- Agent effectiveness metrics without structural evidence

### 7.3 Evidence flow

```
Agent emits event → Rust validates role permissions → Event appended to events.jsonl
→ Derived state computed → Evidence record generated → Dashboard export (build-time static)
```

No live API calls. No agent-writable dashboard data.

---

## 8. Evidence Model

### 8.1 Per-feature evidence

Each feature produces an `evidence.json` that records:

- What artifacts were generated
- Which roles participated
- Whether tests were run and passed
- Whether human approval gates were met
- Whether external validation occurred (always `false` until real external beta)

### 8.2 Evidence integrity

Evidence records:

- Are typed and schema-validated
- Are derived from event logs, not agent-authored summaries
- Cannot claim `external_validation: true` without an accepted external evidence bundle
- Cannot claim `qa_status: "passed"` without mechanically verified test results
- Are included in CI bundle scan for forbidden patterns

### 8.3 Relationship to existing evidence model

Product Team Mode evidence extends the existing `docs/evidence/` model. The existing evidence files (release-history, test-count-history, security-boundaries, etc.) remain the canonical project-level evidence. Feature-level evidence is additive.

---

## 9. Version Roadmap

### Pre-requisite

**v2.15.0 remains blocked on real external participant and accepted evidence bundle.**

No Product Team Mode implementation begins until v2.15.0 is sealed.

### Post-v2.15 arc

| Version | Scope | What it delivers |
|---------|-------|-----------------|
| **v2.16.0** | Product Team + Engineering Skill Foundations | Spec committed, directory/persistence policy finalized, typed artifact schemas, event type definitions, role permission declarations, engineering skill registry schema, rationalization guard model, evidence requirement model. No agent execution. No source patching. |
| **v2.17.0** | Intake + Knowledge Pack + Skill Router | Repo scan, domain-language draft, architecture-map draft, stack-map draft, conventions draft, human approval gate. Skill router suggests applicable engineering skills. No patch proposals. |
| **v2.18.0** | Plan + PRD + Issue Graph | Multi-role planning, spec-driven-development adapted, planning-and-task-breakdown adapted, PRD approval gate, issue graph approval gate, QA plan draft. No source patching. |
| **v2.19.0** | Role-Bounded Patch Proposals + Engineering Gates | Implementation agent proposal-only, role-aware patch guard, incremental-implementation adapted, test-driven-development adapted, source-driven-development adapted, doubt-driven-development adapted. No commit. No merge. |
| **v2.20.0** | QA / Review / Evidence / Dashboard Surface | QA agent, code-review-and-quality adapted, security-and-hardening adapted, performance review, review report, event-sourced evidence derivation, dashboard feature progress panel only after dashboard guard conditions are met. |

### Design-only phase

If work begins before v2.15.0 external beta evidence, it is limited to:

- This specification and refinements
- Typed artifact schema definitions
- Role permission model design
- State transition model design
- **No source code implementation**

---

## 10. Engineering Skill System

Product Team Mode uses an internal library of engineering process modules. These modules are not autonomous agents and not personas. They are **typed, versioned process definitions** that specify:

- When the skill applies
- Required inputs
- Required outputs
- Allowed role bindings
- Required approval gates
- Prohibited shortcuts
- Rationalization guards
- Evidence requirements

### 10.1 Relationship to agent-skills

The engineering skill system draws inspiration from [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills), which encodes 24 production-grade engineering workflows for AI coding agents. Orqestra absorbs the discipline but not the autonomy model.

Key difference: `agent-skills` supports `/build auto` — one plan approval, then autonomous implementation. Orqestra does not. The Orqestra autonomy boundary is:

```
auto-suggest:  allowed
auto-plan:     allowed after human intake approval
auto-propose:  allowed within file scope
auto-apply:    not allowed
auto-commit:   never
auto-merge:    never
```

### 10.2 Orqestra-native skill schema

Every engineering skill has a typed declaration:

```json
{
  "skill_id": "test-driven-development",
  "schema_version": 1,
  "stage_bindings": ["patch", "verify"],
  "role_bindings": ["implementation-agent", "qa-agent"],
  "required_inputs": ["issue-graph", "file-scope", "test-command"],
  "required_outputs": ["test-plan", "test-result", "evidence-event"],
  "approval_gates": ["patch.approval", "qa.report.accepted"],
  "forbidden_rationalizations": [
    "tests_later",
    "manual_only",
    "seems_right"
  ],
  "evidence_requirements": [
    "test_command_recorded",
    "result_captured",
    "human_override_if_missing"
  ]
}
```

### 10.3 Rationalization guards

Every skill includes a list of forbidden rationalizations — common excuses agents use to skip steps, with documented counter-arguments. The Rust backend rejects events that match forbidden rationalization patterns.

Example rationalization guard for `test-driven-development`:

| Rationalization | Rebuttal |
|----------------|----------|
| "I'll add tests later" | Tests are proof. No tests = no evidence. |
| "Manual testing is sufficient" | Manual tests are not reproducible. |
| "The logic is simple enough" | Simple logic still has edge cases. |
| "This is just a prototype" | Prototypes become production. |

### 10.4 Per-skill evidence requirements

Each skill declares what evidence it must produce before its stage is considered complete. Evidence requirements are mechanically verified by the Rust backend — not claimed by agent output.

Example for `security-and-hardening`:

- Dependency audit recorded
- OWASP Top 10 checklist addressed
- Boundary classification assigned
- No unresolved high-severity findings

### 10.5 Initial skill registry

| Skill | Stage binding | Role binding |
|-------|--------------|-------------|
| `using-engineering-skills` | All | All (meta-skill) |
| `interview-clarification` | Intake, Plan | Product Manager |
| `idea-refinement` | Intake, Plan | Product Manager |
| `spec-driven-development` | Design | Product Manager, Architect |
| `planning-and-task-breakdown` | Design | Tech Lead, Architect |
| `incremental-implementation` | Patch | Implementation Agent |
| `test-driven-development` | Patch, Verify | Implementation Agent, QA Agent |
| `context-engineering` | All | All |
| `source-driven-development` | Patch | Implementation Agent |
| `doubt-driven-development` | Patch, Verify | Architect, QA Agent |
| `frontend-ui-engineering` | Patch | Implementation Agent |
| `api-and-interface-design` | Design, Patch | Architect, Implementation Agent |
| `browser-runtime-verification` | Verify | QA Agent |
| `debugging-and-error-recovery` | Patch, Verify | Implementation Agent, QA Agent |
| `code-review-and-quality` | Verify | Tech Lead |
| `code-simplification` | Verify | Tech Lead |
| `security-and-hardening` | Verify | QA Agent, Architect |
| `performance-optimization` | Verify | QA Agent |
| `git-workflow-and-versioning` | All | All |
| `ci-cd-and-automation` | Verify | Release/Evidence Agent |
| `documentation-and-adrs` | Design, Verify | Architect |
| `observability-and-instrumentation` | Verify | Release/Evidence Agent |
| `shipping-and-launch` | Verify | Release/Evidence Agent |
| `deprecation-and-migration` | Design, Patch | Architect, Implementation Agent |

### 10.6 Skill registry storage

The skill registry is a typed artifact stored at:

```
.Orqestra/product-team/team/skill-registry.json
```

It is committed to the repository as project truth. Skills are loaded at runtime by the orchestrator based on pipeline stage and active roles.

---

## 11. Non-Scope

| Out of scope | Reason |
|-------------|--------|
| Auto-commit | Always `false`. No exceptions, no role overrides. |
| Agent-generated roadmap files | `roadmap/` is human-only. Issue graphs live in `.Orqestra/product-team/features/`. |
| Live dashboard writes | All dashboard data is build-time static from evidence files. |
| External validation claims | Without accepted external evidence bundle, dashboard says `external_validation: false`. |
| Unbounded agent authority | No role can act outside its declared file scope. |
| Template character dynamics | Roles are capability-bounded agents, not personas. No simulated personality. |
| Web research without provenance | Any external information used in planning must cite its source. |
| Replacing existing agent portfolio | Docs/bugfix/architect agents continue. Product Team Mode is additive. |

---

## 12. Open Questions

1. **Bootstrap depth** — How much knowledge pack can be mechanically generated vs. requiring user input? Recommendation: auto-detect language, frameworks, directory structure; user provides domain terms.

2. **Intake UX** — Wizard (multi-step) or single prompt? Recommendation: 3-5 question wizard with auto-fill from detection.

3. **Cross-examination implementation** — Sequential or parallel role questioning? Recommendation: sequential with turn-taking.

4. **Issue graph granularity** — Min/max issues per feature? Recommendation: 3-15. Below 3 = too small. Above 15 = split.

5. **Feature ID format** — Recommendation: slug + nanosecond timestamp.

6. **Memory model** — Recommendation: append-only markdown, 100-line split rule, indexed by feature ID.

---

## 13. Acceptance Bar for MVP (v2.16.0–v2.17.0)

- [ ] No source files modified without explicit human approval
- [ ] All generated artifacts are typed and stored in `.Orqestra/product-team/`
- [ ] All role outputs cite repo context (project knowledge pack)
- [ ] PRD requires human approval before issue graph generation
- [ ] Issue graph requires human approval before patch proposals
- [ ] Patch proposals use existing Orqestra patch guard
- [ ] QA plan links to real tests or explicitly marks missing tests
- [ ] Dashboard does not claim external validation
- [ ] Roadmap directory is never agent-writable
- [ ] Event log is append-only, never mutated
- [ ] State is derived from events, not agent-edited JSON
- [ ] All role file scopes enforced by Rust backend
- [ ] No auto-commit, ever
- [ ] Evidence remains `external_beta_user_data: false` until real external beta
