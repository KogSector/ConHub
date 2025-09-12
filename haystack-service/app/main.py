import os
import logging
from typing import List, Optional, Dict, Any
from fastapi import FastAPI, HTTPException, UploadFile, File, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import uvicorn
from dotenv import load_dotenv

from .core.haystack_manager import HaystackManager
from .models.schemas import (
    DocumentRequest,
    SearchRequest,
    SearchResponse,
    IndexResponse,
    HealthResponse
)
from .core.config import get_settings

# Load environment variables
load_dotenv()

# Configure logging
logging.basicConfig(
    level=getattr(logging, os.getenv("LOG_LEVEL", "INFO")),
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

app = FastAPI(
    title="ConHub Haystack Service",
    description="Document indexing and search service using Haystack",
    version="1.0.0",
    docs_url="/docs",
    redoc_url="/redoc"
)

# CORS middleware
settings = get_settings()
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize Haystack manager
haystack_manager = None

@app.on_event("startup")
async def startup_event():
    """Initialize Haystack components on startup."""
    global haystack_manager
    try:
        logger.info("Initializing Haystack service...")
        haystack_manager = HaystackManager()
        await haystack_manager.initialize()
        logger.info("Haystack service initialized successfully")
    except Exception as e:
        logger.error(f"Failed to initialize Haystack service: {e}")
        raise

@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown."""
    global haystack_manager
    if haystack_manager:
        await haystack_manager.cleanup()

def get_haystack_manager() -> HaystackManager:
    """Dependency to get Haystack manager."""
    if haystack_manager is None:
        raise HTTPException(status_code=500, detail="Haystack service not initialized")
    return haystack_manager

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint."""
    return HealthResponse(
        status="healthy",
        service="ConHub Haystack Service",
        version="1.0.0"
    )

@app.post("/documents", response_model=IndexResponse)
async def index_documents(
    request: DocumentRequest,
    manager: HaystackManager = Depends(get_haystack_manager)
):
    """Index documents in the document store."""
    try:
        logger.info(f"Indexing {len(request.documents)} documents")
        result = await manager.index_documents(request.documents)
        return IndexResponse(
            success=True,
            message=f"Successfully indexed {len(request.documents)} documents",
            document_count=len(request.documents),
            index_id=result.get("index_id")
        )
    except Exception as e:
        logger.error(f"Error indexing documents: {e}")
        raise HTTPException(status_code=500, detail=f"Indexing failed: {str(e)}")

@app.post("/documents/upload", response_model=IndexResponse)
async def upload_and_index_file(
    file: UploadFile = File(...),
    metadata: Optional[str] = None,
    manager: HaystackManager = Depends(get_haystack_manager)
):
    """Upload and index a file."""
    try:
        logger.info(f"Processing uploaded file: {file.filename}")
        result = await manager.process_uploaded_file(file, metadata)
        return IndexResponse(
            success=True,
            message=f"Successfully processed file: {file.filename}",
            document_count=result.get("document_count", 1),
            index_id=result.get("index_id")
        )
    except Exception as e:
        logger.error(f"Error processing uploaded file: {e}")
        raise HTTPException(status_code=500, detail=f"File processing failed: {str(e)}")

@app.post("/search", response_model=SearchResponse)
async def search_documents(
    request: SearchRequest,
    manager: HaystackManager = Depends(get_haystack_manager)
):
    """Search documents using semantic search."""
    try:
        logger.info(f"Searching for: {request.query}")
        results = await manager.search_documents(
            query=request.query,
            top_k=request.top_k,
            filters=request.filters
        )
        return SearchResponse(
            success=True,
            query=request.query,
            results=results,
            total_count=len(results)
        )
    except Exception as e:
        logger.error(f"Error searching documents: {e}")
        raise HTTPException(status_code=500, detail=f"Search failed: {str(e)}")

@app.post("/ask", response_model=SearchResponse)
async def ask_question(
    request: SearchRequest,
    manager: HaystackManager = Depends(get_haystack_manager)
):
    """Ask a question and get an answer from indexed documents."""
    try:
        logger.info(f"Answering question: {request.query}")
        results = await manager.answer_question(
            question=request.query,
            top_k=request.top_k,
            filters=request.filters
        )
        return SearchResponse(
            success=True,
            query=request.query,
            results=results,
            total_count=len(results)
        )
    except Exception as e:
        logger.error(f"Error answering question: {e}")
        raise HTTPException(status_code=500, detail=f"Question answering failed: {str(e)}")

@app.get("/stats")
async def get_stats(
    manager: HaystackManager = Depends(get_haystack_manager)
):
    """Get statistics about indexed documents."""
    try:
        stats = await manager.get_document_stats()
        return {
            "success": True,
            "stats": stats
        }
    except Exception as e:
        logger.error(f"Error getting stats: {e}")
        raise HTTPException(status_code=500, detail=f"Stats retrieval failed: {str(e)}")

if __name__ == "__main__":
    settings = get_settings()
    uvicorn.run(
        "app.main:app",
        host=settings.host,
        port=settings.port,
        reload=settings.environment == "development",
        log_level=settings.log_level.lower()
    )
