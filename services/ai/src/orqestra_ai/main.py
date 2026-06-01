from fastapi import FastAPI, HTTPException
from .models import *
from .intent import extract_intent
from .embed import embed
from .explore import explore

app = FastAPI(title="Orqestra AI Service", version="0.1.0")


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


@app.post("/query-history")
async def query_history_route(request: HistoryQuery):
    # Phase 4 stub
    return {"answer": "not yet implemented", "supporting_commits": []}
