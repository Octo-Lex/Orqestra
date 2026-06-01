"""Docs-agent endpoint for Orqestra AI service.

POST /agent/docs — receives task + context files, returns proposed edits.
The docs agent only edits documentation files (README.md, docs/*, roadmap/*, CHANGELOG.md).
"""

import os
import httpx
from typing import Optional
from pydantic import BaseModel


class DocsContextFile(BaseModel):
    path: str
    content: str


class DocsConstraints(BaseModel):
    allowed_paths: list[str] = ["README.md", "docs/", "roadmap/", "CHANGELOG.md"]
    max_files_changed: int = 3
    auto_commit: bool = False


class DocsAgentRequest(BaseModel):
    task: dict
    context_files: list[DocsContextFile] = []
    constraints: Optional[DocsConstraints] = None


class DocsEdit(BaseModel):
    path: str
    before: str
    after: str


class DocsAgentResponse(BaseModel):
    summary: str
    confidence: float
    has_breaking_change: bool = False
    edits: list[DocsEdit] = []
    notes: list[str] = []


ALLOWED_PREFIXES = ["README.md", "docs/", "roadmap/", "CHANGELOG.md"]


def _is_allowed_path(path: str) -> bool:
    """Check if the path is within the docs agent's allowed scope."""
    normalized = path.replace("\\", "/")
    # Remove leading ./ or /
    while normalized.startswith("./"):
        normalized = normalized[2:]
    while normalized.startswith("/"):
        normalized = normalized[1:]

    for prefix in ALLOWED_PREFIXES:
        if normalized.startswith(prefix) or normalized == prefix.rstrip("/"):
            return True
    return False


async def docs_agent(request: DocsAgentRequest) -> DocsAgentResponse:
    """
    Execute the docs agent: analyze task, propose documentation edits.

    Uses the Z.ai gateway to generate edits, constrained to allowed paths.
    """
    constraints = request.constraints or DocsConstraints()

    # Filter context files to allowed paths
    allowed_files = [
        f for f in request.context_files
        if _is_allowed_path(f.path)
    ]

    task_title = request.task.get("title", "Unknown task")
    task_body = request.task.get("body", "")
    task_labels = request.task.get("labels", [])

    # Build prompt for the AI
    file_contents = ""
    for f in allowed_files[:constraints.max_files_changed]:
        file_contents += f"\n--- {f.path} ---\n{f.content}\n"

    prompt = f"""You are the Orqestra documentation agent. Your job is to propose edits to documentation files based on the task description.

TASK: {task_title}
LABELS: {', '.join(task_labels) if task_labels else 'none'}
BODY: {task_body}

CURRENT FILE CONTENTS:
{file_contents if file_contents else "No context files provided."}

INSTRUCTIONS:
1. Propose specific edits to the documentation files shown above.
2. Each edit must include the exact "before" text and the proposed "after" text.
3. Only edit files within these paths: {', '.join(constraints.allowed_paths)}
4. Keep changes minimal and focused on the task.
5. If no documentation changes are needed, return an empty edits list.

RESPOND IN THIS FORMAT:
SUMMARY: <one line summary>
CONFIDENCE: <0.0 to 1.0>
NOTES: <any additional notes>
EDITS:
--- FILE: path/to/file ---
BEFORE:
<exact text to replace>
AFTER:
<replacement text>
--- END EDIT ---"""

    # Call Z.ai gateway
    api_key = os.environ.get("ZAI_API_KEY", "")
    base_url = os.environ.get("ZAI_BASE_URL", "https://api.z.ai/api/anthropic")

    if not api_key:
        # Fallback: return a mock response with instructions
        return DocsAgentResponse(
            summary=f"Docs agent would process task '{task_title}' — no API key configured",
            confidence=0.5,
            has_breaking_change=False,
            edits=[],
            notes=["ZAI_API_KEY not set — returning mock response"],
        )

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                base_url.rstrip("/") + "/messages",
                headers={
                    "x-api-key": api_key,
                    "content-type": "application/json",
                    "anthropic-version": "2023-06-01",
                },
                json={
                    "model": "claude-sonnet-4-20250514",
                    "max_tokens": 2000,
                    "messages": [{"role": "user", "content": prompt}],
                },
            )
            response.raise_for_status()
            result = response.json()

        # Extract text from response
        text = ""
        for block in result.get("content", []):
            if block.get("type") == "text":
                text += block.get("text", "")

        # Parse the response
        return _parse_agent_response(text, task_title)

    except Exception as e:
        return DocsAgentResponse(
            summary=f"Docs agent failed: {str(e)[:100]}",
            confidence=0.0,
            has_breaking_change=False,
            edits=[],
            notes=[f"Error: {str(e)[:200]}"],
        )


def _parse_agent_response(text: str, task_title: str) -> DocsAgentResponse:
    """Parse the AI response into structured edits."""
    summary = task_title
    confidence = 0.7
    notes = []
    edits = []

    lines = text.split("\n")
    i = 0

    while i < len(lines):
        line = lines[i].strip()

        if line.startswith("SUMMARY:"):
            summary = line[len("SUMMARY:"):].strip()
        elif line.startswith("CONFIDENCE:"):
            try:
                confidence = float(line[len("CONFIDENCE:"):].strip())
            except ValueError:
                confidence = 0.5
        elif line.startswith("NOTES:"):
            notes.append(line[len("NOTES:"):].strip())
        elif line.startswith("--- FILE:"):
            # Parse an edit block
            file_path = line[len("--- FILE:"):].strip().rstrip(" ---")
            if not _is_allowed_path(file_path):
                notes.append(f"Skipping disallowed path: {file_path}")
                i += 1
                continue

            # Find BEFORE section
            before_lines = []
            after_lines = []
            in_before = False
            in_after = False

            i += 1
            while i < len(lines):
                l = lines[i]
                if l.strip() == "BEFORE:":
                    in_before = True
                    in_after = False
                elif l.strip() == "AFTER:":
                    in_before = False
                    in_after = True
                elif l.strip() == "--- END EDIT ---":
                    break
                elif in_before:
                    before_lines.append(l)
                elif in_after:
                    after_lines.append(l)
                i += 1

            before_text = "\n".join(before_lines)
            after_text = "\n".join(after_lines)

            if before_text and after_text:
                edits.append(DocsEdit(
                    path=file_path,
                    before=before_text,
                    after=after_text,
                ))
        i += 1

    return DocsAgentResponse(
        summary=summary,
        confidence=confidence,
        has_breaking_change=False,
        edits=edits,
        notes=notes,
    )
