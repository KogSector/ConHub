from fastapi import FastAPI, File, UploadFile, HTTPException
from fastapi.middleware.cors import CORSMiddleware
import uvicorn
import os
from dotenv import load_dotenv
from app.core.haystack_manager import HaystackManager
from app.models.schemas import SearchRequest, SearchResponse, DocumentResponse
from typing import List
import logging

# Load environment variables
load_dotenv()

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(
    title="Haystack Document Processing Service",
    description="A service for document processing, search, and Q&A using Haystack",
    version="1.0.0"
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize Haystack manager
haystack_manager = HaystackManager()

@app.on_event("startup")
async def startup_event():
    """Initialize the Haystack components on startup"""
    try:
        await haystack_manager.initialize()
        logger.info("Haystack service initialized successfully")
    except Exception as e:
        logger.error(f"Failed to initialize Haystack service: {e}")
        raise

@app.get("/")
async def root():
    """Health check endpoint"""
    return {"message": "Haystack Document Processing Service is running", "status": "healthy"}

@app.get("/health")
async def health_check():
    """Detailed health check"""
    return {
        "status": "healthy",
        "service": "haystack-document-processor",
        "version": "1.0.0",
        "components": {
            "document_store": haystack_manager.is_document_store_ready(),
            "retriever": haystack_manager.is_retriever_ready(),
            "reader": haystack_manager.is_reader_ready()
        }
    }

@app.post("/upload", response_model=DocumentResponse)
async def upload_document(file: UploadFile = File(...)):
    """Upload and process a document"""
    try:
        # Validate file type
        allowed_types = ['.pdf', '.docx', '.txt', '.md']
        file_extension = os.path.splitext(file.filename)[1].lower()
        
        if file_extension not in allowed_types:
            raise HTTPException(
                status_code=400, 
                detail=f"File type {file_extension} not supported. Allowed types: {allowed_types}"
            )
        
        # Read file content
        content = await file.read()
        
        # Process document with Haystack
        document_id = await haystack_manager.process_document(
            filename=file.filename,
            content=content,
            content_type=file.content_type
        )
        
        return DocumentResponse(
            document_id=document_id,
            filename=file.filename,
            status="processed",
            message="Document uploaded and processed successfully"
        )
        
    except Exception as e:
        logger.error(f"Error uploading document: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/search", response_model=List[SearchResponse])
async def search_documents(request: SearchRequest):
    """Search through processed documents"""
    try:
        results = await haystack_manager.search(
            query=request.query,
            top_k=request.top_k,
            filters=request.filters
        )
        
        return [
            SearchResponse(
                document_id=result.get("id", ""),
                content=result.get("content", ""),
                score=result.get("score", 0.0),
                metadata=result.get("meta", {})
            )
            for result in results
        ]
        
    except Exception as e:
        logger.error(f"Error searching documents: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/ask")
async def ask_question(request: SearchRequest):
    """Ask a question and get an answer from the documents"""
    try:
        answer = await haystack_manager.ask_question(
            question=request.query,
            top_k=request.top_k,
            filters=request.filters
        )
        
        return {
            "question": request.query,
            "answer": answer.get("answer", "No answer found"),
            "confidence": answer.get("confidence", 0.0),
            "supporting_documents": answer.get("documents", [])
        }
        
    except Exception as e:
        logger.error(f"Error answering question: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/documents")
async def list_documents():
    """List all processed documents"""
    try:
        documents = await haystack_manager.list_documents()
        return {"documents": documents}
    except Exception as e:
        logger.error(f"Error listing documents: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.delete("/documents/{document_id}")
async def delete_document(document_id: str):
    """Delete a specific document"""
    try:
        success = await haystack_manager.delete_document(document_id)
        if success:
            return {"message": f"Document {document_id} deleted successfully"}
        else:
            raise HTTPException(status_code=404, detail="Document not found")
    except Exception as e:
        logger.error(f"Error deleting document: {e}")
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    port = int(os.getenv("PORT", 8001))
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=port,
        reload=True,
        log_level="info"
    )
