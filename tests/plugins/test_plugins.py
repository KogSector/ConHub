#!/usr/bin/env python3
"""
Test script for Plugins microservice
Tests various endpoints with sample payloads
"""

import requests
import json
import sys

BASE_URL = "http://localhost:3020"

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

def test_list_plugins():
    """Test listing plugins (requires auth)"""
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.get(f"{BASE_URL}/api/plugins", headers=headers)
        print(f"List plugins: {response.status_code}")
        if response.status_code in [200, 401, 403]:
            print("✓ List plugins endpoint accessible")
            return True
        else:
            print(f"✗ List plugins failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ List plugins error: {e}")
        return False

def test_execute_plugin():
    """Test executing a plugin (requires auth)"""
    payload = {
        "plugin_id": "filesystem",
        "action": "list_files",
        "params": {
            "path": "/tmp"
        }
    }
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.post(f"{BASE_URL}/api/plugins/execute", json=payload, headers=headers)
        print(f"Execute plugin: {response.status_code}")
        if response.status_code in [200, 401, 403, 404]:  # 404 if plugin not found
            print("✓ Execute plugin endpoint accessible")
            return True
        else:
            print(f"✗ Execute plugin failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Execute plugin error: {e}")
        return False

def main():
    print("Testing Plugins Service...")
    print("=" * 50)

    # Test health
    if not test_health():
        print("Health check failed, aborting tests")
        sys.exit(1)

    # Test plugins
    test_list_plugins()
    test_execute_plugin()

    print("=" * 50)
    print("Plugins service tests completed")

if __name__ == "__main__":
    main()