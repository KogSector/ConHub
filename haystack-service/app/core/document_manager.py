"""
Simplified Document Manager for ConHub
Handles document processing and search without complex Haystack dependencies
"""
import os
import json
from typing import List, Dict, Any, Optional
from pathlib import Path
import hashlib
from datetime import datetime

class DocumentManager:
    def __init__(self, storage_path: str = "documents"):
        self.storage_path = Path(storage_path)
        self.storage_path.mkdir(exist_ok=True)
        self.index_file = self.storage_path / "index.json"
        self.documents = self._load_index()
    
    def _load_index(self) -> Dict[str, Any]:
        """Load document index from file"""
        if self.index_file.exists():
            try:
                with open(self.index_file, 'r', encoding='utf-8') as f:
                    return json.load(f)
            except Exception:
                return {}
        return {}
    
    def _save_index(self):
        """Save document index to file"""
        with open(self.index_file, 'w', encoding='utf-8') as f:
            json.dump(self.documents, f, indent=2, ensure_ascii=False)
    
    def _generate_doc_id(self, content: str) -> str:
        """Generate unique document ID"""
        return hashlib.md5(content.encode()).hexdigest()
    
    def add_document(self, content: str, metadata: Optional[Dict] = None) -> str:
        """Add a document to the collection"""
        doc_id = self._generate_doc_id(content)
        
        document = {
            "id": doc_id,
            "content": content,
            "metadata": metadata or {},
            "created_at": datetime.now().isoformat(),
            "word_count": len(content.split())
        }
        
        self.documents[doc_id] = document
        self._save_index()
        return doc_id
    
    def search_documents(self, query: str, limit: int = 10) -> List[Dict]:
        """Simple text-based search"""
        query_lower = query.lower()
        results = []
        
        for doc_id, doc in self.documents.items():
            content_lower = doc["content"].lower()
            if query_lower in content_lower:
                # Simple relevance scoring based on query frequency
                score = content_lower.count(query_lower) / len(content_lower.split())
                results.append({
                    "document": doc,
                    "score": score,
                    "matches": content_lower.count(query_lower)
                })
        
        # Sort by relevance score
        results.sort(key=lambda x: x["score"], reverse=True)
        return results[:limit]
    
    def get_document(self, doc_id: str) -> Optional[Dict]:
        """Get document by ID"""
        return self.documents.get(doc_id)
    
    def delete_document(self, doc_id: str) -> bool:
        """Delete document by ID"""
        if doc_id in self.documents:
            del self.documents[doc_id]
            self._save_index()
            return True
        return False
    
    def get_stats(self) -> Dict:
        """Get collection statistics"""
        total_docs = len(self.documents)
        total_words = sum(doc.get("word_count", 0) for doc in self.documents.values())
        
        return {
            "total_documents": total_docs,
            "total_words": total_words,
            "average_words_per_doc": total_words / total_docs if total_docs > 0 else 0
        }
    
    def ask_question(self, question: str, context_limit: int = 3) -> Dict:
        """Simple Q&A based on document search"""
        # Search for relevant documents
        search_results = self.search_documents(question, limit=context_limit)
        
        if not search_results:
            return {
                "answer": "I couldn't find any relevant information in the documents.",
                "sources": [],
                "confidence": 0.0
            }
        
        # Extract context from top results
        context_docs = []
        for result in search_results:
            doc = result["document"]
            context_docs.append({
                "id": doc["id"],
                "content": doc["content"][:500] + "..." if len(doc["content"]) > 500 else doc["content"],
                "score": result["score"]
            })
        
        # Simple answer generation (in a real implementation, you'd use an LLM here)
        answer = f"Based on the documents, here are the most relevant excerpts for '{question}':\n\n"
        for i, doc in enumerate(context_docs, 1):
            answer += f"{i}. {doc['content']}\n\n"
        
        return {
            "answer": answer.strip(),
            "sources": context_docs,
            "confidence": search_results[0]["score"] if search_results else 0.0
        }