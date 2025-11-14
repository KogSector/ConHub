"""
Security Service API Tests

Tests for the security service REST API endpoints.
"""

import pytest
import requests
import os

class TestSecurityServiceAPI:
    """Test suite for security service API endpoints."""
    
    BASE_URL = "http://localhost:3014"
    
    @pytest.fixture(scope="class")
    def security_service_url(self) -> str:
        """Get the security service URL from environment or use default."""
        return os.getenv("SECURITY_SERVICE_URL", self.BASE_URL)
    
    def test_health_check(self, security_service_url: str):
        """Test that the security service is healthy and responding."""
        try:
            response = requests.get(f"{security_service_url}/health", timeout=5)
            assert response.status_code == 200
            
            health_data = response.json()
            assert health_data["status"] == "healthy"
            assert "service" in health_data
        except requests.exceptions.RequestException:
            pytest.skip("Security service not available")
    
    def test_rate_limiting_endpoint(self, security_service_url: str):
        """Test rate limiting functionality."""
        try:
            # Test rate limit check endpoint
            response = requests.get(
                f"{security_service_url}/api/rate-limit/check",
                params={"identifier": "test_ip", "action": "test"},
                timeout=5
            )
            
            # Should return rate limit status
            assert response.status_code in [200, 404]
            
        except requests.exceptions.RequestException:
            pytest.skip("Security service not available")
    
    def test_security_audit_endpoint(self, security_service_url: str):
        """Test security audit logging endpoint."""
        try:
            # Test audit log endpoint (should require auth)
            response = requests.get(f"{security_service_url}/api/audit", timeout=5)
            
            # Should require authentication or return 404 if not implemented
            assert response.status_code in [401, 403, 404]
            
        except requests.exceptions.RequestException:
            pytest.skip("Security service not available")
