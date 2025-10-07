from typing import List, Dict, Any, Optional
from pydantic import BaseModel
import logging
import numpy as np
from datetime import datetime

logger = logging.getLogger(__name__)

class Document(BaseModel):
    id: str
    content: str
    metadata: Dict[str, Any]
    embedding: Optional[List[float]] = None
    created_at: datetime
    updated_at: datetime

class SearchResult(BaseModel):
    document: Document
    score: float
    rank: int

class VectorStoreService:
    def __init__(self):
        self.documents: Dict[str, Document] = {}
        self.embeddings: Dict[str, List[float]] = {}

    async def add_document(self, doc_id: str, content: str, metadata: Dict[str, Any]) -> Document:
        """Add a document to the vector store"""
        # Generate a simple embedding (in production, use a real embedding model)
        embedding = self._generate_simple_embedding(content)
        
        document = Document(
            id=doc_id,
            content=content,
            metadata=metadata,
            embedding=embedding,
            created_at=datetime.now(),
            updated_at=datetime.now()
        )
        
        self.documents[doc_id] = document
        self.embeddings[doc_id] = embedding
        
        logger.info(f"Added document {doc_id} to vector store")
        return document

    async def similarity_search(self, query: str, k: int = 5) -> List[SearchResult]:
        """Perform similarity search"""
        if not self.documents:
            return []

        query_embedding = self._generate_simple_embedding(query)
        
        # Calculate similarities
        similarities = []
        for doc_id, doc_embedding in self.embeddings.items():
            similarity = self._cosine_similarity(query_embedding, doc_embedding)
            similarities.append((doc_id, similarity))
        
        # Sort by similarity and take top k
        similarities.sort(key=lambda x: x[1], reverse=True)
        top_results = similarities[:k]
        
        # Create search results
        results = []
        for rank, (doc_id, score) in enumerate(top_results):
            document = self.documents[doc_id]
            result = SearchResult(
                document=document,
                score=score,
                rank=rank + 1
            )
            results.append(result)
        
        logger.info(f"Found {len(results)} similar documents for query")
        return results

    async def delete_document(self, doc_id: str) -> bool:
        """Delete a document from the vector store"""
        if doc_id in self.documents:
            del self.documents[doc_id]
            del self.embeddings[doc_id]
            logger.info(f"Deleted document {doc_id} from vector store")
            return True
        return False

    async def get_document(self, doc_id: str) -> Optional[Document]:
        """Get a document by ID"""
        return self.documents.get(doc_id)

    async def list_documents(self) -> List[Document]:
        """List all documents"""
        return list(self.documents.values())

    def _generate_simple_embedding(self, text: str) -> List[float]:
        """Generate a simple embedding (placeholder for real embedding model)"""
        # This is a very simple hash-based embedding for demonstration
        # In production, use a proper embedding model like sentence-transformers
        words = text.lower().split()
        embedding = [0.0] * 384  # Standard embedding dimension
        
        for i, word in enumerate(words[:50]):  # Limit to first 50 words
            hash_val = hash(word) % 384
            embedding[hash_val] += 1.0 / (i + 1)  # Weight by position
        
        # Normalize
        norm = sum(x * x for x in embedding) ** 0.5
        if norm > 0:
            embedding = [x / norm for x in embedding]
        
        return embedding

    def _cosine_similarity(self, vec1: List[float], vec2: List[float]) -> float:
        """Calculate cosine similarity between two vectors"""
        dot_product = sum(a * b for a, b in zip(vec1, vec2))
        magnitude1 = sum(a * a for a in vec1) ** 0.5
        magnitude2 = sum(a * a for a in vec2) ** 0.5
        
        if magnitude1 == 0 or magnitude2 == 0:
            return 0.0
        
        return dot_product / (magnitude1 * magnitude2)

vector_store_service = VectorStoreService()