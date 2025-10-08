import os
from typing import List
from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    # Server settings
    host: str = "0.0.0.0"
    port: int = 8001
    environment: str = "development"
    log_level: str = "INFO"
    
    # CORS settings
    cors_origins: List[str] = ["http://localhost:3000", "http://localhost:3001", "http://localhost:3002", "http://localhost:3003"]
    
    # Haystack settings
    document_store_type: str = "memory"  # memory, elasticsearch, etc.
    embedding_model: str = "sentence-transformers/all-MiniLM-L6-v2"
    
    class Config:
        env_file = ".env"

def get_settings() -> Settings:
    return Settings()