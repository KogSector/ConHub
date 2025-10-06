"""
Qdrant Vector Database Service
Handles vector operations for different data types with separate collections
"""

import os
import logging
from typing import List, Dict, Any, Optional
from qdrant_client import QdrantClient
from qdrant_client.http import models
from qdrant_client.http.models import Distance, VectorParams, PointStruct
import hashlib

logger = logging.getLogger(__name__)

class QdrantService:
    def __init__(self):
        self.client = QdrantClient(
            url=os.getenv("QDRANT_URL", "http://localhost:6333"),
            api_key=os.getenv("QDRANT_API_KEY")
        )
        
        # Collection names for different data types
        self.collections = {
            "code": os.getenv("QDRANT_COLLECTION_CODE", "conhub_code"),
            "documents": os.getenv("QDRANT_COLLECTION_DOCS", "conhub_documents"),
            "urls": os.getenv("QDRANT_COLLECTION_URLS", "conhub_urls"),
            "conversations": os.getenv("QDRANT_COLLECTION_CONVERSATIONS", "conhub_conversations")
        }
        
        self._ensure_collections()
    
    def _ensure_collections(self):
        """Create collections if they don't exist"""
        for collection_type, collection_name in self.collections.items():
            try:
                self.client.get_collection(collection_name)
                logger.info(f"Collection {collection_name} exists")
            except Exception:
                logger.info(f"Creating collection {collection_name}")
                self.client.create_collection(
                    collection_name=collection_name,
                    vectors_config=VectorParams(size=384, distance=Distance.COSINE)
                )
    
    def _generate_id(self, content: str) -> str:
        """Generate consistent ID from content"""
        return hashlib.md5(content.encode()).hexdigest()
    
    def add_code_vectors(self, vectors: List[Dict[str, Any]]) -> bool:
        """Add code vectors to code collection"""
        try:
            points = []
            for vector in vectors:
                point_id = self._generate_id(vector["content"])
                points.append(PointStruct(
                    id=point_id,
                    vector=vector["embedding"],
                    payload={
                        "content": vector["content"],
                        "file_path": vector.get("file_path", ""),
                        "language": vector.get("language", ""),
                        "function_name": vector.get("function_name", ""),
                        "class_name": vector.get("class_name", ""),
                        "repository": vector.get("repository", ""),
                        "type": "code"
                    }
                ))
            
            self.client.upsert(
                collection_name=self.collections["code"],
                points=points
            )
            logger.info(f"Added {len(points)} code vectors")
            return True
        except Exception as e:
            logger.error(f"Error adding code vectors: {e}")
            return False
    
    def add_document_vectors(self, vectors: List[Dict[str, Any]]) -> bool:
        """Add document vectors to documents collection"""
        try:
            points = []
            for vector in vectors:
                point_id = self._generate_id(vector["content"])
                points.append(PointStruct(
                    id=point_id,
                    vector=vector["embedding"],
                    payload={
                        "content": vector["content"],
                        "title": vector.get("title", ""),
                        "source": vector.get("source", ""),
                        "file_type": vector.get("file_type", ""),
                        "metadata": vector.get("metadata", {}),
                        "type": "document"
                    }
                ))
            
            self.client.upsert(
                collection_name=self.collections["documents"],
                points=points
            )
            logger.info(f"Added {len(points)} document vectors")
            return True
        except Exception as e:
            logger.error(f"Error adding document vectors: {e}")
            return False
    
    def add_url_vectors(self, vectors: List[Dict[str, Any]]) -> bool:
        """Add URL content vectors to URLs collection"""
        try:
            points = []
            for vector in vectors:
                point_id = self._generate_id(vector["content"])
                points.append(PointStruct(
                    id=point_id,
                    vector=vector["embedding"],
                    payload={
                        "content": vector["content"],
                        "url": vector.get("url", ""),
                        "title": vector.get("title", ""),
                        "domain": vector.get("domain", ""),
                        "crawl_depth": vector.get("crawl_depth", 0),
                        "type": "url"
                    }
                ))
            
            self.client.upsert(
                collection_name=self.collections["urls"],
                points=points
            )
            logger.info(f"Added {len(points)} URL vectors")
            return True
        except Exception as e:
            logger.error(f"Error adding URL vectors: {e}")
            return False
    
    def add_conversation_vectors(self, vectors: List[Dict[str, Any]]) -> bool:
        """Add conversation vectors to conversations collection"""
        try:
            points = []
            for vector in vectors:
                point_id = self._generate_id(vector["content"])
                points.append(PointStruct(
                    id=point_id,
                    vector=vector["embedding"],
                    payload={
                        "content": vector["content"],
                        "platform": vector.get("platform", ""),
                        "channel": vector.get("channel", ""),
                        "author": vector.get("author", ""),
                        "timestamp": vector.get("timestamp", ""),
                        "thread_id": vector.get("thread_id", ""),
                        "type": "conversation"
                    }
                ))
            
            self.client.upsert(
                collection_name=self.collections["conversations"],
                points=points
            )
            logger.info(f"Added {len(points)} conversation vectors")
            return True
        except Exception as e:
            logger.error(f"Error adding conversation vectors: {e}")
            return False
    
    def search_vectors(self, query_vector: List[float], collection_type: str, limit: int = 10, filter_conditions: Optional[Dict] = None) -> List[Dict[str, Any]]:
        """Search vectors in specified collection"""
        try:
            collection_name = self.collections.get(collection_type)
            if not collection_name:
                raise ValueError(f"Unknown collection type: {collection_type}")
            
            search_filter = None
            if filter_conditions:
                search_filter = models.Filter(
                    must=[
                        models.FieldCondition(
                            key=key,
                            match=models.MatchValue(value=value)
                        ) for key, value in filter_conditions.items()
                    ]
                )
            
            results = self.client.search(
                collection_name=collection_name,
                query_vector=query_vector,
                limit=limit,
                query_filter=search_filter
            )
            
            return [
                {
                    "id": result.id,
                    "score": result.score,
                    "payload": result.payload
                }
                for result in results
            ]
        except Exception as e:
            logger.error(f"Error searching vectors: {e}")
            return []
    
    def search_all_collections(self, query_vector: List[float], limit: int = 5) -> Dict[str, List[Dict[str, Any]]]:
        """Search across all collections"""
        results = {}
        for collection_type in self.collections.keys():
            results[collection_type] = self.search_vectors(query_vector, collection_type, limit)
        return results
    
    def get_collection_info(self) -> Dict[str, Dict[str, Any]]:
        """Get information about all collections"""
        info = {}
        for collection_type, collection_name in self.collections.items():
            try:
                collection_info = self.client.get_collection(collection_name)
                info[collection_type] = {
                    "name": collection_name,
                    "vectors_count": collection_info.vectors_count,
                    "status": collection_info.status
                }
            except Exception as e:
                info[collection_type] = {"error": str(e)}
        return info

# Global instance
qdrant_service = QdrantService()