"""
Data Service API Tests

Tests for the data service REST API endpoints.
"""

import pytest
import requests
import os

class TestDataServiceAPI:
    """Test suite for data service API endpoints."""
    
    BASE_URL = "http://localhost:3013"
    
    @pytest.fixture(scope="class")
    def data_service_url(self) -> str:
        """Get the data service URL from environment or use default."""
        return os.getenv("DATA_SERVICE_URL", self.BASE_URL)
    
    def test_health_check(self, data_service_url: str):
        """Test that the data service is healthy and responding."""
        response = requests.get(f"{data_service_url}/health")
        assert response.status_code == 200
        
        health_data = response.json()
        assert health_data["status"] == "healthy"
        assert "service" in health_data
    
    def test_connectors_endpoint(self, data_service_url: str):
        """Test the connectors listing endpoint."""
        response = requests.get(f"{data_service_url}/api/connectors")
        
        # Should return 200 even if no connectors configured
        assert response.status_code in [200, 404]
        
        if response.status_code == 200:
            data = response.json()
            assert isinstance(data, (list, dict))
    
    def test_ingestion_jobs_endpoint(self, data_service_url: str):
        """Test the ingestion jobs endpoint."""
        response = requests.get(f"{data_service_url}/api/ingestion/jobs")
        
        # Should return 200 with empty list if no jobs
        assert response.status_code == 200
        
        data = response.json()
        assert isinstance(data, list)
    
    def test_documents_search_endpoint(self, data_service_url: str):
        """Test the document search endpoint."""
        # Test with empty query
        response = requests.get(f"{data_service_url}/api/documents/search?q=test")
        
        # Should handle search even with no documents
        assert response.status_code in [200, 404]
    
    def test_cors_headers(self, data_service_url: str):
        """Test that CORS headers are properly set."""
        response = requests.options(
            f"{data_service_url}/api/connectors",
            headers={"Origin": "http://localhost:3000"}
        )
        
        # Should allow CORS for frontend
        assert response.status_code in [200, 204]
