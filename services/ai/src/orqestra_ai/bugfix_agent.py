"""
Bugfix agent — user-scoped, human-reviewed AI execution path.

POST /agent/bugfix
  - Receives task + user-selected files
  - Only proposes edits to allowed files
  - Can request more files but cannot read them
  - Returns reviewable diffs, never auto-commits
"""

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import Optional
from .models import TaskContext
import os
import httpx

router = APIRouter()


class AllowedFile(BaseModel):
    path: str
    content: str


class BugfixConstraints(BaseModel):
    allowed_paths: list[str] = []
    max_files_changed: int = 2
    auto_commit: bool = False
    may_request_more_files: bool = True
    test_command: Optional[str] = None


class BugfixRequest(BaseModel):
    task: TaskContext
    allowed_files: list[AllowedFile]
    constraints: BugfixConstraints


class BugfixEdit(BaseModel):
    path: str
    before: str
    after: str


class RequestedFile(BaseModel):
    path: str
    reason: str


class BugfixResponse(BaseModel):
    summary: str
    confidence: float
    has_breaking_change: bool = False
    edits: list[BugfixEdit] = []
    test_plan: Optional[dict] = None
    needs_more_files: bool = False
    requested_files: list[RequestedFile] = []
    notes: list[str] = []


@router.post("/agent/bugfix", response_model=BugfixResponse)
async def run_bugfix_agent(request: BugfixRequest) -> BugfixResponse:
    """Run the bugfix agent over user-selected files only."""

    allowed_paths = request.constraints.allowed_paths
    files = request.allowed_files

    if not files:
        return BugfixResponse(
            summary="No files selected for bugfix analysis.",
            confidence=0.0,
            notes=["Select at least one file to run the bugfix agent."],
        )

    # Build the prompt for the AI
    file_sections = []
    for f in files:
        if f.path not in allowed_paths:
            return BugfixResponse(
                summary=f"File {f.path} is not in the allowed paths.",
                confidence=0.0,
                notes=[f"Allowed paths: {', '.join(allowed_paths)}"],
            )
        file_sections.append(f"### File: {f.path}\n```\n{f.content}\n```")

    file_block = "\n\n".join(file_sections)

    task_title = getattr(request.task, 'title', 'Unknown task')
    task_body = getattr(request.task, 'body', '') or ''
    task_id = getattr(request.task, 'id', 'UNKNOWN')

    prompt = f"""You are a bugfix agent. Analyze the following bug task and propose fixes.

## Task: {task_id} — {task_title}
{task_body}

## Files you may inspect and edit:
{file_block}

## Constraints:
- You may ONLY edit the files listed above.
- You may NOT read or reference files not listed above.
- If you need additional files to safely propose a fix, set needs_more_files=true and list them in requested_files.
- Do NOT suggest changes to files not listed above.
- Return the exact before/after content for each edit.
- Assess your confidence honestly (0.0 to 1.0).
- Mark has_breaking_change=true if the fix could break existing behavior.

## Response format:
Provide a summary, confidence score, and any edits as before/after pairs.
If you cannot safely fix the bug with only the provided files, explain why and request additional files.
"""

    # Call the AI service (Z.ai gateway)
    ai_base = os.getenv("ZAI_BASE_URL", "https://api.z.ai/api/anthropic")
    ai_key = os.getenv("ZAI_API_KEY", "")
    ai_model = os.getenv("ZAI_MODEL", "glm-5.1")

    if not ai_key:
        # No AI key available — return a structured "needs configuration" response
        return BugfixResponse(
            summary="[AI service not configured] Bugfix agent requires ZAI_API_KEY.",
            confidence=0.0,
            notes=[
                "Set ZAI_API_KEY environment variable to enable real AI analysis.",
                f"Task: {task_id} — {task_title}",
                f"Selected files: {', '.join(f.path for f in files)}",
                "Agent is ready — only the AI backend needs configuration.",
            ],
        )

    try:
        async with httpx.AsyncClient(timeout=45.0) as client:
            response = await client.post(
                ai_base,
                headers={
                    "Authorization": f"Bearer {ai_key}",
                    "Content-Type": "application/json",
                },
                json={
                    "model": ai_model,
                    "messages": [
                        {"role": "user", "content": prompt}
                    ],
                    "max_tokens": 4096,
                    "temperature": 0.3,
                },
            )
            response.raise_for_status()
            ai_body = response.json()
            ai_text = ai_body.get("choices", [{}])[0].get("message", {}).get("content", "")

    except Exception as e:
        return BugfixResponse(
            summary=f"AI service error: {str(e)}",
            confidence=0.0,
            notes=[f"Failed to reach AI service: {str(e)}"],
        )

    # Parse the AI response into structured edits
    edits = _parse_edits(ai_text, allowed_paths)

    return BugfixResponse(
        summary=_extract_summary(ai_text),
        confidence=_estimate_confidence(ai_text),
        has_breaking_change="breaking" in ai_text.lower(),
        edits=edits,
        needs_more_files="need more file" in ai_text.lower() or "need additional file" in ai_text.lower(),
        requested_files=_extract_requested_files(ai_text),
        notes=[
            f"Analyzed {len(files)} file(s) for task {task_id}.",
            "All edits are proposals — human review required.",
            "No auto-commit: v1.0.2 policy enforces review-only mode.",
        ],
    )


def _extract_summary(text: str) -> str:
    """Extract a summary from the AI response."""
    lines = text.strip().split("\n")
    for line in lines:
        line = line.strip()
        if line and not line.startswith("#") and not line.startswith("```"):
            return line[:200]
    return "Bugfix analysis complete."


def _estimate_confidence(text: str) -> float:
    """Estimate confidence from the AI response."""
    text_lower = text.lower()
    if "confidence" in text_lower:
        # Try to extract a number
        import re
        match = re.search(r"confidence[:\s]+(0?\.\d+|1\.0|1|0)", text_lower)
        if match:
            try:
                val = float(match.group(1))
                return min(max(val, 0.0), 1.0)
            except ValueError:
                pass

    # Heuristic
    if "cannot" in text_lower or "insufficient" in text_lower:
        return 0.3
    if "fix" in text_lower and ("before" in text_lower or "after" in text_lower):
        return 0.75
    return 0.5


def _parse_edits(text: str, allowed_paths: list[str]) -> list[BugfixEdit]:
    """Parse before/after edit pairs from the AI response."""
    edits = []
    # Simple pattern: look for ```before and ```after blocks
    import re
    blocks = re.findall(
        r'(?i)(?:file|path)[:\s]+`?([^`\n]+)`?\s*.*?```(?:\w*)\n(.*?)```.*?```(?:\w*)\n(.*?)```',
        text,
        re.DOTALL,
    )
    for match in blocks:
        path = match[0].strip()
        before = match[1].strip()
        after = match[2].strip()
        if path in allowed_paths:
            edits.append(BugfixEdit(path=path, before=before, after=after))

    return edits


def _extract_requested_files(text: str) -> list[RequestedFile]:
    """Extract requested additional files from the AI response."""
    import re
    requests = []
    # Look for patterns like "need to see path/to/file" or "request: path/to/file"
    matches = re.findall(
        r'(?:need|request|would like).*(?:file|see|read|inspect)[:\s]+`?([^\s`]+\.\w+)`?',
        text,
        re.IGNORECASE,
    )
    for path in matches:
        requests.append(RequestedFile(
            path=path.strip(),
            reason=f"Agent requested access to {path.strip()} for safe fix proposal.",
        ))
    return requests
