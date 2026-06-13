# Orqestra Development Lifecycle Specification

**Version:** 0.2.0-draft  
**Date:** 2026-06-13  
**Status:** Canonical design direction  
**Supersedes:** `product-team-mode-v0.1.2.md` (absorbed and expanded)  
**Inspiration:** Timovi (product team orchestration), addyosmani/agent-skills (engineering discipline), Orqestra operational evidence (v2.3.0–v2.14.11)

---

## 1. North Star

```
Orqestra is a repo-native development operating system that guides a product
from idea to release and learning, using bounded AI agents, typed artifacts,
human approval gates, and evidence-backed state.
```

Orqestra is not "a chat with agents." It is a **workflow operating surface**.

The user always sees:

```
Where am I in the lifecycle?
What artifact am I producing?
Which role is helping?
Which skill is active?
What is required before I can proceed?
What evidence exists?
What is the next safe action?
```

### What makes Orqestra distinct

| Framework | Core idea |
|-----------|-----------|
| Timovi | Simulate a product team inside an AI agent |
| agent-skills | Give AI agents disciplined engineering workflows |
| **Orqestra** | **Make the entire development lifecycle governable, inspectable, repeatable, and evidence-backed** |

---

## 2. Four Primitives

Orqestra is modeled around four primitives:

### 2.1 Lifecycle stages

```
0. Orient       — Understand the repo
1. Discover     — Understand the user/problem
2. Define       — Turn intent into product scope
3. Design       — Decide how it should work
4. Plan         — Break work into executable slices
5. Prepare      — Set up the work safely
6. Build        — Propose implementation
7. Verify       — Prove it works
8. Review       — Check quality, security, maintainability
9. Ship         — Release with integrity
10. Observe     — Collect post-release signal
11. Learn       — Convert signal into decisions
12. Evolve      — Plan the next cycle
```

### 2.2 Roles

```
Product Manager
UX Designer
Architect
Tech Lead
Implementation Agent
QA Agent
Security Reviewer
Release/Evidence Agent
```

### 2.3 Skills

```
Clarify          Refine           Specify          Break down
Research source  Question assumptions
Implement incrementally           Test
Review           Harden           Simplify
Document         Release          Observe          Learn
```

### 2.4 Evidence

Every important state change must be backed by:

- artifact
- event
- approval
- test/check result
- or explicit human override

---

## 3. Lifecycle Stages

Each stage has: purpose, typed artifacts, active roles/skills, and a gate.

### 0. Orient — Understand the repo

**Purpose:** What is this project? What exists? What is safe to touch?

**Artifacts:**
- `project-profile.json` — language, frameworks, tooling, build commands
- `architecture-map.md` — module relationships, entry points, data flow
- `stack-map.md` — dependencies, versions, known CVEs
- `repo-map.json` — directory structure, file types, LOC by area
- `conventions.md` — naming, structure, testing patterns detected
- `risk-map.md` — areas flagged as high-touch, fragile, or security-sensitive

**Roles:** Repo Analyst, Architect  
**Skills:** Source-driven analysis, Context engineering  

**Gate:** Human confirms project understanding before Orqestra plans work.

---

### 1. Discover — Understand the user/problem

**Purpose:** What problem are we solving, for whom, and why now?

**Artifacts:**
- `problem-brief.md` — problem statement, affected users, current pain
- `user-segment.md` — who, characteristics, constraints
- `constraints.md` — technical, timeline, resource, regulatory
- `assumptions.json` — explicit assumptions with confidence levels
- `open-questions.md` — unresolved questions that block definition

**Roles:** Product Manager, UX Researcher  
**Skills:** Interview/clarification, Idea refinement, Doubt-driven development  

**Gate:** No PRD until assumptions and open questions are explicit.

---

### 2. Define — Turn intent into product scope

**Purpose:** What exactly are we building, and what is out of scope?

**Artifacts:**
- `prd.md` — product requirements document
- `acceptance-criteria.json` — testable, binary pass/fail criteria
- `non-scope.md` — explicitly out of scope for this cycle
- `success-measures.md` — how we know it worked

**Roles:** Product Manager, UX Designer  
**Skills:** Spec-driven development  

**Gate:** Human approves PRD before design or task breakdown.

---

### 3. Design — Decide how it should work

**Purpose:** What is the user experience, system shape, and technical approach?

**Artifacts:**
- `ux-flow.md` — user journey, screen states, interaction patterns
- `interface-contracts.json` — API/type contracts between components
- `adr.md` — architecture decision record with alternatives considered
- `technical-design.md` — implementation approach, data model, migration path
- `security-boundaries.md` — trust boundaries, attack surface, mitigations

**Roles:** UX Designer, Architect, Security Reviewer  
**Skills:** API/interface design, Security and hardening  

**Gate:** Design must be reviewable before implementation planning.

---

### 4. Plan — Break work into executable slices

**Purpose:** How do we build this safely, in what order, with what checks?

**Artifacts:**
- `issue-graph.json` — typed issues with dependencies, typed as epic/story/task
- `task-slices.json` — ordered slices, each independently shippable
- `dependency-map.json` — what blocks what
- `test-plan.md` — what tests prove each slice works
- `rollout-plan.md` — release order, feature flags, rollback strategy

**Roles:** Tech Lead, QA Agent  
**Skills:** Planning and task breakdown, Incremental implementation, Test-driven development  

**Gate:** Human approves issue graph before patch proposals.

---

### 5. Prepare — Set up the work safely

**Purpose:** What files, commands, tests, and constraints apply to this slice?

**Artifacts:**
- `file-scope.json` — allowed files for this slice (enforced by Rust path guard)
- `command-plan.json` — build, test, lint commands
- `test-command-map.json` — which tests validate which changes
- `rollback-plan.md` — how to undo if the patch breaks something

**Roles:** Implementation Agent, QA Agent  
**Skills:** Git workflow and versioning, CI/CD and automation  

**Gate:** No patch proposal outside approved file scope.

---

### 6. Build — Propose implementation

**Purpose:** How should the repo change?

**Artifacts:**
- `patch-proposal.diff` — unified diff of proposed changes
- `implementation-notes.md` — what was done, why, trade-offs
- `changed-files.json` — list of changed files with change classification
- `rationale.md` — short decision summary (not chain-of-thought)

**Roles:** Implementation Agent, Frontend Agent, Backend Agent, DB Agent  
**Skills:** Incremental implementation, Source-driven development  

**Gate:** Patch proposal requires review. No auto-apply. No auto-commit.

---

### 7. Verify — Prove it works

**Purpose:** What evidence shows this works or does not work?

**Artifacts:**
- `test-results.json` — pass/fail counts, durations, command used
- `qa-report.md` — what was tested, what wasn't, known gaps
- `verification-log.json` — mechanical verification events
- `known-failures.md` — tests that fail, with classification

**Roles:** QA Agent  
**Skills:** Test-driven development, Browser/runtime verification, Debugging and recovery  

**Gate:** No status upgrade without mechanical evidence or explicit human override.

---

### 8. Review — Check quality, security, maintainability

**Purpose:** Is the work safe, simple, maintainable, and aligned with the spec?

**Artifacts:**
- `review-report.md` — code review findings, severity, recommendations
- `security-review.md` — boundary check, dependency audit, attack surface
- `simplification-report.md` — complexity reduction opportunities
- `unresolved-risks.json` — risks accepted with justification

**Roles:** Tech Lead, Security Reviewer  
**Skills:** Code review and quality, Code simplification, Security and hardening  

**Gate:** Human accepts or rejects review findings.

---

### 9. Ship — Release with integrity

**Purpose:** What exactly shipped, from which commit, with what artifacts and evidence?

**Artifacts:**
- `release-manifest.json` — version, commit, tag, artifacts
- `checksums.txt` — integrity hashes
- `release-notes.md` — human-readable changelog
- `artifact-list.json` — what was built and where
- `provenance.json` — build environment, CI run, signer

**Roles:** Release/Evidence Agent  
**Skills:** CI/CD and automation, Shipping and launch, Documentation and ADRs  

**Gate:** Release claims must match artifacts, tests, commit, tag, and evidence.

---

### 10. Observe — Collect post-release signal

**Purpose:** What happened after release?

**Artifacts:**
- `beta-feedback.json` — consented user feedback (friction, what worked, what didn't)
- `diagnostics-summary.json` — aggregate session outcomes
- `failure-taxonomy.json` — structured failure codes from real sessions
- `session-outcome.json` — steps completed, warnings, errors
- `user-friction-report.md` — qualitative friction analysis

**Roles:** QA Agent, Product Manager  
**Skills:** Observability and instrumentation  

**Gate:** No external evidence claim without consented, redacted, accepted evidence.

---

### 11. Learn — Convert signal into decisions

**Purpose:** What did we learn, and what changes because of it?

**Artifacts:**
- `learning-summary.md` — key findings, surprises, validated assumptions
- `accepted-evidence.json` — evidence bundles that passed review
- `rejected-evidence.json` — evidence bundles that failed, with reasons
- `decision-log.md` — what changes because of the evidence
- `roadmap-adjustment.md` — how the roadmap changes

**Roles:** Product Manager, Architect, Tech Lead  
**Skills:** Doubt-driven development  

**Gate:** Human decides whether evidence changes roadmap or release status.

---

### 12. Evolve — Plan the next cycle

**Purpose:** What is the next most important improvement?

**Artifacts:**
- `next-cycle-plan.md` — what to work on next and why
- `prioritized-backlog.json` — ordered by impact × confidence ÷ effort
- `debt-register.md` — technical debt acknowledged, with payoff criteria
- `risk-register.md` — known risks with mitigation status

**Roles:** Product Manager, Tech Lead, Architect  
**Skills:** Planning and task breakdown  

**Gate:** Next cycle begins only from accepted state, not agent enthusiasm.

---

## 4. Role Portfolio

### 4.1 Roles and their lifecycle stage bindings

| Role | Stages | Can | Cannot |
|------|--------|-----|--------|
| Product Manager | Discover, Define, Observe, Learn, Evolve | Generate PRD, user stories, acceptance criteria, prioritize | Edit source, modify release manifest, claim validation |
| UX Designer | Define, Design | Generate user flows, personas, interface contracts | Edit source, modify roadmap/issue files |
| Architect | Orient, Design, Learn, Evolve | Map modules, propose ADRs, assess feasibility | Apply patches without approval, modify project state |
| Tech Lead | Plan, Review, Learn, Evolve | Break down work, review findings, prioritize | Edit source directly, modify release manifest |
| Implementation Agent | Prepare, Build | Propose patches within allowed file scope | Commit, merge, publish, edit evidence claims |
| QA Agent | Plan, Verify | Generate test plan, run tests, produce QA report | Mark release validated, modify source |
| Security Reviewer | Design, Review | Read-only security analysis, boundary checks | Modify any file |
| Release/Evidence Agent | Ship, Observe | Assemble release/evidence metadata | Claim external evidence without accepted bundle |

### 4.2 Role enforcement

Every role has a typed permission declaration enforced by Rust:

```json
{
  "role_id": "product-manager",
  "lifecycle_stages": ["discover", "define", "observe", "learn", "evolve"],
  "allowed_inputs": ["project-profile", "feature-intake", "user-feedback"],
  "allowed_outputs": ["prd", "user-stories", "acceptance-criteria"],
  "allowed_file_scopes": {
    "read": [".Orqestra/lifecycle/", "docs/"],
    "write": [".Orqestra/lifecycle/features/<id>/prd.md"],
    "propose": []
  },
  "disallowed_actions": ["edit_source", "modify_release_manifest", "claim_validation"],
  "required_approval_gates": ["prd.approval"],
  "evidence_requirements": []
}
```

### 4.3 Deny-by-default semantics

- Absent permission = denied
- Unmatched path = denied
- Schema failure = rejected
- Missing gate = rejected

No implicit grants. Every permitted operation is explicitly declared.

---

## 5. State Architecture

### 5.1 Directory structure

```
.Orqestra/lifecycle/
├── project/
│   ├── project-profile.json      ← Orient output
│   ├── architecture-map.md       ← Orient output
│   ├── conventions.md            ← Orient output
│   └── risk-map.md               ← Orient output
├── features/
│   └── <feature-id>/
│       ├── intake/
│       │   ├── problem-brief.md
│       │   ├── assumptions.json
│       │   └── open-questions.md
│       ├── define/
│       │   ├── prd.md
│       │   ├── acceptance-criteria.json
│       │   └── non-scope.md
│       ├── design/
│       │   ├── technical-design.md
│       │   ├── adr.md
│       │   └── security-boundaries.md
│       ├── plan/
│       │   ├── issue-graph.json
│       │   ├── task-slices.json
│       │   └── test-plan.md
│       ├── build/
│       │   ├── patch-proposal.diff
│       │   └── implementation-notes.md
│       ├── verify/
│       │   ├── test-results.json
│       │   └── qa-report.md
│       ├── review/
│       │   ├── review-report.md
│       │   └── security-review.md
│       └── events.jsonl          ← Append-only event log for this feature
├── releases/
│   └── <version>/
│       ├── release-manifest.json
│       ├── release-notes.md
│       └── provenance.json
├── observations/
│   └── <evidence-id>/
│       ├── beta-feedback.json
│       └── session-outcome.json
├── learnings/
│   ├── decision-log.md
│   └── roadmap-adjustment.md
└── team/
    ├── role-registry.json
    └── skill-registry.json
```

### 5.2 Event-sourced state

State is derived from events, not agent-edited JSON.

```jsonl
{"event":"lifecycle.stage.entered","stage":"orient","feature_id":null,"timestamp":"2026-06-13T20:00:00Z","actor":"human"}
{"event":"artifact.created","type":"project-profile","path":".Orqestra/lifecycle/project/project-profile.json","timestamp":"...","actor":"repo-analyst"}
{"event":"gate.approved","gate":"orient.understanding_confirmed","timestamp":"...","actor":"human"}
{"event":"lifecycle.stage.advanced","from":"orient","to":"discover","timestamp":"...","actor":"human"}
```

The Rust backend:
- Validates every event before appending
- Derives current state from replaying events
- Never allows mutation of past events
- Rejects events that violate gate requirements

### 5.3 Backward compatibility

`.Orqestra/product-team/` (from PTM v0.1.2) is migrated to `.Orqestra/lifecycle/` on first launch of v2.15.0. The migration is one-way and logged.

---

## 6. Skill System

### 6.1 Skill registry

Skills are stored at `.Orqestra/lifecycle/team/skill-registry.json` and loaded at runtime based on lifecycle stage and active roles.

### 6.2 Lifecycle skill matrix

| Skill | Primary stages | Primary roles |
|-------|---------------|---------------|
| Source-driven analysis | Orient | Architect, Implementation Agent |
| Context engineering | All | All |
| Interview/clarification | Discover | Product Manager |
| Idea refinement | Discover | Product Manager |
| Spec-driven development | Define | Product Manager, UX Designer |
| Planning and task breakdown | Plan, Evolve | Tech Lead, Architect |
| Incremental implementation | Build | Implementation Agent |
| Test-driven development | Verify | Implementation Agent, QA Agent |
| Doubt-driven development | Verify, Learn | Architect, QA Agent |
| API/interface design | Design | Architect |
| Security and hardening | Design, Review | Security Reviewer |
| Code review and quality | Review | Tech Lead |
| Code simplification | Review | Tech Lead |
| Debugging and recovery | Verify | Implementation Agent, QA Agent |
| Git workflow and versioning | All | All |
| CI/CD and automation | Ship | Release/Evidence Agent |
| Documentation and ADRs | Design, Ship | Architect |
| Observability and instrumentation | Observe | Release/Evidence Agent |
| Shipping and launch | Ship | Release/Evidence Agent |

### 6.3 Rationalization guards

Every skill includes forbidden rationalizations — common excuses agents use to skip steps. The Rust backend rejects events that match forbidden rationalization patterns.

Example for `test-driven-development`:

| Rationalization | Rebuttal |
|----------------|----------|
| "I'll add tests later" | Tests are proof. No tests = no evidence. |
| "Manual testing is sufficient" | Manual tests are not reproducible. |
| "The logic is simple enough" | Simple logic still has edge cases. |

### 6.4 Autonomy boundary

| Allowed | Never allowed |
|---------|---------------|
| auto-suggest | auto-apply |
| auto-plan | auto-commit |
| auto-propose | auto-merge |

---

## 7. Product UI — Workflow Operating Surface

### 7.1 Design principle

The UI is not a chat. It is a structured work surface.

### 7.2 Lifecycle home view

The user always sees:

```
┌─────────────────────────────────────────────────────┐
│  Orqestra                              [Settings]    │
│  ─────────────────────────────────────────────────── │
│                                                      │
│  Lifecycle: ● Orient  ○ Discover  ○ Define  ...     │
│             ↑ you are here                           │
│                                                      │
│  ┌─ Project Profile ──────────────────────────────┐ │
│  │  Language: Rust + TypeScript                    │ │
│  │  Framework: Tauri v2                            │ │
│  │  Tests: 834 Rust + 50 Worker + 53 Dashboard     │ │
│  │  Risk areas: relay auth, evidence redaction     │ │
│  └─────────────────────────────────────────────────┘ │
│                                                      │
│  Active role: Architect                              │
│  Active skill: Source-driven analysis                │
│  Gate: Human must confirm understanding              │
│                                                      │
│  Next action: [Review architecture map]              │
│               [Confirm and advance → Discover]       │
│                                                      │
│  Evidence: 0 artifacts this cycle                    │
│  Artifacts: project-profile.json, architecture-map   │
└─────────────────────────────────────────────────────┘
```

### 7.3 What the user always knows

| Question | Answered by |
|----------|------------|
| Where am I? | Lifecycle stage indicator |
| What am I producing? | Active artifact panel |
| Who is helping? | Active role display |
| What skill is active? | Active skill display |
| What is required? | Gate status |
| What evidence exists? | Evidence counter + list |
| What should I do next? | Next action buttons |

---

## 8. Roadmap

### 8.1 Lifecycle coverage arc

| Version | Stages covered | Focus |
|---------|---------------|-------|
| v2.15.0 | Lifecycle shell + Orient + Discover + Define | Foundation |
| v2.16.0 | Design + Plan | Architecture and task breakdown |
| v2.17.0 | Prepare + Build | Patch proposals in lifecycle context |
| v2.18.0 | Verify + Review | Evidence-gated quality |
| v2.19.0 | Ship + Observe + Learn | Release and feedback loop |
| v2.20.0 | Evolve + Full A-to-Z | Complete lifecycle beta |

### 8.2 v2.15.0 — Lifecycle Foundation

**Acceptance bar:**

- [ ] Start lifecycle mode from first screen
- [ ] Open ordinary repo
- [ ] Generate or review project knowledge pack (Orient)
- [ ] Create feature/problem intake (Discover)
- [ ] Generate PRD draft (Define)
- [ ] Generate issue graph draft (Plan preview)
- [ ] Show active lifecycle stage, role, and skill
- [ ] Ask for human approval before advancing stages
- [ ] Store artifacts under `.Orqestra/lifecycle/`
- [ ] Export beta evidence (existing capability, preserved)
- [ ] Never auto-apply, auto-commit, or auto-merge

**What v2.15.0 does NOT include:**
- Source patching (v2.17.0)
- Test execution gating (v2.18.0)
- Release management (v2.19.0)
- External beta evidence claim (v2.20.0)

### 8.3 External beta timing

The external beta packet is **held** until v2.15.0 delivers the lifecycle concept. Rationale:

> Current v2.14.11 proves: the app can launch, not freeze, guide readiness, and export evidence. It does not yet prove: the user understands Orqestra as an A-to-Z development system. That is what v2.15.0 must deliver.

The v2.14.11 beta packet remains prepared. It will be sent with the v2.15.0 lifecycle build.

---

## 9. Non-Scope

| Out of scope | Reason |
|-------------|--------|
| Auto-commit | Always `false`. No exceptions, no role overrides. |
| Agent-generated roadmap files | `roadmap/` is human-only. Issue graphs live in `.Orqestra/lifecycle/features/`. |
| Live dashboard writes | All dashboard data is build-time static from evidence files. |
| External validation claims | Without accepted external evidence bundle. |
| Unbounded agent authority | No role can act outside its declared file scope. |
| Template character dynamics | Roles are capability-bounded agents, not personas. |
| Chat-first interface | Orqestra is a workflow surface, not a chatbot. |
| Replacing existing agents | Docs/bugfix/architect agents continue. Lifecycle mode is additive. |

---

## 10. Relationship to PTM v0.1.2

This specification **absorbs and supersedes** PTM v0.1.2. Specifically:

| PTM v0.1.2 concept | Lifecycle v0.2.0 |
|--------------------|--------------------|
| 5-stage pipeline (Intake→Plan→Design→Patch→Verify) | Expanded to 13 stages |
| `.Orqestra/product-team/` | Migrated to `.Orqestra/lifecycle/` |
| Event-sourced state | Preserved |
| Role permission model | Preserved, expanded bindings |
| 24-skill registry | Reorganized into lifecycle matrix |
| Deny-by-default | Preserved |
| No auto-commit | Preserved |
| Engineering Skill System (§10) | Absorbed into §6 |

PTM v0.1.2 remains in the repo as a historical reference. It is marked superseded.
