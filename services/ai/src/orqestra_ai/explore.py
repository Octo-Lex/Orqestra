from .models import ExploreRequest, ExplorationResult


async def explore(request: ExploreRequest) -> ExplorationResult:
    # Phase 1 stub — ML-Master loop implemented in Phase 3
    return ExplorationResult(
        plan="[stub] exploration not yet implemented",
        adr_draft="",
        affected_files=[],
        confidence=0.0,
        reasoning_trace="stub",
    )
