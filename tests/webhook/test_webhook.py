#!/usr/bin/env python3
"""
Test script for Webhook microservice
Tests various endpoints with sample payloads
"""

import requests
import json
import sys

BASE_URL = "http://localhost:3015"

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

def test_github_webhook():
    """Test GitHub webhook endpoint"""
    payload = {
        "action": "push",
        "repository": {
            "name": "test-repo",
            "full_name": "test/test-repo"
        },
        "commits": [
            {
                "id": "abc123",
                "message": "Test commit"
            }
        ]
    }
    headers = {
        "X-GitHub-Event": "push",
        "X-Hub-Signature-256": "test_signature"
    }
    try:
        response = requests.post(f"{BASE_URL}/webhooks/github", json=payload, headers=headers)
        print(f"GitHub webhook: {response.status_code}")
        if response.status_code in [200, 201, 400, 401]:  # 400/401 for invalid signature
            print("✓ GitHub webhook endpoint accessible")
            return True
        else:
            print(f"✗ GitHub webhook failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ GitHub webhook error: {e}")
        return False

def main():
    print("Testing Webhook Service...")
    print("=" * 50)

    # Test health
    if not test_health():
        print("Health check failed, aborting tests")
        sys.exit(1)

    # Test webhook
    test_github_webhook()

    print("=" * 50)
    print("Webhook service tests completed")

if __name__ == "__main__":
    main()