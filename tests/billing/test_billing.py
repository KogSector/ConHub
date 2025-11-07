#!/usr/bin/env python3
"""
Test script for Billing microservice
Tests various endpoints with sample payloads
"""

import requests
import json
import sys

BASE_URL = "http://localhost:3011"

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

def test_create_subscription():
    """Test creating a subscription (requires auth)"""
    payload = {
        "plan_id": "basic_plan",
        "payment_method_id": "pm_test_card"
    }
    headers = {
        "Authorization": "Bearer test_token"  # Would need real token
    }
    try:
        response = requests.post(f"{BASE_URL}/api/billing/subscription", json=payload, headers=headers)
        print(f"Create subscription: {response.status_code}")
        if response.status_code in [200, 201, 401, 403]:  # 401/403 if auth fails
            print("✓ Create subscription endpoint accessible")
            return True
        else:
            print(f"✗ Create subscription failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Create subscription error: {e}")
        return False

def test_get_invoices():
    """Test getting invoices (requires auth)"""
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.get(f"{BASE_URL}/api/billing/invoices", headers=headers)
        print(f"Get invoices: {response.status_code}")
        if response.status_code in [200, 401, 403]:
            print("✓ Get invoices endpoint accessible")
            return True
        else:
            print(f"✗ Get invoices failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Get invoices error: {e}")
        return False

def test_add_payment_method():
    """Test adding payment method (requires auth)"""
    payload = {
        "type": "card",
        "card": {
            "number": "4242424242424242",
            "exp_month": 12,
            "exp_year": 2025,
            "cvc": "123"
        }
    }
    headers = {
        "Authorization": "Bearer test_token"
    }
    try:
        response = requests.post(f"{BASE_URL}/api/billing/payment-method", json=payload, headers=headers)
        print(f"Add payment method: {response.status_code}")
        if response.status_code in [200, 201, 401, 403]:
            print("✓ Add payment method endpoint accessible")
            return True
        else:
            print(f"✗ Add payment method failed: {response.text}")
            return False
    except Exception as e:
        print(f"✗ Add payment method error: {e}")
        return False

def main():
    print("Testing Billing Service...")
    print("=" * 50)

    # Test health
    if not test_health():
        print("Health check failed, aborting tests")
        sys.exit(1)

    # Test subscription creation
    test_create_subscription()

    # Test getting invoices
    test_get_invoices()

    # Test adding payment method
    test_add_payment_method()

    print("=" * 50)
    print("Billing service tests completed")
    print("Note: Auth-required endpoints may return 401/403 without valid tokens")

if __name__ == "__main__":
    main()