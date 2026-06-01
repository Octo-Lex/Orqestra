"""Natural language query over commit history using vector search + graph traversal."""

import json
import os
from pathlib import Path
from typing import Optional

import numpy as np
from pydantic import BaseModel

from .embed import get_model
from .models import HistoryAnswer


class ScoredCommit(BaseModel):
    hash: str
    score: float
    intent: str
    concepts: list[str]
    task_ids: list[str]
    confidence: float
    trace_id: Optional[str] = None
    conventional_message: str = ""
    timestamp: str = ""


def _load_commits(project_root: str) -> list[dict]:
    """Load all commit stubs from .Orqestra/graph/commits/."""
    commits_dir = Path(project_root) / ".Orqestra" / "graph" / "commits"
    commits = []
    if not commits_dir.exists():
        return commits
    for f in sorted(commits_dir.glob("*.json")):
        try:
            data = json.loads(f.read_text(encoding="utf-8"))
            commits.append(data)
        except (json.JSONDecodeError, OSError):
            continue
    return commits


def _load_reasoning_trace(project_root: str, trace_id: str) -> str:
    """Load a reasoning trace file."""
    trace_path = Path(project_root) / ".Orqestra" / "graph" / "reasoning" / f"{trace_id}.txt"
    if trace_path.exists():
        return trace_path.read_text(encoding="utf-8")
    return ""


def _cosine_similarity(a: np.ndarray, b: np.ndarray) -> float:
    """Compute cosine similarity between two vectors."""
    norm_a = np.linalg.norm(a)
    norm_b = np.linalg.norm(b)
    if norm_a == 0 or norm_b == 0:
        return 0.0
    return float(np.dot(a, b) / (norm_a * norm_b))


async def query_history(question: str, project_root: str) -> HistoryAnswer:
    """
    Answer a natural language question about commit history.
    
    Strategy:
    1. Embed the question
    2. Embed each commit's intent_summary
    3. Rank by cosine similarity
    4. Return top matches with reasoning traces
    """
    model = get_model()

    # Embed the question
    question_vec = model.encode(question)

    # Load and score commits
    commits = _load_commits(project_root)
    if not commits:
        return HistoryAnswer(
            answer="No indexed commits found. Run `index_commits` first.",
            supporting_commits=[],
        )

    scored: list[ScoredCommit] = []
    for commit in commits:
        sem = commit.get("semantic", {})
        intent = sem.get("intent_summary", "")
        if not intent:
            continue

        intent_vec = model.encode(intent)
        score = _cosine_similarity(question_vec, intent_vec)

        scored.append(ScoredCommit(
            hash=commit["hash"],
            score=score,
            intent=intent,
            concepts=sem.get("affected_concepts", []),
            task_ids=sem.get("task_ids", []),
            confidence=sem.get("confidence", 0.0),
            trace_id=sem.get("reasoning_trace_id"),
            conventional_message=commit.get("conventional_message", ""),
            timestamp=commit.get("timestamp", ""),
        ))

    # Sort by score descending
    scored.sort(key=lambda c: c.score, reverse=True)

    # Build answer from top results
    top = scored[:5]
    if not top:
        return HistoryAnswer(
            answer="No relevant commits found.",
            supporting_commits=[],
        )

    best = top[0]
    parts = [f"Best match (score {best.score:.3f}):"]
    parts.append(f"  Commit: {best.hash}")
    parts.append(f"  Message: {best.conventional_message}")
    parts.append(f"  Intent: {best.intent}")
    parts.append(f"  Confidence: {best.confidence:.2f}")
    if best.concepts:
        parts.append(f"  Concepts: {', '.join(best.concepts)}")
    if best.task_ids:
        parts.append(f"  Tasks: {', '.join(best.task_ids)}")

    # Load reasoning trace if available
    if best.trace_id:
        trace = _load_reasoning_trace(project_root, best.trace_id)
        if trace:
            parts.append(f"  Reasoning trace:")
            for line in trace.strip().splitlines():
                parts.append(f"    {line}")

    if len(top) > 1:
        parts.append(f"\nOther relevant commits:")
        for c in top[1:]:
            parts.append(
                f"  {c.hash[:12]} (score {c.score:.3f}): {c.intent[:80]}..."
                if len(c.intent) > 80
                else f"  {c.hash[:12]} (score {c.score:.3f}): {c.intent}"
            )

    return HistoryAnswer(
        answer="\n".join(parts),
        supporting_commits=[c.hash for c in top],
    )
