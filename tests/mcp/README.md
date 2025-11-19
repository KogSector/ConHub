# MCP Service Tests

Integration and unit tests for the unified MCP (Model Context Protocol) service.

## Setup

```bash
# Install Python dependencies
pip install pytest requests

# Set environment variables
export GITHUB_ACCESS_TOKEN=your_token_here
export DATABASE_URL=postgresql://...
export REDIS_URL=redis://...
```

## Running Tests

```bash
# Run all MCP tests
pytest tests/mcp/ -v

# Run specific test file
pytest tests/mcp/test_mcp_service.py -v

# Run specific test
pytest tests/mcp/test_mcp_service.py::TestMCPCore::test_health_check -v

# Skip tests that require GitHub token
pytest tests/mcp/ -v -m "not github"
```

## Test Structure

- `test_mcp_service.py` - Integration tests for MCP service functionality
- `test_context_schema.py` - Tests for unified context schema validation
- `test_connectors.py` - Individual connector tests (when implemented)

## What's Tested

### Core MCP Protocol
- JSON-RPC request/response handling
- Tool listing (`mcp.listTools`)
- Tool calling (`mcp.callTool`)
- Resource listing and reading
- Health checks
- Error handling

### GitHub Connector
- Repository listing
- Branch listing
- File tree navigation
- File content retrieval
- Token authentication
- Rate limiting

### Context Schema
- RepositoryDescriptor normalization
- FileDescriptor normalization
- BranchDescriptor normalization
- Token-efficient field naming
- Consistent structure across providers

### Architecture
- Connector manager routing
- Tool naming conventions (connector.tool)
- Multi-connector support
- Feature flags

## Notes

- Most tests require a running database and Redis instance
- GitHub tests require a valid `GITHUB_ACCESS_TOKEN`
- Tests use stdio communication with the MCP service
- Build the MCP service before running tests: `cargo build --manifest-path mcp/Cargo.toml`
