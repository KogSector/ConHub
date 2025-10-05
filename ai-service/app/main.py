"""
ConHub Haystack Service - Simplified Document Processing API
"""
from fastapi import FastAPI, HTTPException, UploadFile, File, Form, Request
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Dict, Any, Optional
import json
import os
import time
import logging
from pathlib import Path
import uuid
import asyncio

# Import our enhanced logging
from .utils.logging import (
    performance_monitor, 
    document_logger, 
    timed_operation,
    setup_logging
)

from .core.document_manager import DocumentManager
from .services import vector_store_service
from .models.schemas import (
    DocumentResponse, 
    SearchRequest, 
    SearchResponse, 
    QuestionRequest, 
    QuestionResponse,
    StatsResponse
)

# Setup logging first
setup_logging()
logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(
    title="ConHub Haystack Service",
    description="Document processing and search service for ConHub",
    version="1.0.0"
)

# Request logging middleware
@app.middleware("http")
async def log_requests(request: Request, call_next):
    request_id = str(uuid.uuid4())
    start_time = time.time()
    
    logger.info("Request started", extra={
        'category': 'request',
        'request_id': request_id,
        'method': request.method,
        'url': str(request.url),
        'client_ip': request.client.host if request.client else None,
        'user_agent': request.headers.get('user-agent')
    })
    
    # Add request ID to request state
    request.state.request_id = request_id
    
    try:
        response = await call_next(request)
        duration = time.time() - start_time
        
        logger.info("Request completed", extra={
            'category': 'request',
            'request_id': request_id,
            'method': request.method,
            'url': str(request.url),
            'status_code': response.status_code,
            'duration': round(duration, 3)
        })
        
        return response
    except Exception as e:
        duration = time.time() - start_time
        
        logger.error("Request failed", extra={
            'category': 'request',
            'request_id': request_id,
            'method': request.method,
            'url': str(request.url),
            'duration': round(duration, 3),
            'error': str(e),
            'exception_type': type(e).__name__
        })
        
        raise

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


# Enhanced indexing endpoints
from .services.url_indexer import url_indexing_service

@app.post("/index/repository")
async def index_repository_content(content: str = Form(...), metadata: str = Form("{}")):
    """Index repository content for semantic search"""
    try:
        metadata_dict = json.loads(metadata) if metadata else {}
        metadata_dict["source_type"] = "repository"
        
        doc_id = f"repo-{uuid.uuid4()}"
        document = await vector_store_service.add_document(doc_id, content, metadata_dict)
        
        return {
            "id": document.id, 
            "message": "Repository content indexed successfully", 
            "success": True
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error indexing repository content: {str(e)}")

@app.post("/index/urls")
async def index_urls(urls: str = Form(...), config: str = Form("{}")):
    """Index URLs by crawling and extracting content"""
    try:
        url_list = json.loads(urls) if isinstance(urls, str) else [urls]
        config_dict = json.loads(config) if config else {}
        
        # Start background indexing
        job_id = await url_indexing_service.start_url_indexing(url_list, config_dict)
        
        return {
            "job_id": job_id,
            "message": "URL indexing started",
            "success": True,
            "urls_count": len(url_list)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error starting URL indexing: {str(e)}")

@app.get("/index/urls/{job_id}/status")
async def get_url_indexing_status(job_id: str):
    """Get the status of a URL indexing job"""
    try:
        status = url_indexing_service.get_job_status(job_id)
        if not status:
            raise HTTPException(status_code=404, detail="Job not found")
            
        return status
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error getting job status: {str(e)}")

@app.get("/index/urls/{job_id}/results")
async def get_url_indexing_results(job_id: str):
    """Get the results of a completed URL indexing job"""
    try:
        results = url_indexing_service.get_job_results(job_id)
        if results is None:
            raise HTTPException(status_code=404, detail="Job not found")
            
        # Index the results in vector store
        indexed_count = 0
        for doc in results:
            try:
                metadata = {
                    "source_type": "url",
                    "url": doc["url"],
                    "title": doc["title"],
                    "description": doc.get("description"),
                    "size": doc["size"]
                }
                
                await vector_store_service.add_document(doc["id"], doc["content"], metadata)
                indexed_count += 1
                
            except Exception as e:
                logger.error(f"Failed to index document {doc['id']}: {e}")
                
        return {
            "job_id": job_id,
            "documents": results,
            "total_documents": len(results),
            "indexed_documents": indexed_count
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error getting job results: {str(e)}")

@app.get("/index/urls/jobs")
async def list_url_indexing_jobs():
    """List all URL indexing jobs"""
    try:
        jobs = url_indexing_service.list_jobs()
        return {"jobs": jobs}
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error listing jobs: {str(e)}")

# Vector store endpoints
@app.post("/vector/documents")
async def add_vector_document(content: str = Form(...), metadata: str = Form("{}")):
    """Add a document to the vector store"""
    try:
        metadata_dict = json.loads(metadata) if metadata else {}
        doc_id = f"doc-{uuid.uuid4()}"
        document = await vector_store_service.add_document(doc_id, content, metadata_dict)
        return {"id": document.id, "message": "Document added to vector store", "success": True}
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error adding document to vector store: {str(e)}")

# Document source management
from .services.document_connectors import connector_manager

@app.post("/sources/dropbox")
async def connect_dropbox(access_token: str = Form(...), folder_path: str = Form("/")):
    """Connect Dropbox as a document source"""
    try:
        connector = connector_manager.create_connector("dropbox")
        credentials = {
            "access_token": access_token,
            "folder_path": folder_path
        }
        
        success = await connector.connect(credentials)
        if success:
            # Start background sync
            asyncio.create_task(sync_connector_documents(connector))
            
            return {
                "source_id": connector.connector_id,
                "message": "Dropbox connected successfully",
                "success": True,
                "folder_path": folder_path
            }
        else:
            raise HTTPException(status_code=400, detail="Failed to connect to Dropbox")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error connecting Dropbox: {str(e)}")

@app.post("/sources/google-drive")
async def connect_google_drive(credentials: str = Form(...), folder_id: str = Form(None)):
    """Connect Google Drive as a document source"""
    try:
        connector = connector_manager.create_connector("google_drive")
        creds = {
            "credentials": credentials,
            "folder_id": folder_id
        }
        
        success = await connector.connect(creds)
        if success:
            # Start background sync
            asyncio.create_task(sync_connector_documents(connector))
            
            return {
                "source_id": connector.connector_id,
                "message": "Google Drive connected successfully",
                "success": True,
                "folder_id": folder_id
            }
        else:
            raise HTTPException(status_code=400, detail="Failed to connect to Google Drive")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error connecting Google Drive: {str(e)}")

@app.post("/sources/onedrive")
async def connect_onedrive(access_token: str = Form(...), folder_path: str = Form("/")):
    """Connect Microsoft OneDrive as a document source"""
    try:
        connector = connector_manager.create_connector("onedrive")
        credentials = {
            "access_token": access_token,
            "folder_path": folder_path
        }
        
        success = await connector.connect(credentials)
        if success:
            # Start background sync
            asyncio.create_task(sync_connector_documents(connector))
            
            return {
                "source_id": connector.connector_id,
                "message": "OneDrive connected successfully",
                "success": True,
                "folder_path": folder_path
            }
        else:
            raise HTTPException(status_code=400, detail="Failed to connect to OneDrive")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error connecting OneDrive: {str(e)}")

@app.post("/sources/local-files")
async def upload_local_files(files: List[UploadFile] = File(...)):
    """Upload and index local files"""
    try:
        # Create or get local file connector
        connector = connector_manager.create_connector("local_files")
        await connector.connect({"upload_path": "./uploads"})
        
        indexed_files = []
        
        for file in files:
            content = await file.read()
            
            # Save file using connector
            file_info = await connector.save_uploaded_file(content, file.filename)
            
            # Process different file types
            if file.filename.endswith(('.txt', '.md', '.py', '.js', '.ts', '.rs', '.go', '.java')):
                text_content = content.decode('utf-8')
                
                metadata = {
                    "filename": file.filename,
                    "content_type": file.content_type,
                    "source_type": "local_file",
                    "file_size": len(content),
                    "file_path": file_info["path"]
                }
                
                doc_id = f"local-{uuid.uuid4()}"
                document = await vector_store_service.add_document(doc_id, text_content, metadata)
                indexed_files.append({
                    "id": document.id,
                    "filename": file.filename,
                    "size": len(content),
                    "path": file_info["path"]
                })
        
        return {
            "message": f"Successfully indexed {len(indexed_files)} files",
            "success": True,
            "files": indexed_files
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error uploading local files: {str(e)}")

@app.post("/vector/search")
async def vector_search(query: str = Form(...), k: int = Form(5)):
    """Perform vector similarity search"""
    try:
        results = await vector_store_service.similarity_search(query, k)
        return {
            "query": query,
            "results": [{
                "content": result.document.content,
                "metadata": result.document.metadata,
                "score": result.score
            } for result in results],
            "total_results": len(results)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error performing vector search: {str(e)}")

# Context aggregation endpoint
@app.post("/context/aggregate")
async def aggregate_context(query: str = Form(...), sources: str = Form("all")):
    """Aggregate context from multiple sources for AI agents"""
    try:
        source_list = sources.split(",") if sources != "all" else ["documents", "repositories", "urls"]
        
        aggregated_context = {
            "query": query,
            "sources": {},
            "total_results": 0
        }
        
        # Search documents
        if "documents" in source_list:
            doc_results = await vector_store_service.similarity_search(query, 5)
            aggregated_context["sources"]["documents"] = [{
                "content": result.content,
                "metadata": result.metadata,
                "score": getattr(result, 'score', 0.8)
            } for result in doc_results]
            aggregated_context["total_results"] += len(doc_results)
        
        return aggregated_context
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error aggregating context: {str(e)}")


# Background sync function
async def sync_connector_documents(connector):
    """Background task to sync documents from a connector"""
    try:
        documents = await connector.sync_documents()
        
        for doc in documents:
            try:
                metadata = {
                    "source_type": doc["source"],
                    "filename": doc["name"],
                    "file_path": doc["path"],
                    "file_size": doc["size"],
                    "modified": doc["modified"]
                }
                
                doc_id = doc["id"]
                await vector_store_service.add_document(doc_id, doc["content"], metadata)
                logger.info(f"Successfully indexed document: {doc['name']}", extra={'category': 'indexing'})
                
            except Exception as e:
                logger.error(f"Failed to index document {doc['name']}: {e}", extra={'category': 'indexing'})
                
    except Exception as e:
        logger.error(f"Failed to sync documents for connector {connector.connector_id}: {e}", extra={'category': 'indexing'})

# Document source management endpoints
@app.get("/sources")
async def list_sources():
    """List all connected document sources"""
    try:
        connectors = connector_manager.list_connectors()
        return {
            "sources": connectors,
            "total": len(connectors)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error listing sources: {str(e)}")

@app.delete("/sources/{source_id}")
async def disconnect_source(source_id: str):
    """Disconnect a document source"""
    try:
        success = connector_manager.remove_connector(source_id)
        if success:
            # TODO: Remove indexed documents from vector store
            return {
                "message": "Source disconnected successfully",
                "success": True
            }
        else:
            raise HTTPException(status_code=404, detail="Source not found")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error disconnecting source: {str(e)}")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8001)
