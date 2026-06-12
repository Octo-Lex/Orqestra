"""Architect agent — read-only planning endpoint.

Produces structured plans with risk assessments, affected symbols,
acceptance criteria, test strategies, and optional ADR drafts.
Never writes files, creates patches, or mutates repository state.
"""

import json
import uuid
from datetime import datetime, timezone
from typing import Optional

from fastapi import APIRouter
from pydantic import BaseModel, Field

router = APIRouter()


# ---------------------------------------------------------------------------
# Request DTOs
# ---------------------------------------------------------------------------

class ArchitectConstraints(BaseModel):
    read_only: bool = True
    may_edit_files: bool = False
    may_create_adrs: bool = False
    max_plan_sections: int = 8


class ADRTeaser(BaseModel):
    """Bounded ADR metadata — no full body content."""
    path: str
    title: str
    status: str
    excerpt: Optional[str] = Field(None, max_length=500)


class ArchitectRequest(BaseModel):
    task: dict
    git_context: dict = {}
    git_context_status: str = "unavailable"
    symbol_summaries: list[dict] = []
    risk_summary: Optional[dict] = None
    existing_adrs: list[ADRTeaser] = []
    constraints: ArchitectConstraints = ArchitectConstraints()


# ---------------------------------------------------------------------------
# Response DTOs
# ---------------------------------------------------------------------------

class SymbolRef(BaseModel):
    name: str
    kind: str
    file: str
    is_public: bool = False


class RiskItem(BaseModel):
    risk: str
    severity: str  # low, medium, high
    mitigation: str


class TaskBreakdownItem(BaseModel):
    task: str
    scope: str
    complexity: str  # low, medium, high


class ArchitectPlan(BaseModel):
    plan_id: str
    schema_version: str = "architect-plan-v1"
    summary: str
    context_analysis: str
    proposed_approach: list[str]
    affected_symbols: list[SymbolRef]
    risk_assessment: list[RiskItem]
    dependency_warnings: list[str]
    acceptance_criteria: list[str]
    test_strategy: list[str]
    task_breakdown: list[TaskBreakdownItem]
    adr_draft: Optional[str] = None
    confidence: float
    # Structural guarantee: no patch-shaped fields
    # This DTO contains no before, after, path (as edit target), or edits fields.


class ArchitectResponse(BaseModel):
    plan: ArchitectPlan
    agent: str = "architect"
    mode: str = "read-only-planner"
    timestamp: str = ""


# ---------------------------------------------------------------------------
# Endpoint
# ---------------------------------------------------------------------------

@router.post("/agent/architect", response_model=ArchitectResponse)
async def run_architect_agent(request: ArchitectRequest) -> ArchitectResponse:
    """Architect agent — read-only planning.

    Produces a structured plan from task context, git state, symbols, and risks.
    Never writes files or creates patches.
    If the AI service is unavailable, returns an error (no fake plans).
    """
    # Build bounded prompt from context
    task_title = request.task.get("title", "Unknown task")
    task_labels = request.task.get("labels", [])

    prompt = f"""You are an architect agent producing a read-only planning analysis.

TASK: {task_title}
LABELS: {', '.join(task_labels) if task_labels else 'none'}

GIT CONTEXT STATUS: {request.git_context_status}

CHANGED FILES AND SYMBOLS:
{_format_symbols(request.symbol_summaries)}

RISK SUMMARY:
{_format_risks(request.risk_summary)}

EXISTING ADRs:
{_format_adrs(request.existing_adrs)}

CONSTRAINTS:
- read_only: {request.constraints.read_only}
- may_edit_files: {request.constraints.may_create_adrs}
- max_plan_sections: {request.constraints.max_plan_sections}

Produce a JSON plan with this exact structure:
{{
  "summary": "One-paragraph plan summary",
  "context_analysis": "Analysis of the current state",
  "proposed_approach": ["Step 1", "Step 2", ...],
  "affected_symbols": [{{"name": "...", "kind": "...", "file": "...", "is_public": false}}],
  "risk_assessment": [{{"risk": "...", "severity": "low|medium|high", "mitigation": "..."}}],
  "dependency_warnings": ["Warning 1", ...],
  "acceptance_criteria": ["Criterion 1", ...],
  "test_strategy": ["Strategy 1", ...],
  "task_breakdown": [{{"task": "...", "scope": "...", "complexity": "low|medium|high"}}],
  "adr_draft": "Markdown ADR or null",
  "confidence": 0.85
}}

Be specific. Reference actual symbol names and file paths from the context.
If insufficient context, state what is needed rather than guessing."""

    # Call AI service
    plan_data = await _call_ai_service(prompt)

    plan = ArchitectPlan(
        plan_id=f"arch-{uuid.uuid4().hex[:12]}",
        schema_version="architect-plan-v1",
        summary=plan_data.get("summary", "No summary produced"),
        context_analysis=plan_data.get("context_analysis", ""),
        proposed_approach=plan_data.get("proposed_approach", []),
        affected_symbols=[SymbolRef(**s) for s in plan_data.get("affected_symbols", [])],
        risk_assessment=[RiskItem(**r) for r in plan_data.get("risk_assessment", [])],
        dependency_warnings=plan_data.get("dependency_warnings", []),
        acceptance_criteria=plan_data.get("acceptance_criteria", []),
        test_strategy=plan_data.get("test_strategy", []),
        task_breakdown=[TaskBreakdownItem(**t) for t in plan_data.get("task_breakdown", [])],
        adr_draft=plan_data.get("adr_draft"),
        confidence=_clamp_confidence(plan_data.get("confidence", 0.5)),
    )

    return ArchitectResponse(
        plan=plan,
        timestamp=datetime.now(timezone.utc).isoformat(),
    )


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _format_symbols(symbols: list[dict]) -> str:
    if not symbols:
        return "(no symbol context available)"
    lines = []
    for s in symbols[:20]:  # Bound to 20 files
        path = s.get("path", "?")
        syms = s.get("symbols", [])
        if syms:
            sym_strs = [f"{sym.get('name', '?')} ({sym.get('kind', '?')})" for sym in syms[:10]]
            lines.append(f"  {path}: {', '.join(sym_strs)}")
        else:
            lines.append(f"  {path}: (no symbols)")
    return "\n".join(lines)


def _format_risks(risks: dict | None) -> str:
    if not risks:
        return "(no risk summary available)"
    return json.dumps(risks, indent=2)[:500]  # Bounded


def _format_adrs(adrs: list[ADRTeaser]) -> str:
    if not adrs:
        return "(no existing ADRs)"
    lines = []
    for adr in adrs[:10]:  # Bound to 10 ADRs
        line = f"  {adr.path}: {adr.title} ({adr.status})"
        if adr.excerpt:
            line += f" — {adr.excerpt[:100]}"
        lines.append(line)
    return "\n".join(lines)


def _clamp_confidence(value: float) -> float:
    return max(0.0, min(1.0, float(value)))


async def _call_ai_service(prompt: str) -> dict:
    """Call the centralized AI provider. Raises on failure — no fake plans."""
    from .provider import call_ai

    raw = await call_ai(
        prompt=prompt,
        system_prompt="You are an architecture planning assistant. Produce valid JSON only.",
        max_tokens=2000,
        temperature=0.3,
    )

    # Strip markdown code fences if present
    content = raw.strip()
    if content.startswith("```json"):
        content = content[7:]
    if content.startswith("```"):
        content = content[3:]
    if content.endswith("```"):
        content = content[:-3]
    content = content.strip()

    return json.loads(content)
