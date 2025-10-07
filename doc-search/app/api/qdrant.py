"""
Qdrant API endpoints for vector operations
"""

from fastapi import APIRouter, HTTPException, Form
from typing import Dict, Any, Optional
import json
import logging

from ..services.qdrant_service import get_qdrant_service

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/qdrant", tags=["qdrant"])

def get_embeddings(texts):
    """Simple embedding function - replace with actual implementation"""
    # Mock embeddings for now - 384 dimensions
    import random
    return [[random.random() for _ in range(384)] for _ in texts]

@router.get("/collections/info")
async def get_collections_info():
    """Get information about all Qdrant collections"""
    try:
        info = get_qdrant_service().get_collection_info()
        return {"success": True, "collections": info}
    except Exception as e:
        logger.error(f"Error getting collection info: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.post("/vectors/add/code")
async def add_code_vectors(
    content: str = Form(...),
    file_path: str = Form(""),
    language: str = Form(""),
    repository: str = Form("")
):
    """Add code vectors to Qdrant"""
    try:
        embeddings = get_embeddings([content])
        vectors = [{
            "content": content,
            "embedding": embeddings[0],
            "file_path": file_path,
            "language": language,
            "repository": repository
        }]
        
        success = get_qdrant_service().add_code_vectors(vectors)
        if success:
            return {"success": True, "message": "Code vectors added"}
        else:
            raise HTTPException(status_code=500, detail="Failed to add vectors")
    except Exception as e:
        logger.error(f"Error adding code vectors: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.post("/search")
async def search_vectors(
    query: str = Form(...),
    collection_type: str = Form("documents"),
    limit: int = Form(10)
):
    """Search vectors in specified collection"""
    try:
        query_embeddings = get_embeddings([query])
        results = get_qdrant_service().search_vectors(
            query_vector=query_embeddings[0],
            collection_type=collection_type,
            limit=limit
        )
        
        return {
            "success": True,
            "query": query,
            "results": results
        }
    except Exception as e:
        logger.error(f"Error searching vectors: {e}")
        raise HTTPException(status_code=500, detail=str(e))