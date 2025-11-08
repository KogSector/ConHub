#!/usr/bin/env python3
"""
Test script for Data microservice
Tests various endpoints with sample payloads
"""

import requests
import json
import sys

BASE_URL = "http://localhost:3013"

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

def test_list_repositories():
    """Test listing repositories (requires auth)"""
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.get(f"{BASE_URL}/api/data/repositories", headers=headers)
        print(f"List repositories: {response.status_code}")
        if response.status_code in [200, 401, 403]:
            print("✓ List repositories endpoint accessible")
            return True
        else:
            print(f"✗ List repositories failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ List repositories error: {e}")
        return False

def test_connect_repository():
    """Test connecting a repository (requires auth)"""
    payload = {
        "url": "https://github.com/test/repo",
        "name": "test-repo",
        "source_type": "github"
    }
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.post(f"{BASE_URL}/api/data/repositories", json=payload, headers=headers)
        print(f"Connect repository: {response.status_code}")
        if response.status_code in [200, 201, 401, 403]:
            print("✓ Connect repository endpoint accessible")
            return True
        else:
            print(f"✗ Connect repository failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Connect repository error: {e}")
        return False

def test_get_documents():
    """Test getting documents (requires auth)"""
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.get(f"{BASE_URL}/api/data/documents", headers=headers)
        print(f"Get documents: {response.status_code}")
        if response.status_code in [200, 401, 403]:
            print("✓ Get documents endpoint accessible")
            return True
        else:
            print(f"✗ Get documents failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Get documents error: {e}")
        return False

def test_index_url():
    """Test indexing a URL (requires auth)"""
    payload = {
        "url": "https://example.com",
        "title": "Example Page"
    }
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.post(f"{BASE_URL}/api/data/index/url", json=payload, headers=headers)
        print(f"Index URL: {response.status_code}")
        if response.status_code in [200, 201, 401, 403]:
            print("✓ Index URL endpoint accessible")
            return True
        else:
            print(f"✗ Index URL failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Index URL error: {e}")
        return False

def main():
    print("Testing Data Service...")
    print("=" * 50)

    # Test health
    if not test_health():
        print("Health check failed, aborting tests")
        sys.exit(1)

    # Test repositories
    test_list_repositories()
    test_connect_repository()

    # Test documents
    test_get_documents()

    # Test indexing
    test_index_url()

    print("=" * 50)
    print("Data service tests completed")
    print("Note: Auth-required endpoints may return 401/403 without valid tokens")

if __name__ == "__main__":
    main()