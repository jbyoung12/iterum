from __future__ import annotations

from fastapi import FastAPI

from app.config import get_settings
from app.dependencies import create_retriever, create_store
from app.routes.context import router as context_router
from app.routes.debug import router as debug_router


app = FastAPI(title="Iterum", version="0.2.0")
settings = get_settings()
app.state.store = create_store(settings)
app.state.retriever = create_retriever(app.state.store, settings)
app.include_router(context_router)
app.include_router(debug_router)
