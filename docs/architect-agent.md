# Architect Agent (v1.9.0+)

## Overview

The architect agent is a **read-only planner** that produces structured architecture plans without modifying any files. It is the third and final agent in the Orqestra portfolio.

## Agent Portfolio

| Agent | Mode | Writes | Patch-Governed |
|-------|------|--------|---------------|
| docs-agent | review-only | via governance | yes |
| bugfix-agent | review-only, symbol-aware | via governance | yes |
| architect-agent | read-only planner | **no** | no |
| autonomy | disabled | — | — |

## How It Works

```
React UI → Tauri command → Rust builds context → Python AI endpoint → Structured plan
```

The frontend never calls the AI service directly. The Tauri command:
1. Builds Agent Context v2 (content-free, schema-versioned)
2. Extracts symbol summaries for changed files (max 20 files, 10 symbols each)
3. Finds existing ADRs with bounded metadata (max 10, 500-char excerpt)
4. Calls `POST /agent/architect`
5. Returns `ArchitectPlanResult`

## ArchitectPlanResult DTO

```json
{
  "plan_id": "arch-...",
  "schema_version": "architect-plan-v1",
  "summary": "...",
  "context_analysis": "...",
  "proposed_approach": ["step 1", "step 2"],
  "affected_symbols": [{"name": "...", "kind": "...", "file": "...", "is_public": true}],
  "risk_assessment": [{"risk": "...", "severity": "high|medium|low", "mitigation": "..."}],
  "dependency_warnings": ["..."],
  "acceptance_criteria": ["..."],
  "test_strategy": ["..."],
  "task_breakdown": [{"task": "...", "scope": "...", "complexity": "high|medium|low"}],
  "adr_draft": "... (optional)",
  "confidence": 0.85
}
```

### Structural Guarantee

The DTO has **no patch-shaped fields**: no `before`, `after`, `edits`, `path` (as edit target), `before_checksum`, or `after_checksum`. It cannot be passed to `apply_agent_patch_cmd`.

## ADR Context Bounding

Existing ADRs are passed as bounded metadata only:
- **path**, **title**, **status** — full
- **excerpt** — capped at 500 characters
- **limit** — max 10 ADRs

No unbounded ADR body content is included in the prompt.

## No Runtime Mock

Production behavior:
- Missing AI service → structured error (no fake plan)
- Network timeout → error
- Invalid response → error

Test behavior:
- Fixtures may mock HTTP responses
- Production fallback must never produce fake plans

## Non-Mutating

The architect agent:
- Does not write files
- Does not create patches
- Does not apply patches
- Does not create ADRs
- Does not mutate `.Orqestra` runtime state
- Does not modify the working tree

The `ArchitectAgentPanel.tsx` displays the plan with no accept/reject patch buttons.

## Context Sources

1. **Agent Context v2** — branch, HEAD SHA, changed files (paths + risk only), content policy
2. **Symbol summaries** — tree-sitter extracted symbols for changed files
3. **Risk summary** — file risk classifications
4. **Existing ADRs** — bounded metadata from roadmap directory

## Configuration

- Endpoint: `POST /agent/architect`
- AI service: `localhost:8000`
- API key: `ZAI_API_KEY` environment variable
- Model: `gpt-4o-mini` (via zukijourney.com)
- Timeout: 45 seconds

## Test Coverage

10 tests verify:
1. Plan structure (all required fields present)
2. No repository mutation (git status unchanged)
3. Missing AI service returns error (no fake plan)
4. Schema version present
5. Confidence bounded 0.0–1.0
6. No patch-shaped fields in DTO
7. ADR draft is optional
8. `.Orqestra` runtime state unchanged
9. Agent context wired (available for real repo)
10. Plan cannot be passed to patch governance (structurally incompatible)
