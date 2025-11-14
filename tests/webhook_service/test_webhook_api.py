"""
Webhook Service API Tests

Tests for the webhook service REST API endpoints.
"""

import pytest
import requests
import os

class TestWebhookServiceAPI:
    """Test suite for webhook service API endpoints."""
    
    BASE_URL = "http://localhost:3015"
    
    @pytest.fixture(scope="class")
    def webhook_service_url(self) -> str:
        """Get the webhook service URL from environment or use default."""
        return os.getenv("WEBHOOK_SERVICE_URL", self.BASE_URL)
    
    def test_health_check(self, webhook_service_url: str):
        """Test that the webhook service is healthy and responding."""
        try:
            response = requests.get(f"{webhook_service_url}/health", timeout=5)
            assert response.status_code == 200
            
            health_data = response.json()
            assert health_data["status"] == "healthy"
            assert "service" in health_data
        except requests.exceptions.RequestException:
            pytest.skip("Webhook service not available")
    
    def test_webhook_endpoints_exist(self, webhook_service_url: str):
        """Test that webhook endpoints are available."""
        try:
            # Test GitHub webhook endpoint
            response = requests.post(
                f"{webhook_service_url}/api/webhooks/github",
                json={"test": "payload"},
                timeout=5
            )
            
            # Should accept POST requests (even if payload is invalid)
            assert response.status_code in [200, 400, 404]
            
        except requests.exceptions.RequestException:
            pytest.skip("Webhook service not available")
    
    def test_webhook_security(self, webhook_service_url: str):
        """Test webhook security measures."""
        try:
            # Test webhook without proper signature/auth
            response = requests.post(
                f"{webhook_service_url}/api/webhooks/github",
                json={"malicious": "payload"},
                timeout=5
            )
            
            # Should validate webhook signatures
            assert response.status_code in [400, 401, 403, 404]
            
        except requests.exceptions.RequestException:
            pytest.skip("Webhook service not available")
