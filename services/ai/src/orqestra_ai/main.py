from pathlib import Path
from dotenv import load_dotenv

# Load .env from the service directory (gitignored, never committed)
_env_path = Path(__file__).resolve().parent.parent.parent / ".env"
load_dotenv(_env_path)

from fastapi import FastAPI, HTTPException
from .models import *
from .intent import extract_intent
from .embed import embed
from .explore import explore
from .docs_agent import docs_agent, DocsAgentRequest, DocsAgentResponse
from .bugfix_agent import router as bugfix_router
from .architect_agent import router as architect_router
from .lifecycle_ai import router as lifecycle_router

app = FastAPI(title="Orqestra AI Service", version="0.3.0")
app.include_router(bugfix_router)
app.include_router(architect_router)
app.include_router(lifecycle_router)


@app.get("/health")
async def health():
    return {"status": "ok", "service": "orqestra-ai"}


@app.post("/extract-intent", response_model=SemanticIntent)
async def extract_intent_route(request: DiffRequest):
    try:
        return await extract_intent(request)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/embed", response_model=EmbeddingVector)
async def embed_route(request: EmbedRequest):
    return await embed(request)


@app.post("/explore", response_model=ExplorationResult)
async def explore_route(request: ExploreRequest):
    return await explore(request)


@app.post("/agent/docs", response_model=DocsAgentResponse)
async def docs_agent_route(request: DocsAgentRequest):
    try:
        return await docs_agent(request)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


from .query_history import query_history as _query_history_impl


@app.post("/query-history", response_model=HistoryAnswer)
async def query_history_route(request: HistoryQuery):
    return await _query_history_impl(request.question, request.project_root)
