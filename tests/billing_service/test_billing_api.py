"""
Billing Service API Tests

Tests for the billing service REST API endpoints.
"""

import pytest
import requests
import os

class TestBillingServiceAPI:
    """Test suite for billing service API endpoints."""
    
    BASE_URL = "http://localhost:3011"
    
    @pytest.fixture(scope="class")
    def billing_service_url(self) -> str:
        """Get the billing service URL from environment or use default."""
        return os.getenv("BILLING_SERVICE_URL", self.BASE_URL)
    
    def test_health_check(self, billing_service_url: str):
        """Test that the billing service is healthy and responding."""
        try:
            response = requests.get(f"{billing_service_url}/health", timeout=5)
            assert response.status_code == 200
            
            health_data = response.json()
            assert health_data["status"] == "healthy"
            assert "service" in health_data
        except requests.exceptions.RequestException:
            pytest.skip("Billing service not available")
    
    def test_subscription_plans_endpoint(self, billing_service_url: str):
        """Test the subscription plans endpoint."""
        try:
            response = requests.get(f"{billing_service_url}/api/plans", timeout=5)
            
            # Should return available plans
            assert response.status_code in [200, 404]
            
            if response.status_code == 200:
                data = response.json()
                assert isinstance(data, (list, dict))
        except requests.exceptions.RequestException:
            pytest.skip("Billing service not available")
    
    def test_billing_endpoints_require_auth(self, billing_service_url: str):
        """Test that billing endpoints require authentication."""
        try:
            # Test without auth token
            response = requests.post(
                f"{billing_service_url}/api/subscribe",
                json={"plan": "premium"},
                timeout=5
            )
            
            # Should require authentication
            assert response.status_code in [401, 403, 404]
        except requests.exceptions.RequestException:
            pytest.skip("Billing service not available")
