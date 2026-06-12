"""
Intent rationale utilities for the AI backfill pipeline.
Phase 1: stub — rationale summaries are stored by git-bridge as flat files.
Phase 2: will add embedding + triple store integration.

Note: This module stores short rationale summaries, NOT chain-of-thought.
The field is named 'rationale' to be explicit about what it contains.
"""

from .models import DiffRequest, SemanticIntent


async def store_rationale(
    trace_id: str,
    rationale: str,
    intent: SemanticIntent,
    project_root: str,
) -> str:
    """Store a rationale summary alongside the semantic stub.

    Returns the path where the rationale was written.
    Phase 1: writes to .Orqestra/graph/rationale/{trace_id}.txt
    """
    import os

    rationale_dir = os.path.join(
        project_root, ".Orqestra", "graph", "rationale"
    )
    os.makedirs(rationale_dir, exist_ok=True)

    path = os.path.join(rationale_dir, f"{trace_id}.txt")
    with open(path, "w", encoding="utf-8") as f:
        f.write(f"# Rationale: {trace_id}\n")
        f.write(f"# Intent: {intent.intent_summary}\n")
        f.write(f"# Confidence: {intent.confidence}\n\n")
        f.write(rationale)

    return path
