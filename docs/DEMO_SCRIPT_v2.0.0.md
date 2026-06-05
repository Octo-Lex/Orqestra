# Demo Script v2.0.0

Deterministic external beta demo for Orqestra.

## Prerequisites

- Windows 10/11 x64
- Git installed
- (Optional) Python 3.11+ with `pip install fastapi uvicorn` for AI service

## Steps

### 1. Install App

Download `Orqestra_1.9.0_x64-setup.exe` from [GitHub Releases](https://github.com/Elephant-Rock-Lab/Orqestra/releases).

```powershell
# Verify SHA256
Get-FileHash .\Orqestra_1.9.0_x64-setup.exe -Algorithm SHA256
# Compare against checksums.txt from the release
```

Run the installer. Windows SmartScreen warning is expected — click "More info" → "Run anyway".

### 2. Open Sample Repo

Launch Orqestra. Click **"Try sample project"** or open the Orqestra repository itself.

Expected: Environment check panel shows 10 readiness probes.

### 3. Verify Environment Checks

The first-run panel should show:

| Check | Expected Status |
|-------|----------------|
| Git available | ✓ complete |
| Repository selectable | ✓ complete |
| Roadmap valid | ✓ complete |
| AI service reachable | ⚠ optional/degraded (unless AI service is running) |
| Credential provider available | ✓ complete |
| Dashboard export visible | varies |
| Agent endpoints available | ⚠ optional/degraded (unless AI service is running) |
| Patch governance enabled | ✓ complete |
| Code intelligence enabled | ✓ complete |
| Git provider resolved | ✓ complete |

AI service checks show **optional/degraded**, not failure — this is correct.

### 4. Index Roadmap

Navigate to the roadmap view. The task list should populate from `roadmap/_index.md`.

Expected: Tasks displayed in Table, Gantt, and Kanban views.

### 5. Show Dashboard

Navigate to the dashboard view or visit [orqestra.pages.dev](https://orqestra.pages.dev).

Expected: Roadmap data rendered as a public dashboard.

### 6. Run Git Diagnostics

Open the Git diagnostics panel.

Expected: Per-operation provider table showing gix/gix-hybrid/CLI fallback for each operation.

### 7. Run Bugfix Agent

*(Requires AI service running)*

Select a task with a bugfix label. Click **"Run bugfix agent"**.

Expected:
- Agent analyzes the task with symbol-aware context
- Returns a patch proposal with `before`/`after` content
- Proposal displayed in bugfix panel

### 8. Review Patch

Examine the proposed patch:
- Shows file path, before content, after content
- Shows confidence score
- Shows checksums for verification

### 9. Apply Patch

Click **"Apply"**.

Expected:
- File written atomically (temp-then-rename)
- Audit entry recorded in `.Orqestra/audit/patch_audit.jsonl`
- No auto-commit — working tree is dirty but not staged

### 10. Show Audit Record

Open `.Orqestra/audit/patch_audit.jsonl`.

Expected: JSONL entry with `proposal_id`, `agent_type`, `outcome: "applied"`, timestamp, and file path.

### 11. Run Architect Planner

*(Requires AI service running)*

Select a task with an architecture label. Click **"Generate Plan"**.

Expected:
- Architect analyzes task with context + symbols + ADRs
- Returns structured plan with:
  - Summary and context analysis
  - Proposed approach (ordered steps)
  - Affected symbols table
  - Risk assessment (high/medium/low)
  - Dependency warnings
  - Acceptance criteria
  - Test strategy
  - Task breakdown
  - Optional ADR draft

### 12. Verify Read-Only Plan

Examine the architect plan panel:

Expected:
- **No "Apply" button** — display only
- **No "Reject" button** — not a patch
- Plan shows "proposal — not implementation" badge
- Message: "Architect output is a proposal. No files were modified."

### 13. Verify No Auto-Commit

Check `git status`:

```bash
git status
```

Expected: Working tree may have the bugfix patch applied (from step 9), but **no commit was made by any agent**. All commits require human initiation.

### 14. Export Beta Diagnostics

Click **"Export Diagnostics"**.

Expected: Bundle created at `.Orqestra/orqestra-diagnostics-{timestamp}/` with 13 files:
- app.json, readiness.json, project-validation.json
- git-provider.json, credential-status.json, agent-matrix.json
- patch-governance.json, code-intel.json, roadmap-status.json
- recent-errors.json, system.txt, ai-health.json, dashboard-status.json
- README.txt (bundle overview)

Verify:
- No secrets in any file
- No source code bodies
- No raw diffs
- No `.env` content

### 15. Review Bundle Redaction

Open `README.txt` in the bundle.

Expected: Shows redaction rules applied and redacted value count > 0.

---

## Without AI Service

If the AI service is not running:

- Steps 7–12 are skipped
- Environment checks show AI service as "optional/degraded"
- All other features (roadmap, dashboard, Git diagnostics, code intel) work normally
- Diagnostics bundle still exports correctly

## Success Criteria

- [ ] App installed and launched
- [ ] Environment checks show pass/fail/optional/degraded
- [ ] Roadmap indexed and displayed
- [ ] Dashboard visible
- [ ] Git diagnostics show provider report
- [ ] Bugfix agent produces proposal (if AI service running)
- [ ] Patch applied through governance (if AI service running)
- [ ] Audit record exists
- [ ] Architect produces read-only plan (if AI service running)
- [ ] No auto-commit occurred
- [ ] Diagnostics bundle exported with 13 redacted files
- [ ] No secrets in bundle
