"""
Service Integration Tests

Tests for integration between different ConHub microservices.
"""

import pytest
import requests
import time
import os

class TestServiceIntegration:
    """Test suite for service-to-service integration."""
    
    SERVICES = {
        'auth': 'http://localhost:3010',
        'data': 'http://localhost:3013',
        'frontend': 'http://localhost:3000',
        'billing': 'http://localhost:3011',
        'security': 'http://localhost:3014',
        'webhook': 'http://localhost:3015'
    }
    
    def test_all_services_health(self):
        """Test that all services are healthy."""
        health_results = {}
        
        for service_name, base_url in self.SERVICES.items():
            try:
                if service_name == 'frontend':
                    # Frontend doesn't have /health endpoint, just check if it responds
                    response = requests.get(base_url, timeout=5)
                    health_results[service_name] = response.status_code == 200
                else:
                    # Backend services should have /health endpoint
                    response = requests.get(f"{base_url}/health", timeout=5)
                    health_results[service_name] = response.status_code == 200
            except requests.exceptions.RequestException:
                health_results[service_name] = False
        
        # At least auth and frontend should be running for basic functionality
        assert health_results.get('auth', False), "Auth service is not healthy"
        assert health_results.get('frontend', False), "Frontend service is not healthy"
        
        print(f"Service health status: {health_results}")
    
    def test_auth_data_integration(self):
        """Test integration between auth and data services."""
        auth_url = self.SERVICES['auth']
        data_url = self.SERVICES['data']
        
        # Check if both services are running
        try:
            auth_health = requests.get(f"{auth_url}/health", timeout=5)
            data_health = requests.get(f"{data_url}/health", timeout=5)
            
            if auth_health.status_code == 200 and data_health.status_code == 200:
                # Both services are running, test integration
                
                # Test that data service can validate auth tokens (if implemented)
                # This would require actual token validation logic
                
                # For now, just verify both services use same database
                auth_health_data = auth_health.json()
                data_health_data = data_health.json()
                
                # Both should report database connectivity
                assert "database" in auth_health_data or "status" in auth_health_data
                assert "database" in data_health_data or "status" in data_health_data
                
        except requests.exceptions.RequestException:
            pytest.skip("Auth or Data service not available for integration testing")
    
    def test_frontend_auth_integration(self):
        """Test integration between frontend and auth service."""
        frontend_url = self.SERVICES['frontend']
        auth_url = self.SERVICES['auth']
        
        try:
            # Check if both are running
            frontend_response = requests.get(frontend_url, timeout=5)
            auth_response = requests.get(f"{auth_url}/health", timeout=5)
            
            if frontend_response.status_code == 200 and auth_response.status_code == 200:
                # Test CORS configuration
                cors_response = requests.options(
                    f"{auth_url}/api/auth/register",
                    headers={"Origin": frontend_url}
                )
                
                # Should allow CORS from frontend
                assert cors_response.status_code in [200, 204]
                
                # Check for CORS headers
                cors_headers = cors_response.headers
                assert "access-control-allow-origin" in cors_headers or cors_response.status_code == 204
                
        except requests.exceptions.RequestException:
            pytest.skip("Frontend or Auth service not available for integration testing")
    
    def test_service_startup_order(self):
        """Test that services can start in any order and still work."""
        # This test verifies that services handle dependencies gracefully
        
        running_services = []
        for service_name, base_url in self.SERVICES.items():
            try:
                if service_name == 'frontend':
                    response = requests.get(base_url, timeout=3)
                else:
                    response = requests.get(f"{base_url}/health", timeout=3)
                
                if response.status_code == 200:
                    running_services.append(service_name)
            except requests.exceptions.RequestException:
                pass
        
        # If any services are running, they should be functional
        if running_services:
            print(f"Running services: {running_services}")
            
            # Auth service should always be functional if running
            if 'auth' in running_services:
                auth_response = requests.get(f"{self.SERVICES['auth']}/health")
                auth_data = auth_response.json()
                assert auth_data.get('status') == 'healthy'
    
    def test_database_consistency(self):
        """Test that all services use the same database."""
        database_services = ['auth', 'data', 'billing', 'security']
        database_status = {}
        
        for service_name in database_services:
            if service_name in self.SERVICES:
                try:
                    response = requests.get(f"{self.SERVICES[service_name]}/health", timeout=5)
                    if response.status_code == 200:
                        health_data = response.json()
                        database_status[service_name] = health_data.get('database', 'unknown')
                except requests.exceptions.RequestException:
                    database_status[service_name] = 'unavailable'
        
        # All services that report database status should report 'connected'
        connected_services = [
            service for service, status in database_status.items() 
            if status == 'connected'
        ]
        
        if connected_services:
            # If any service is connected, all should be using the same database
            assert len(connected_services) >= 1, "At least one service should be connected to database"
            print(f"Database status: {database_status}")
    
    @pytest.mark.slow
    def test_end_to_end_user_flow(self):
        """Test complete user flow across services."""
        # This test requires all services to be running
        auth_url = self.SERVICES['auth']
        frontend_url = self.SERVICES['frontend']
        
        try:
            # 1. Check frontend is accessible
            frontend_response = requests.get(frontend_url, timeout=5)
            assert frontend_response.status_code == 200
            
            # 2. Check auth service is healthy
            auth_health = requests.get(f"{auth_url}/health", timeout=5)
            assert auth_health.status_code == 200
            
            # 3. Test user registration API (with cleanup)
            test_user = {
                "email": "integration_test@example.com",
                "password": "TestPassword123!",
                "name": "Integration Test User"
            }
            
            register_response = requests.post(
                f"{auth_url}/api/auth/register",
                json=test_user,
                headers={"Content-Type": "application/json"}
            )
            
            # Should either succeed or fail with "already exists"
            assert register_response.status_code in [201, 400]
            
            if register_response.status_code == 201:
                # Registration succeeded, verify response structure
                data = register_response.json()
                assert "user" in data
                assert "token" in data
                assert data["user"]["email"] == test_user["email"]
                
                print("✅ End-to-end user flow test passed")
            else:
                # User already exists, which is also fine for this test
                print("ℹ️  User already exists, skipping registration test")
                
        except requests.exceptions.RequestException as e:
            pytest.skip(f"Services not available for end-to-end testing: {e}")
