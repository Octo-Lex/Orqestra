from sentence_transformers import SentenceTransformer
from .models import EmbedRequest, EmbeddingVector

_model = None


def get_model() -> SentenceTransformer:
    global _model
    if _model is None:
        _model = SentenceTransformer("all-MiniLM-L6-v2")
    return _model


async def embed(request: EmbedRequest) -> EmbeddingVector:
    model = get_model()
    vector = model.encode(request.text).tolist()
    return EmbeddingVector(
        vector=vector,
        model="all-MiniLM-L6-v2",
        dimensions=len(vector),
    )
