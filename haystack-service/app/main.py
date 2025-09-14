"""
ConHub Haystack Service - Simplified Document Processing API
"""
from fastapi import FastAPI, HTTPException, UploadFile, File, Form
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Dict, Any, Optional
import json
import os
from pathlib import Path

from .core.document_manager import DocumentManager
from .models.schemas import (
    DocumentResponse, 
    SearchRequest, 
    SearchResponse, 
    QuestionRequest, 
    QuestionResponse,
    StatsResponse
)

# Initialize FastAPI app
app = FastAPI(
    title="ConHub Haystack Service",
    description="Document processing and search service for ConHub",
    version="1.0.0"
)

# Configure CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize document manager
doc_manager = DocumentManager()

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "service": "haystack"}

@app.post("/documents", response_model=DocumentResponse)
async def add_document(content: str = Form(...), metadata: str = Form("{}")) -> DocumentResponse:
    """Add a document to the collection"""
    try:
        metadata_dict = json.loads(metadata) if metadata else {}
        doc_id = doc_manager.add_document(content, metadata_dict)
        
        return DocumentResponse(
            id=doc_id,
            message="Document added successfully",
            success=True
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error adding document: {str(e)}")

@app.post("/documents/upload", response_model=DocumentResponse)
async def upload_document(file: UploadFile = File(...), metadata: str = Form("{}")) -> DocumentResponse:
    """Upload and process a document file"""
    try:
        # Read file content
        content = await file.read()
        
        # Simple text extraction (you could add more sophisticated processing here)
        if file.filename.endswith('.txt'):
            text_content = content.decode('utf-8')
        elif file.filename.endswith('.pdf'):
            # For PDF files, you'd need a PDF processing library
            # For now, just return an error message
            raise HTTPException(status_code=400, detail="PDF processing not implemented yet")
        else:
            # Try to decode as text
            try:
                text_content = content.decode('utf-8')
            except UnicodeDecodeError:
                raise HTTPException(status_code=400, detail="Unsupported file format")
        
        metadata_dict = json.loads(metadata) if metadata else {}
        metadata_dict["filename"] = file.filename
        metadata_dict["content_type"] = file.content_type
        
        doc_id = doc_manager.add_document(text_content, metadata_dict)
        
        return DocumentResponse(
            id=doc_id,
            message=f"File '{file.filename}' uploaded and processed successfully",
            success=True
        )
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error processing file: {str(e)}")

@app.post("/search", response_model=SearchResponse)
async def search_documents(request: SearchRequest) -> SearchResponse:
    """Search documents"""
    try:
        results = doc_manager.search_documents(request.query, request.limit)
        
        return SearchResponse(
            query=request.query,
            results=results,
            total_results=len(results)
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error searching documents: {str(e)}")

@app.post("/ask", response_model=QuestionResponse)
async def ask_question(request: QuestionRequest) -> QuestionResponse:
    """Ask a question about the documents"""
    try:
        result = doc_manager.ask_question(request.query, request.top_k)
        
        return QuestionResponse(
            query=request.query,
            answer=result["answer"],
            sources=result["sources"],
            confidence=result["confidence"]
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error processing question: {str(e)}")

@app.get("/stats", response_model=StatsResponse)
async def get_stats() -> StatsResponse:
    """Get document collection statistics"""
    try:
        stats = doc_manager.get_stats()
        return StatsResponse(**stats)
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error getting stats: {str(e)}")

@app.delete("/documents/{doc_id}")
async def delete_document(doc_id: str):
    """Delete a document"""
    try:
        success = doc_manager.delete_document(doc_id)
        if success:
            return {"message": "Document deleted successfully", "success": True}
        else:
            raise HTTPException(status_code=404, detail="Document not found")
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error deleting document: {str(e)}")

@app.get("/documents/{doc_id}")
async def get_document(doc_id: str):
    """Get a document by ID"""
    try:
        document = doc_manager.get_document(doc_id)
        if document:
            return document
        else:
            raise HTTPException(status_code=404, detail="Document not found")
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error retrieving document: {str(e)}")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8001)