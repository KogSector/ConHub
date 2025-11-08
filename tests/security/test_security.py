#!/usr/bin/env python3
"""
Test script for Security microservice
Tests various endpoints with sample payloads
"""

import requests
import json
import sys

BASE_URL = "http://localhost:3012"

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

def test_security_scan():
    """Test security scan endpoint (requires auth)"""
    payload = {
        "target": "https://example.com",
        "scan_type": "basic"
    }
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.post(f"{BASE_URL}/api/security/scan", json=payload, headers=headers)
        print(f"Security scan: {response.status_code}")
        if response.status_code in [200, 201, 401, 403]:
            print("✓ Security scan endpoint accessible")
            return True
        else:
            print(f"✗ Security scan failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Security scan error: {e}")
        return False

def main():
    print("Testing Security Service...")
    print("=" * 50)

    # Test health
    if not test_health():
        print("Health check failed, aborting tests")
        sys.exit(1)

    # Test security scan
    test_security_scan()

    print("=" * 50)
    print("Security service tests completed")

if __name__ == "__main__":
    main()