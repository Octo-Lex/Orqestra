"""
v2.14.7: AI provider configuration truth tests.

Tests verify:
- Provider config is centralized and provider-agnostic
- No reasoning_trace / chain-of-thought fields in schemas
- rationale fields are short summaries, not CoT
- Error states are structured and consistent
- Agent responses use provider-independent result shapes
"""

import pytest
import json
from orqestra_ai.models import SemanticIntent, ExplorationResult
from orqestra_ai.provider import (
    get_config,
    AIProviderError,
    ProviderUnavailableError,
    ProviderConfigMissingError,
    ProviderRequestFailedError,
    InvalidProviderResponseError,
    AgentDisabledError,
)


# ---------------------------------------------------------------------------
# Schema: no chain-of-thought fields
# ---------------------------------------------------------------------------

class TestNoChainOfThoughtFields:
    """Prove that no model schema contains reasoning_trace or chain-of-thought fields."""

    def test_semantic_intent_has_rationale_not_reasoning_trace(self):
        fields = SemanticIntent.model_fields
        assert "rationale" in fields, "SemanticIntent must have 'rationale' field"
        assert "reasoning_trace" not in fields, "SemanticIntent must NOT have 'reasoning_trace'"
        assert "chain_of_thought" not in fields, "SemanticIntent must NOT have 'chain_of_thought'"

    def test_exploration_result_has_rationale_not_reasoning_trace(self):
        fields = ExplorationResult.model_fields
        assert "rationale" in fields, "ExplorationResult must have 'rationale' field"
        assert "reasoning_trace" not in fields, "ExplorationResult must NOT have 'reasoning_trace'"

    def test_semantic_intent_rationale_is_string(self):
        intent = SemanticIntent(
            intent_summary="test",
            affected_concepts=["test"],
            affected_apis=[],
            risk_assessment={"breaking_change": False},
            confidence=0.8,
            rationale="Brief classification summary",
        )
        assert isinstance(intent.rationale, str)
        assert len(intent.rationale) < 500  # rationale is short, not CoT

    def test_exploration_result_rationale_is_string(self):
        result = ExplorationResult(
            plan="test plan",
            adr_draft="",
            affected_files=[],
            confidence=0.5,
            rationale="Brief exploration summary",
        )
        assert isinstance(result.rationale, str)

    def test_semantic_intent_serializes_without_cot(self):
        intent = SemanticIntent(
            intent_summary="Added feature X",
            affected_concepts=["feature-x"],
            affected_apis=["GET /api/x"],
            risk_assessment={"breaking_change": False},
            confidence=0.9,
            rationale="New endpoint addition, non-breaking",
        )
        json_str = intent.model_dump_json()
        data = json.loads(json_str)
        assert "rationale" in data
        assert "reasoning_trace" not in data
        assert "chain_of_thought" not in data
        assert "thinking" not in data


# ---------------------------------------------------------------------------
# Provider configuration
# ---------------------------------------------------------------------------

class TestProviderConfiguration:
    """Prove centralized provider config exists and is provider-agnostic."""

    def test_get_config_returns_dict(self):
        config = get_config()
        assert isinstance(config, dict)
        assert "base_url" in config
        assert "model" in config
        assert "has_api_key" in config
        assert "provider" in config

    def test_get_config_is_provider_agnostic(self):
        config = get_config()
        # Desktop calls localhost — no vendor name in base_url default
        assert "localhost" in config["base_url"] or config["provider"] == "remote"

    def test_get_config_no_vendor_names_in_keys(self):
        config = get_config()
        json_str = json.dumps(config).lower()
        # No vendor-specific env var names
        assert "zukijourney" not in json_str
        assert "z.ai" not in json_str


# ---------------------------------------------------------------------------
# Error states
# ---------------------------------------------------------------------------

class TestErrorStates:
    """Prove all error codes are structured and consistent."""

    def test_provider_unavailable_error(self):
        err = ProviderUnavailableError("test detail")
        assert err.code == "provider_unavailable"
        assert "test detail" in err.message

    def test_provider_config_missing_error(self):
        err = ProviderConfigMissingError()
        assert err.code == "provider_config_missing"

    def test_provider_request_failed_error(self):
        err = ProviderRequestFailedError("timeout")
        assert err.code == "provider_request_failed"
        assert "timeout" in err.message

    def test_invalid_provider_response_error(self):
        err = InvalidProviderResponseError("bad json")
        assert err.code == "invalid_provider_response"

    def test_agent_disabled_error(self):
        err = AgentDisabledError()
        assert err.code == "agent_disabled"

    def test_all_errors_inherit_ai_provider_error(self):
        errors = [
            ProviderUnavailableError(),
            ProviderConfigMissingError(),
            ProviderRequestFailedError(),
            InvalidProviderResponseError(),
            AgentDisabledError(),
        ]
        for err in errors:
            assert isinstance(err, AIProviderError)
            assert hasattr(err, "code")
            assert hasattr(err, "message")

    def test_five_error_codes_are_distinct(self):
        codes = {
            ProviderUnavailableError().code,
            ProviderConfigMissingError().code,
            ProviderRequestFailedError().code,
            InvalidProviderResponseError().code,
            AgentDisabledError().code,
        }
        assert len(codes) == 5


# ---------------------------------------------------------------------------
# Agent result shapes (provider-independent)
# ---------------------------------------------------------------------------

class TestAgentResultShapes:
    """Prove agent responses use consistent provider-independent shapes."""

    def test_docs_agent_response_has_no_provider_field(self):
        from orqestra_ai.docs_agent import DocsAgentResponse
        fields = DocsAgentResponse.model_fields
        assert "provider" not in fields
        assert "model" not in fields
        assert "raw_response" not in fields
        assert "reasoning_trace" not in fields

    def test_bugfix_agent_response_has_no_provider_field(self):
        from orqestra_ai.bugfix_agent import BugfixResponse
        fields = BugfixResponse.model_fields
        assert "provider" not in fields
        assert "model" not in fields
        assert "reasoning_trace" not in fields

    def test_architect_response_has_no_provider_field(self):
        from orqestra_ai.architect_agent import ArchitectResponse
        fields = ArchitectResponse.model_fields
        assert "provider" not in fields
        assert "reasoning_trace" not in fields
        # Plan has no chain-of-thought either
        plan_fields = ArchitectResponse.model_fields["plan"]
        assert plan_fields is not None


# ---------------------------------------------------------------------------
# Architect agent: routes through centralized provider
# ---------------------------------------------------------------------------

class TestArchitectProviderRouting:
    """Prove architect agent uses call_ai() without direct env/vendor branching."""

    def test_architect_no_direct_os_import_in_endpoint(self):
        """The run_architect_agent function should not import os or read env directly."""
        import inspect
        from orqestra_ai.architect_agent import run_architect_agent

        source = inspect.getsource(run_architect_agent)
        assert "os.environ" not in source, (
            "run_architect_agent must not read os.environ directly"
        )
        assert "ZAI_API_KEY" not in source, (
            "run_architect_agent must not reference vendor-specific env vars"
        )
        assert "ORQESTRA_AI_API_KEY" not in source, (
            "run_architect_agent must not gate on API key — call_ai() handles that"
        )

    def test_architect_call_ai_service_has_no_api_key_param(self):
        """_call_ai_service should not take an api_key parameter."""
        import inspect
        from orqestra_ai.architect_agent import _call_ai_service

        sig = inspect.signature(_call_ai_service)
        params = list(sig.parameters.keys())
        assert "api_key" not in params, (
            f"_call_ai_service should not take api_key param. Got: {params}"
        )
        assert "prompt" in params, (
            f"_call_ai_service should take prompt param. Got: {params}"
        )

    def test_architect_uses_call_ai(self):
        """_call_ai_service should import and use call_ai from provider."""
        import inspect
        from orqestra_ai.architect_agent import _call_ai_service

        source = inspect.getsource(_call_ai_service)
        assert "call_ai" in source, (
            "_call_ai_service must use call_ai from provider module"
        )

    def test_docs_agent_no_vendor_references(self):
        """docs_agent.py should have no vendor-specific references."""
        import inspect
        from orqestra_ai.docs_agent import docs_agent

        source = inspect.getsource(docs_agent)
        assert "Z.ai" not in source
        assert "zukijourney" not in source
        assert "ZAI_API_KEY" not in source
        assert "ZAI_BASE_URL" not in source

    def test_bugfix_agent_no_vendor_references(self):
        """bugfix_agent.py should have no vendor-specific references."""
        import inspect
        from orqestra_ai.bugfix_agent import run_bugfix_agent

        source = inspect.getsource(run_bugfix_agent)
        assert "Z.ai" not in source
        assert "zukijourney" not in source
        assert "ZAI_API_KEY" not in source
        assert "ZAI_BASE_URL" not in source
