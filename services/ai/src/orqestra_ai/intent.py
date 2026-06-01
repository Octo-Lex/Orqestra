import anthropic
import json
import os
from .models import DiffRequest, SemanticIntent

SYSTEM_PROMPT = """You are a code analysis assistant. Given a git diff and commit message,
extract structured semantic information about what changed and why.

Respond with valid JSON matching this schema:
{
  "intent_summary": "one sentence describing what this commit does and why",
  "affected_concepts": ["list", "of", "domain", "concepts"],
  "affected_apis": ["POST /api/endpoint", "list of affected API routes if any"],
  "risk_assessment": {
    "breaking_change": true/false,
    "migration_required": "description or null",
    "rollback_complexity": "low/medium/high"
  },
  "confidence": 0.0-1.0,
  "reasoning_trace": "your chain of thought"
}"""

# Initialize Anthropic SDK pointing to the Z.ai proxy gateway
client = anthropic.AsyncAnthropic(
    api_key=os.environ.get("ZAI_API_KEY"),
    base_url="https://api.z.ai/api/anthropic",
)


async def extract_intent(request: DiffRequest) -> SemanticIntent:
    user_message = f"""Commit message: {request.commit_message_draft}

Diff:
{request.diff}

Task ID: {request.task_id or 'unknown'}
Codebase context: {request.repo_context or 'not provided'}

Extract the semantic intent as JSON."""

    message = await client.messages.create(
        model="glm-5.1",
        max_tokens=1024,
        system=SYSTEM_PROMPT,
        messages=[{"role": "user", "content": user_message}],
    )

    raw = message.content[0].text
    # Strip markdown code fences if present
    if raw.startswith("```"):
        raw = raw.split("```")[1]
        if raw.startswith("json"):
            raw = raw[4:]

    data = json.loads(raw.strip())
    return SemanticIntent(**data)
