#!/usr/bin/env python3
"""
Test script for Auth microservice
Tests various endpoints with sample payloads
"""

import requests
import json
import sys

BASE_URL = "http://localhost:3010"

def test_health():
    """Test health check endpoint"""
    try:
        response = requests.get(f"{BASE_URL}/health")
        print(f"Health check: {response.status_code}")
        if response.status_code == 200:
            print("✓ Health check passed")
            return True
        else:
            print("✗ Health check failed")
            return False
    except Exception as e:
        print(f"✗ Health check error: {e}")
        return False

def test_register():
    """Test user registration"""
    payload = {
        "email": "test@example.com",
        "password": "testpassword123",
        "first_name": "Test",
        "last_name": "User"
    }
    try:
        response = requests.post(f"{BASE_URL}/api/auth/register", json=payload)
        print(f"Register: {response.status_code}")
        if response.status_code in [200, 201, 409]:  # 409 if user exists
            print("✓ Register test passed")
            return True
        else:
            print(f"✗ Register failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Register error: {e}")
        return False

def test_login():
    """Test user login"""
    payload = {
        "email": "test@example.com",
        "password": "testpassword123"
    }
    try:
        response = requests.post(f"{BASE_URL}/api/auth/login", json=payload)
        print(f"Login: {response.status_code}")
        if response.status_code == 200:
            data = response.json()
            if "token" in data:
                print("✓ Login test passed")
                return data["token"]
            else:
                print("✗ Login response missing token")
                return None
        else:
            print(f"✗ Login failed: {response.text}")
            return None
    except Exception as e:
        print(f"✗ Login error: {e}")
        return None

def test_forgot_password():
    """Test forgot password"""
    payload = {
        "email": "test@example.com"
    }
    try:
        response = requests.post(f"{BASE_URL}/api/auth/forgot-password", json=payload)
        print(f"Forgot password: {response.status_code}")
        # Usually returns 200 even if email not found for security
        if response.status_code in [200, 404]:
            print("✓ Forgot password test passed")
            return True
        else:
            print(f"✗ Forgot password failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Forgot password error: {e}")
        return False

def test_oauth_providers():
    """Test OAuth provider list (mock)"""
    providers = ["google", "github", "microsoft"]
    for provider in providers:
        try:
            # This would redirect, so just check if endpoint exists
            response = requests.get(f"{BASE_URL}/api/auth/oauth/{provider}", allow_redirects=False)
            print(f"OAuth {provider}: {response.status_code}")
            if response.status_code in [302, 404]:  # Redirect or not configured
                print(f"✓ OAuth {provider} endpoint accessible")
            else:
                print(f"✗ OAuth {provider} unexpected status")
        except Exception as e:
            print(f"✗ OAuth {provider} error: {e}")

def main():
    print("Testing Auth Service...")
    print("=" * 50)

    # Test health
    if not test_health():
        print("Health check failed, aborting tests")
        sys.exit(1)

    # Test registration
    test_register()

    # Test login
    token = test_login()

    # Test forgot password
    test_forgot_password()

    # Test OAuth (basic check)
    test_oauth_providers()

    print("=" * 50)
    print("Auth service tests completed")

if __name__ == "__main__":
    main()