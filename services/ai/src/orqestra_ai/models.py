from pydantic import BaseModel
from typing import Optional


class DiffRequest(BaseModel):
    diff: str  # raw git diff output
    commit_message_draft: str  # conventional commit message
    task_id: Optional[str] = None  # linked task ID if known
    repo_context: Optional[str] = None  # brief description of the codebase


class SemanticIntent(BaseModel):
    intent_summary: str
    affected_concepts: list[str]
    affected_apis: list[str]
    risk_assessment: dict  # breaking_change, migration_required, rollback_complexity
    confidence: float  # 0.0–1.0
    reasoning_trace: str  # the model's chain of thought


class EmbedRequest(BaseModel):
    text: str


class EmbeddingVector(BaseModel):
    vector: list[float]
    model: str
    dimensions: int


class ExploreRequest(BaseModel):
    task_description: str
    codebase_snapshot: str  # relevant file contents
    task_id: Optional[str] = None


class ExplorationResult(BaseModel):
    plan: str
    adr_draft: str
    affected_files: list[str]
    confidence: float
    reasoning_trace: str


class HistoryQuery(BaseModel):
    question: str
    project_root: str


class HistoryAnswer(BaseModel):
    answer: str
    supporting_commits: list[str]
