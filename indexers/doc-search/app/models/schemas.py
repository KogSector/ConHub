"""
Pydantic schemas for the Haystack service
"""
from pydantic import BaseModel
from typing import List, Dict, Any, Optional

class DocumentResponse(BaseModel):
    id: str
    message: str
    success: bool

class SearchRequest(BaseModel):
    query: str
    limit: int = 10

class SearchResult(BaseModel):
    document: Dict[str, Any]
    score: float
    matches: int

class SearchResponse(BaseModel):
    query: str
    results: List[Dict[str, Any]]
    total_results: int

class QuestionRequest(BaseModel):
    query: str
    top_k: int = 3

class QuestionResponse(BaseModel):
    query: str
    answer: str
    sources: List[Dict[str, Any]]
    confidence: float

class StatsResponse(BaseModel):
    total_documents: int
    total_words: int
    average_words_per_doc: float

class HealthResponse(BaseModel):
    status: str
    service: str