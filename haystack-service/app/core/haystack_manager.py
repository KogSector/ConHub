import logging
import asyncio
from typing import List, Dict, Any, Optional
import hashlib
import os
from sentence_transformers import SentenceTransformer
import numpy as np
from transformers import pipeline

logger = logging.getLogger(__name__)

class HaystackManager:
    """Simplified Haystack-like document manager without heavy dependencies"""
    
    def __init__(self):
        self.documents = {}  # In-memory document store
        self.embeddings = {}  # Document embeddings
        self.encoder = None
        self.qa_pipeline = None
        self.initialized = False
    
    async def initialize(self):
        """Initialize the components"""
        try:
            logger.info("Initializing Haystack Manager...")
            
            # Initialize sentence transformer for embeddings
            self.encoder = SentenceTransformer('all-MiniLM-L6-v2')
            
            # Initialize QA pipeline
            self.qa_pipeline = pipeline(
                "question-answering",
                model="distilbert-base-cased-distilled-squad"
            )
            
            self.initialized = True
            logger.info("Haystack Manager initialized successfully")
            
        except Exception as e:
            logger.error(f"Failed to initialize Haystack Manager: {e}")
            raise
    
    def is_document_store_ready(self) -> bool:
        """Check if document store is ready"""
        return self.initialized
    
    def is_retriever_ready(self) -> bool:
        """Check if retriever is ready"""
        return self.encoder is not None
    
    def is_reader_ready(self) -> bool:
        """Check if reader is ready"""
        return self.qa_pipeline is not None
    
    async def process_document(self, filename: str, content: bytes, content_type: str) -> str:
        """Process and store a document"""
        try:
            # Generate document ID
            doc_id = hashlib.md5(f"{filename}{len(content)}".encode()).hexdigest()
            
            # Extract text based on file type
            text_content = await self._extract_text(content, filename)
            
            # Generate embeddings
            embedding = self.encoder.encode([text_content])[0]
            
            # Store document
            self.documents[doc_id] = {
                "id": doc_id,
                "filename": filename,
                "content": text_content,
                "content_type": content_type,
                "meta": {
                    "filename": filename,
                    "size": len(content)
                }
            }
            
            # Store embedding
            self.embeddings[doc_id] = embedding
            
            logger.info(f"Processed document: {filename} (ID: {doc_id})")
            return doc_id
            
        except Exception as e:
            logger.error(f"Error processing document {filename}: {e}")
            raise
    
    async def _extract_text(self, content: bytes, filename: str) -> str:
        """Extract text from different file types"""
        file_extension = os.path.splitext(filename)[1].lower()
        
        if file_extension == '.txt' or file_extension == '.md':
            return content.decode('utf-8')
        elif file_extension == '.pdf':
            try:
                from pypdf import PdfReader
                import io
                pdf_reader = PdfReader(io.BytesIO(content))
                text = ""
                for page in pdf_reader.pages:
                    text += page.extract_text()
                return text
            except Exception as e:
                logger.error(f"Error extracting PDF text: {e}")
                return "Error extracting PDF content"
        elif file_extension == '.docx':
            try:
                from docx import Document
                import io
                doc = Document(io.BytesIO(content))
                text = ""
                for paragraph in doc.paragraphs:
                    text += paragraph.text + "\n"
                return text
            except Exception as e:
                logger.error(f"Error extracting DOCX text: {e}")
                return "Error extracting DOCX content"
        else:
            return content.decode('utf-8', errors='ignore')
    
    async def search(self, query: str, top_k: int = 5, filters: Optional[Dict] = None) -> List[Dict]:
        """Search for documents using semantic similarity"""
        try:
            if not self.documents:
                return []
            
            # Generate query embedding
            query_embedding = self.encoder.encode([query])[0]
            
            # Calculate similarities
            similarities = {}
            for doc_id, doc_embedding in self.embeddings.items():
                similarity = np.dot(query_embedding, doc_embedding) / (
                    np.linalg.norm(query_embedding) * np.linalg.norm(doc_embedding)
                )
                similarities[doc_id] = similarity
            
            # Sort by similarity
            sorted_docs = sorted(similarities.items(), key=lambda x: x[1], reverse=True)
            
            # Return top-k results
            results = []
            for doc_id, score in sorted_docs[:top_k]:
                doc = self.documents[doc_id]
                results.append({
                    "id": doc_id,
                    "content": doc["content"][:500] + "..." if len(doc["content"]) > 500 else doc["content"],
                    "score": float(score),
                    "meta": doc["meta"]
                })
            
            return results
            
        except Exception as e:
            logger.error(f"Error searching documents: {e}")
            raise
    
    async def ask_question(self, question: str, top_k: int = 3, filters: Optional[Dict] = None) -> Dict:
        """Answer a question using the documents"""
        try:
            # First, find relevant documents
            relevant_docs = await self.search(question, top_k, filters)
            
            if not relevant_docs:
                return {
                    "answer": "No relevant documents found",
                    "confidence": 0.0,
                    "documents": []
                }
            
            # Use the most relevant document for QA
            context = relevant_docs[0]["content"]
            
            # Get answer from QA pipeline
            result = self.qa_pipeline(question=question, context=context)
            
            return {
                "answer": result["answer"],
                "confidence": result["score"],
                "documents": relevant_docs
            }
            
        except Exception as e:
            logger.error(f"Error answering question: {e}")
            raise
    
    async def list_documents(self) -> List[Dict]:
        """List all documents"""
        return [
            {
                "id": doc["id"],
                "filename": doc["filename"],
                "content_type": doc["content_type"],
                "meta": doc["meta"]
            }
            for doc in self.documents.values()
        ]
    
    async def delete_document(self, document_id: str) -> bool:
        """Delete a document"""
        if document_id in self.documents:
            del self.documents[document_id]
            if document_id in self.embeddings:
                del self.embeddings[document_id]
            logger.info(f"Deleted document: {document_id}")
            return True
        return False