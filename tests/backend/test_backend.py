#!/usr/bin/env python3
"""
Test script for Backend microservice
Tests various endpoints with sample payloads
"""

import requests
import json
import sys

BASE_URL = "http://localhost:8000"

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

def test_readiness():
    """Test readiness check endpoint"""
    try:
        response = requests.get(f"{BASE_URL}/ready")
        print(f"Readiness check: {response.status_code}")
        if response.status_code == 200:
            print("✓ Readiness check passed")
            return True
        else:
            print("✗ Readiness check failed")
            return False
    except Exception as e:
        print(f"✗ Readiness check error: {e}")
        return False

def test_graphql_query():
    """Test GraphQL endpoint with a simple query"""
    query = """
    query {
        __typename
    }
    """
    payload = {
        "query": query
    }
    try:
        response = requests.post(f"{BASE_URL}/api/graphql", json=payload)
        print(f"GraphQL query: {response.status_code}")
        if response.status_code == 200:
            data = response.json()
            if "data" in data:
                print("✓ GraphQL query passed")
                return True
            else:
                print(f"✗ GraphQL response error: {data}")
                return False
        else:
            print(f"✗ GraphQL query failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ GraphQL query error: {e}")
        return False

def main():
    print("Testing Backend Service...")
    print("=" * 50)

    # Test health
    if not test_health():
        print("Health check failed, aborting tests")
        sys.exit(1)

    # Test readiness
    test_readiness()

    # Test GraphQL
    test_graphql_query()

    print("=" * 50)
    print("Backend service tests completed")

if __name__ == "__main__":
    main()