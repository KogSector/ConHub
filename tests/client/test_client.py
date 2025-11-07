#!/usr/bin/env python3
"""
Test script for Client microservice
Tests various endpoints with sample payloads
"""

import requests
import json
import sys

BASE_URL = "http://localhost:3014"

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

def test_get_agents():
    """Test getting agents (requires auth)"""
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.get(f"{BASE_URL}/api/client/agents", headers=headers)
        print(f"Get agents: {response.status_code}")
        if response.status_code in [200, 401, 403]:
            print("✓ Get agents endpoint accessible")
            return True
        else:
            print(f"✗ Get agents failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Get agents error: {e}")
        return False

def test_query_agents():
    """Test querying agents (requires auth)"""
    payload = {
        "query": "What is the weather today?",
        "agent_id": "general_agent"
    }
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.post(f"{BASE_URL}/api/client/query", json=payload, headers=headers)
        print(f"Query agents: {response.status_code}")
        if response.status_code in [200, 401, 403]:
            print("✓ Query agents endpoint accessible")
            return True
        else:
            print(f"✗ Query agents failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Query agents error: {e}")
        return False

def test_mcp_resources():
    """Test MCP resources (requires auth)"""
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.get(f"{BASE_URL}/api/client/mcp/resources", headers=headers)
        print(f"MCP resources: {response.status_code}")
        if response.status_code in [200, 401, 403]:
            print("✓ MCP resources endpoint accessible")
            return True
        else:
            print(f"✗ MCP resources failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ MCP resources error: {e}")
        return False

def main():
    print("Testing Client Service...")
    print("=" * 50)

    # Test health
    if not test_health():
        print("Health check failed, aborting tests")
        sys.exit(1)

    # Test agents
    test_get_agents()
    test_query_agents()

    # Test MCP
    test_mcp_resources()

    print("=" * 50)
    print("Client service tests completed")
    print("Note: Auth-required endpoints may return 401/403 without valid tokens")

if __name__ == "__main__":
    main()