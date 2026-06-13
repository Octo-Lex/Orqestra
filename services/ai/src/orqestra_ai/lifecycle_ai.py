"""
Lifecycle AI endpoints for Orqestra (v2.15.0).

Provides PRD draft generation and issue graph preview generation
for the Define and Plan lifecycle stages.

F4 Mitigation:
  - max_tokens: 4000 (doubled from 2000)
  - Simplified JSON schemas (fewer keys than architect agent)
  - Raw response preserved on parse failure
  - Structured error returned, never 500/panic
  - No fabricated output
"""

import json
import hashlib
import os
import re
from typing import Optional
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from .provider import (
    call_ai,
    ProviderUnavailableError,
    ProviderRequestFailedError,
    InvalidProviderResponseError,
    AIProviderError,
)

router = APIRouter(prefix="/lifecycle", tags=["lifecycle"])


# ---------------------------------------------------------------------------
# Request / Response models
# ---------------------------------------------------------------------------

class PRDRequest(BaseModel):
    feature_title: str
    problem_brief: str
    project_profile: Optional[dict] = None
    assumptions: Optional[list] = None
    constraints: Optional[str] = None


class PRDResponse(BaseModel):
    ok: bool
    prd_markdown: Optional[str] = None
    acceptance_criteria: Optional[list] = None
    non_scope: Optional[list] = None
    error: Optional[str] = None
    error_code: Optional[str] = None
    raw_response_preserved: bool = False
    confidence: float = 0.0


class IssueGraphRequest(BaseModel):
    feature_title: str
    prd_summary: str
    acceptance_criteria: Optional[list] = None


class IssueGraphResponse(BaseModel):
    ok: bool
    issues: Optional[list] = None
    error: Optional[str] = None
    error_code: Optional[str] = None
    raw_response_preserved: bool = False


# ---------------------------------------------------------------------------
# F4 Mitigation: Safe JSON extraction
# ---------------------------------------------------------------------------

def _save_raw_response(text: str, feature_id: str, stage: str) -> str:
    """Save raw AI response to local diagnostic file (redacted)."""
    diag_dir = os.path.expanduser("~/.orqestra/diagnostics")
    os.makedirs(diag_dir, exist_ok=True)
    safe_id = hashlib.sha256(feature_id.encode()).hexdigest()[:16]
    filename = f"{stage}_raw_{safe_id}.txt"
    path = os.path.join(diag_dir, filename)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text[:8000])  # Cap at 8KB
    return path


def _extract_json_from_response(text: str) -> Optional[dict]:
    """
    Try multiple strategies to extract valid JSON from AI response.
    Returns None if all strategies fail.
    """
    text = text.strip()

    # Strategy 1: Direct parse (response IS the JSON)
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        pass

    # Strategy 2: Find JSON in markdown code block
    pattern = r"```(?:json)?\s*\n(.*?)\n```"
    matches = re.findall(pattern, text, re.DOTALL)
    for match in matches:
        try:
            return json.loads(match.strip())
        except json.JSONDecodeError:
            continue

    # Strategy 3: Find first { ... last } (greedy)
    first_brace = text.find("{")
    last_brace = text.rfind("}")
    if first_brace != -1 and last_brace != -1 and last_brace > first_brace:
        candidate = text[first_brace : last_brace + 1]
        try:
            return json.loads(candidate)
        except json.JSONDecodeError:
            pass

    # Strategy 4: Try adding closing braces if truncated (F4 workaround)
    if first_brace != -1:
        candidate = text[first_brace:]
        # Count unmatched braces
        open_braces = candidate.count("{")
        close_braces = candidate.count("}")
        missing = open_braces - close_braces
        if missing > 0:
            # Try closing the JSON
            patched = candidate + ("}" * missing)
            try:
                return json.loads(patched)
            except json.JSONDecodeError:
                pass

    return None


# ---------------------------------------------------------------------------
# Endpoints
# ---------------------------------------------------------------------------

@router.post("/define/prd", response_model=PRDResponse)
async def generate_prd_draft(req: PRDRequest):
    """Generate a PRD draft from feature intake."""
    system = (
        "You are a product manager assistant. Generate a PRD draft "
        "based on the feature intake. Output valid JSON only.\n\n"
        "JSON schema:\n"
        '{\n'
        '  "prd_markdown": "## Overview\\n...\\n## Requirements\\n...",\n'
        '  "acceptance_criteria": ["criterion 1", "criterion 2"],\n'
        '  "non_scope": ["thing 1", "thing 2"],\n'
        '  "confidence": 0.8\n'
        "}\n"
    )

    user_parts = [f"Feature: {req.feature_title}", f"\nProblem:\n{req.problem_brief}"]
    if req.project_profile:
        langs = ", ".join(l.get("name", "") for l in req.project_profile.get("languages", [])[:5])
        user_parts.append(f"\nTech stack: {langs}")
    if req.constraints:
        user_parts.append(f"\nConstraints: {req.constraints}")

    prompt = "\n".join(user_parts)

    try:
        # F4 mitigation: max_tokens=4000 (doubled from 2000)
        raw = await call_ai(
            prompt=prompt,
            system_prompt=system,
            max_tokens=4000,
            temperature=0.3,
        )
    except ProviderUnavailableError as e:
        return PRDResponse(ok=False, error=str(e), error_code="provider_unavailable")
    except ProviderRequestFailedError as e:
        return PRDResponse(ok=False, error=str(e), error_code="provider_request_failed")
    except InvalidProviderResponseError as e:
        return PRDResponse(ok=False, error=str(e), error_code="invalid_response")
    except AIProviderError as e:
        return PRDResponse(ok=False, error=str(e), error_code="provider_error")

    # Try to parse JSON
    parsed = _extract_json_from_response(raw)

    if parsed is None:
        # F4 mitigation: preserve raw response, return structured error
        diag_path = _save_raw_response(raw, req.feature_title, "prd")
        return PRDResponse(
            ok=False,
            error=f"AI response was not valid JSON. Raw response saved to {diag_path}",
            error_code="json_parse_failed",
            raw_response_preserved=True,
        )

    return PRDResponse(
        ok=True,
        prd_markdown=parsed.get("prd_markdown", ""),
        acceptance_criteria=parsed.get("acceptance_criteria", []),
        non_scope=parsed.get("non_scope", []),
        confidence=float(parsed.get("confidence", 0.0)),
    )


@router.post("/plan/issue-graph", response_model=IssueGraphResponse)
async def generate_issue_graph_preview(req: IssueGraphRequest):
    """Generate an issue graph preview from an approved PRD."""
    system = (
        "You are a tech lead assistant. Break down the feature into "
        "3 to 15 implementation issues. Output valid JSON only.\n\n"
        "JSON schema:\n"
        '{\n'
        '  "issues": [\n'
        '    {\n'
        '      "id": "ISSUE-001",\n'
        '      "title": "short title",\n'
        '      "description": "one sentence",\n'
        '      "depends_on": [],\n'
        '      "estimate_hours": 4\n'
        '    }\n'
        '  ]\n'
        "}\n"
        "Constraints:\n"
        "- Generate between 3 and 15 issues\n"
        "- Each issue must have a unique ID\n"
        "- depends_on references other issue IDs\n"
    )

    prompt = f"Feature: {req.feature_title}\n\nPRD Summary:\n{req.prd_summary}"

    try:
        raw = await call_ai(
            prompt=prompt,
            system_prompt=system,
            max_tokens=4000,
            temperature=0.3,
        )
    except (ProviderUnavailableError, ProviderRequestFailedError, InvalidProviderResponseError) as e:
        return IssueGraphResponse(ok=False, error=str(e), error_code=type(e).__name__)
    except AIProviderError as e:
        return IssueGraphResponse(ok=False, error=str(e), error_code="provider_error")

    parsed = _extract_json_from_response(raw)

    if parsed is None:
        diag_path = _save_raw_response(raw, req.feature_title, "issue_graph")
        return IssueGraphResponse(
            ok=False,
            error=f"AI response was not valid JSON. Raw response saved to {diag_path}",
            error_code="json_parse_failed",
            raw_response_preserved=True,
        )

    issues = parsed.get("issues", [])

    # Validate 3-15 constraint
    if len(issues) < 3:
        return IssueGraphResponse(
            ok=False,
            error=f"Issue graph must have at least 3 issues, got {len(issues)}",
            error_code="too_few_issues",
        )
    if len(issues) > 15:
        issues = issues[:15]  # Truncate, don't fail

    return IssueGraphResponse(ok=True, issues=issues)
