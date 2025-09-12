import os
from typing import List
from pydantic import BaseSettings

class Settings(BaseSettings):
    """Application settings."""
    
    # Server configuration
    environment: str = "development"
    host: str = "0.0.0.0"
    port: int = 8001
    log_level: str = "INFO"
    
    # Document store configuration
    use_in_memory_store: bool = True
    elasticsearch_host: str = "localhost"
    elasticsearch_port: int = 9200
    elasticsearch_username: str = ""
    elasticsearch_password: str = ""
    elasticsearch_index: str = "conhub-documents"
    
    # Model configuration
    embedding_model: str = "sentence-transformers/all-MiniLM-L6-v2"
    reader_model: str = "deepset/roberta-base-squad2"
    use_openai_embeddings: bool = False
    openai_api_key: str = ""
    
    # File processing
    max_file_size_mb: int = 50
    supported_extensions: List[str] = [".txt", ".pdf", ".docx", ".md", ".html"]
    
    # External services
    langchain_service_url: str = "http://localhost:3001"
    rust_backend_url: str = "http://localhost:8000"
    
    # Security
    api_key_header: str = "X-API-Key"
    api_key: str = ""
    
    # CORS
    cors_origins: List[str] = ["http://localhost:3000", "http://localhost:3001"]
    
    class Config:
        env_file = ".env"
        case_sensitive = False

_settings = None

def get_settings() -> Settings:
    """Get application settings (singleton)."""
    global _settings
    if _settings is None:
        _settings = Settings()
    return _settings
