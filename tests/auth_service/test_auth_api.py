"""
ConHub Authentication Tests

Tests for the authentication service including user registration, login, and token validation.
"""

import pytest
import requests
import json
import os
from typing import Dict, Any


class TestAuthService:
    """Test suite for ConHub authentication service."""
    
    BASE_URL = "http://localhost:3010"
    
    @pytest.fixture(scope="class")
    def auth_service_url(self) -> str:
        """Get the auth service URL from environment or use default."""
        return os.getenv("AUTH_SERVICE_URL", self.BASE_URL)
    
    def test_health_check(self, auth_service_url: str):
        """Test that the auth service is healthy and responding."""
        response = requests.get(f"{auth_service_url}/health")
        assert response.status_code == 200
        
        health_data = response.json()
        assert health_data["status"] == "healthy"
        assert health_data["service"] == "auth-service"
        assert health_data["database"] == "connected"
    
    def test_user_registration_success(self, auth_service_url: str):
        """Test successful user registration."""
        test_user = {
            "email": "test_user@example.com",
            "password": "TestPassword123!",
            "name": "Test User",
            "organization": "Test Org"
        }
        
        response = requests.post(
            f"{auth_service_url}/api/auth/register",
            json=test_user,
            headers={"Content-Type": "application/json"}
        )
        
        assert response.status_code == 201
        
        data = response.json()
        assert "user" in data
        assert "token" in data
        assert "refresh_token" in data
        assert data["user"]["email"] == test_user["email"]
        assert data["user"]["name"] == test_user["name"]
        
        # Cleanup - delete the test user
        self._cleanup_user(test_user["email"])
    
    def test_user_registration_duplicate_email(self, auth_service_url: str):
        """Test that duplicate email registration fails."""
        test_user = {
            "email": "duplicate_test@example.com",
            "password": "TestPassword123!",
            "name": "Test User"
        }
        
        # First registration should succeed
        response1 = requests.post(
            f"{auth_service_url}/api/auth/register",
            json=test_user,
            headers={"Content-Type": "application/json"}
        )
        assert response1.status_code == 201
        
        # Second registration should fail
        response2 = requests.post(
            f"{auth_service_url}/api/auth/register",
            json=test_user,
            headers={"Content-Type": "application/json"}
        )
        assert response2.status_code == 400
        
        data = response2.json()
        assert "error" in data
        assert "already exists" in data["details"].lower()
        
        # Cleanup
        self._cleanup_user(test_user["email"])
    
    def test_user_registration_invalid_password(self, auth_service_url: str):
        """Test that weak passwords are rejected."""
        test_user = {
            "email": "weak_password_test@example.com",
            "password": "123",  # Too weak
            "name": "Test User"
        }
        
        response = requests.post(
            f"{auth_service_url}/api/auth/register",
            json=test_user,
            headers={"Content-Type": "application/json"}
        )
        
        assert response.status_code == 400
        data = response.json()
        assert "error" in data
    
    def test_user_registration_missing_fields(self, auth_service_url: str):
        """Test that missing required fields are rejected."""
        incomplete_user = {
            "email": "incomplete@example.com"
            # Missing password and name
        }
        
        response = requests.post(
            f"{auth_service_url}/api/auth/register",
            json=incomplete_user,
            headers={"Content-Type": "application/json"}
        )
        
        assert response.status_code == 400
    
    def _cleanup_user(self, email: str):
        """Helper method to cleanup test users from database."""
        # This would require a cleanup endpoint or direct database access
        # For now, we'll skip cleanup in tests
        pass


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
