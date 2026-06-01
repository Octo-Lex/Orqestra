# Reasoning trace utilities for the AI backfill pipeline.
# Phase 1: stub — reasoning traces are stored by git-bridge as flat files.
# Phase 2: will add embedding + triple store integration.

from .models import DiffRequest, SemanticIntent


async def store_reasoning_trace(
    trace_id: str,
    reasoning: str,
    intent: SemanticIntent,
    project_root: str,
) -> str:
    """Store a reasoning trace alongside the semantic stub.

    Returns the path where the trace was written.
    Phase 1: writes to .Orqestra/graph/reasoning/{trace_id}.txt
    """
    import os

    trace_dir = os.path.join(
        project_root, ".Orqestra", "graph", "reasoning"
    )
    os.makedirs(trace_dir, exist_ok=True)

    path = os.path.join(trace_dir, f"{trace_id}.txt")
    with open(path, "w", encoding="utf-8") as f:
        f.write(f"# Reasoning Trace: {trace_id}\n")
        f.write(f"# Intent: {intent.intent_summary}\n")
        f.write(f"# Confidence: {intent.confidence}\n\n")
        f.write(reasoning)

    return path
