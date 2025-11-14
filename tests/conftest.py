"""
Pytest configuration for ConHub tests.
"""

import pytest
import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

@pytest.fixture(scope="session", autouse=True)
def setup_test_environment():
    """Setup test environment variables."""
    # Ensure we're in development mode for tests
    os.environ["NODE_ENV"] = "development"
    os.environ["ENVIRONMENT"] = "test"
