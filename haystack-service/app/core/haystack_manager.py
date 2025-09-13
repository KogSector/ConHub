import logging
from typing import List, Dict, Any, Optional
from haystack import Document, Pipeline
from haystack.components.embedders import SentenceTransformersTextEmbedder, SentenceTransformersDocumentEmbedder
from haystack.components.retrievers import InMemoryEmbeddingRetriever
from haystack.document_stores.in_memory import InMemoryDocumentStore
from haystack.components.writers import DocumentWriter
from haystack.components.generators import OpenAIGenerator
from haystack.components.builders import PromptBuilder

logger = logging.getLogger(__name__)

class HaystackManager:
    def __init__(self):
        self.document_store = None
        self.indexing_pipeline = None
        self.search_pipeline = None
        self.qa_pipeline = None
        
    async def initialize(self):
        """Initialize Haystack components."""
        try:
            # Initialize document store
            self.document_store = InMemoryDocumentStore()
            
            # Initialize embedder
            embedder = SentenceTransformersDocumentEmbedder(model="sentence-transformers/all-MiniLM-L6-v2")
            embedder.warm_up()
            
            # Create indexing pipeline
            self.indexing_pipeline = Pipeline()
            self.indexing_pipeline.add_component("embedder", embedder)
            self.indexing_pipeline.add_component("writer", DocumentWriter(document_store=self.document_store))
            self.indexing_pipeline.connect("embedder", "writer")
            
            # Create search pipeline
            text_embedder = SentenceTransformersTextEmbedder(model="sentence-transformers/all-MiniLM-L6-v2")
            retriever = InMemoryEmbeddingRetriever(document_store=self.document_store)
            
            self.search_pipeline = Pipeline()
            self.search_pipeline.add_component("text_embedder", text_embedder)
            self.search_pipeline.add_component("retriever", retriever)
            self.search_pipeline.connect("text_embedder.embedding", "retriever.query_embedding")
            
            logger.info("Haystack manager initialized successfully")
            
        except Exception as e:
            logger.error(f"Failed to initialize Haystack manager: {e}")
            raise
    
    async def index_documents(self, documents: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Index documents in the document store."""
        try:
            haystack_docs = []
            for doc in documents:
                haystack_doc = Document(
                    content=doc["content"],
                    meta=doc.get("meta", {})
                )
                haystack_docs.append(haystack_doc)
            
            result = self.indexing_pipeline.run({"embedder": {"documents": haystack_docs}})
            
            return {
                "index_id": "memory_store",
                "document_count": len(haystack_docs)
            }
            
        except Exception as e:
            logger.error(f"Error indexing documents: {e}")
            raise
    
    async def process_uploaded_file(self, file, metadata: Optional[str] = None) -> Dict[str, Any]:
        """Process an uploaded file and index it."""
        try:
            content = await file.read()
            text_content = content.decode('utf-8')
            
            doc = Document(
                content=text_content,
                meta={"filename": file.filename, "metadata": metadata or ""}
            )
            
            result = self.indexing_pipeline.run({"embedder": {"documents": [doc]}})
            
            return {
                "index_id": "memory_store",
                "document_count": 1
            }
            
        except Exception as e:
            logger.error(f"Error processing uploaded file: {e}")
            raise
    
    async def search_documents(self, query: str, top_k: int = 10, filters: Optional[Dict] = None) -> List[Dict[str, Any]]:
        """Search documents using semantic search."""
        try:
            result = self.search_pipeline.run({
                "text_embedder": {"text": query},
                "retriever": {"top_k": top_k}
            })
            
            documents = result["retriever"]["documents"]
            
            search_results = []
            for doc in documents:
                search_results.append({
                    "content": doc.content,
                    "score": doc.score if hasattr(doc, 'score') else 0.0,
                    "meta": doc.meta
                })
            
            return search_results
            
        except Exception as e:
            logger.error(f"Error searching documents: {e}")
            raise
    
    async def answer_question(self, question: str, top_k: int = 3, filters: Optional[Dict] = None) -> List[Dict[str, Any]]:
        """Answer a question using retrieved documents."""
        try:
            # For now, just return search results
            # In a full implementation, you'd use a generator component
            return await self.search_documents(question, top_k, filters)
            
        except Exception as e:
            logger.error(f"Error answering question: {e}")
            raise
    
    async def get_document_stats(self) -> Dict[str, Any]:
        """Get statistics about indexed documents."""
        try:
            count = self.document_store.count_documents()
            return {
                "total_documents": count,
                "document_store_type": "InMemoryDocumentStore"
            }
            
        except Exception as e:
            logger.error(f"Error getting document stats: {e}")
            raise
    
    async def cleanup(self):
        """Cleanup resources."""
        logger.info("Cleaning up Haystack manager")
        # Add any cleanup logic here