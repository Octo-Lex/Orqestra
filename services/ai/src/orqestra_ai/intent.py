"""
Semantic intent extraction from git diffs.

Uses the centralized AI provider. Desktop calls localhost AI service,
never a named vendor. No chain-of-thought is requested or stored.
"""

import json
import os
from .models import DiffRequest, SemanticIntent
from .provider import call_ai, ProviderUnavailableError, ProviderConfigMissingError

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
  "rationale": "one-sentence decision summary explaining your classification"
}

Important: "rationale" should be a brief summary of your classification reasoning,
NOT a full chain-of-thought. Keep it to one sentence."""


async def extract_intent(request: DiffRequest) -> SemanticIntent:
    user_message = f"""Commit message: {request.commit_message_draft}

Diff:
{request.diff}

Task ID: {request.task_id or 'unknown'}
Codebase context: {request.repo_context or 'not provided'}

Extract the semantic intent as JSON."""

    raw = await call_ai(
        prompt=user_message,
        system_prompt=SYSTEM_PROMPT,
        max_tokens=1024,
        temperature=0.2,
    )

    # Strip markdown code fences if present
    if raw.startswith("```"):
        raw = raw.split("```")[1]
        if raw.startswith("json"):
            raw = raw[4:]

    data = json.loads(raw.strip())
    return SemanticIntent(**data)
