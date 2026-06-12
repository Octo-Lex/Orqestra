"""
Centralized AI provider configuration for Orqestra AI service.

All agent endpoints must route through this module. Desktop is provider-agnostic:
it calls localhost AI service, never a named vendor.

Environment variables (all optional, with sensible defaults):
  ORQESTRA_AI_BASE_URL — API base URL (default: http://localhost:18321)
  ORQESTRA_AI_API_KEY  — API key (default: "" — local service may not need one)
  ORQESTRA_AI_MODEL    — Model identifier (default: local-default)
"""

import os
import httpx
from typing import Optional

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

# The authoritative provider path. Desktop calls localhost; the Python
# service routes to whatever backend is configured here.
_BASE_URL = os.environ.get("ORQESTRA_AI_BASE_URL", "http://localhost:18321")
_API_KEY = os.environ.get("ORQESTRA_AI_API_KEY", "")
_MODEL = os.environ.get("ORQESTRA_AI_MODEL", "local-default")


def get_config() -> dict:
    """Return the current AI provider configuration (for diagnostics)."""
    return {
        "base_url": _BASE_URL,
        "model": _MODEL,
        "has_api_key": bool(_API_KEY),
        "provider": "localhost" if "localhost" in _BASE_URL else "remote",
    }


# ---------------------------------------------------------------------------
# Error states
# ---------------------------------------------------------------------------

class AIProviderError(Exception):
    """Base error for AI provider failures."""
    def __init__(self, code: str, message: str):
        self.code = code
        self.message = message
        super().__init__(f"[{code}] {message}")


class ProviderUnavailableError(AIProviderError):
    def __init__(self, detail: str = "AI service is not reachable"):
        super().__init__("provider_unavailable", detail)


class ProviderConfigMissingError(AIProviderError):
    def __init__(self, detail: str = "AI provider is not configured"):
        super().__init__("provider_config_missing", detail)


class ProviderRequestFailedError(AIProviderError):
    def __init__(self, detail: str = "AI provider request failed"):
        super().__init__("provider_request_failed", detail)


class InvalidProviderResponseError(AIProviderError):
    def __init__(self, detail: str = "AI provider returned invalid response"):
        super().__init__("invalid_provider_response", detail)


class AgentDisabledError(AIProviderError):
    def __init__(self, detail: str = "Agent is disabled"):
        super().__init__("agent_disabled", detail)


# ---------------------------------------------------------------------------
# Centralized call
# ---------------------------------------------------------------------------

async def call_ai(
    prompt: str,
    system_prompt: Optional[str] = None,
    max_tokens: int = 2000,
    temperature: float = 0.3,
    model: Optional[str] = None,
) -> str:
    """
    Call the AI service through the centralized provider path.

    Returns the text response from the model.
    Raises AIProviderError on failure.
    """
    effective_model = model or _MODEL

    headers = {
        "Content-Type": "application/json",
    }
    if _API_KEY:
        headers["Authorization"] = f"Bearer {_API_KEY}"

    messages = []
    if system_prompt:
        messages.append({"role": "system", "content": system_prompt})
    messages.append({"role": "user", "content": prompt})

    payload = {
        "model": effective_model,
        "messages": messages,
        "max_tokens": max_tokens,
        "temperature": temperature,
    }

    try:
        async with httpx.AsyncClient(timeout=45.0) as client:
            url = f"{_BASE_URL.rstrip('/')}/v1/chat/completions"
            response = await client.post(url, headers=headers, json=payload)
            response.raise_for_status()
    except httpx.ConnectError as e:
        raise ProviderUnavailableError(f"Cannot reach AI service at {_BASE_URL}: {e}")
    except httpx.TimeoutException:
        raise ProviderUnavailableError(f"AI service timed out at {_BASE_URL}")
    except httpx.HTTPStatusError as e:
        raise ProviderRequestFailedError(f"AI service returned {e.response.status_code}: {e}")
    except Exception as e:
        raise ProviderRequestFailedError(f"AI request failed: {e}")

    try:
        data = response.json()
        content = data["choices"][0]["message"]["content"]
    except (KeyError, IndexError, TypeError) as e:
        raise InvalidProviderResponseError(f"Unexpected response format: {e}")

    return content
