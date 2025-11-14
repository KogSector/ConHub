# ConHub Tests

This directory contains all test suites for ConHub microservices, organized by service.

## Test Structure

### Service-Specific Tests
- `auth_service/` - Authentication service tests (API, database, JWT)
- `data_service/` - Data service tests (API, connectors, ingestion)
- `frontend_service/` - Frontend tests (Selenium E2E tests)
- `billing_service/` - Billing service tests (API, subscriptions)
- `security_service/` - Security service tests (rate limiting, audit)
- `webhook_service/` - Webhook service tests (GitHub, security)
- `integration/` - Cross-service integration tests

### Test Types
- **API Tests**: REST endpoint testing with requests
- **Database Tests**: Direct database testing with psycopg2
- **E2E Tests**: Frontend testing with Selenium WebDriver
- **Integration Tests**: Service-to-service communication testing

## Running Tests

### Setup
```bash
# Install test dependencies
pip install -r requirements.txt
```

### Run All Tests
```bash
# Run all tests with verbose output
pytest -v

# Run tests for specific service
pytest auth_service/ -v
pytest frontend_service/ -v

# Run integration tests only
pytest integration/ -v

# Run with coverage
pytest --cov=. -v
```

### Run Specific Test Types
```bash
# Run only API tests (exclude Selenium)
pytest -m "not selenium" -v

# Run only database tests
pytest -m database -v

# Run only integration tests
pytest -m integration -v

# Run fast tests only (exclude slow tests)
pytest -m "not slow" -v

# Run specific service tests
pytest -m auth -v
pytest -m frontend -v
```

### Run Specific Tests
```bash
# Run specific test file
pytest auth_service/test_auth_api.py -v

# Run specific test class
pytest auth_service/test_auth_api.py::TestAuthService -v

# Run specific test method
pytest auth_service/test_auth_api.py::TestAuthService::test_user_registration_success -v
```

## Test Configuration

- `conftest.py` - Pytest configuration and fixtures
- `requirements.txt` - Test dependencies
- Environment variables are loaded from `.env` file in project root

## Environment Variables

Tests require the following environment variables:
- `AUTH_SERVICE_URL` - Auth service URL (default: http://localhost:3010)
- `NODE_ENV` - Set to "development" for tests
- `DATABASE_URL_NEON` - Database connection for integration tests

## Test Data

Tests use temporary test data that should be cleaned up automatically. If cleanup fails, you can manually remove test users using the database manager:

```bash
python ../scripts/database_manager.py delete-user test_user@example.com
```

## Adding New Tests

1. Create new test files following the `test_*.py` naming convention
2. Use the `TestClassName` format for test classes
3. Use descriptive test method names starting with `test_`
4. Include docstrings explaining what each test validates
5. Use appropriate fixtures for setup and teardown

## CI/CD Integration

These tests are designed to run in CI/CD pipelines. Make sure:
- All external dependencies (auth service, database) are available
- Environment variables are properly set
- Test data cleanup is handled properly
