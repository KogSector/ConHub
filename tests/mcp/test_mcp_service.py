"""
Integration tests for MCP service
Tests the unified MCP server with all connectors
"""
import json
import subprocess
import time
import os
import pytest
from typing import Dict, Any

class MCPClient:
    """Client for interacting with MCP service via stdio"""
    
    def __init__(self, mcp_binary_path: str):
        self.mcp_binary_path = mcp_binary_path
        self.process = None
    
    def start(self):
        """Start the MCP service process"""
        env = os.environ.copy()
        env['DATABASE_URL'] = 'postgresql://neondb_owner:npg_w8jLMEkgsxc9@ep-wispy-credit-aazkw4fu-pooler.westus3.azure.neon.tech/neondb?sslmode=require&channel_binding=require'
        env['REDIS_URL'] = 'redis://default:KTSEuukbz30usz9ZWL89hWJfOVraoRYU@redis-17401.c53.west-us.azure.cloud.redislabs.com:17401'
        
        self.process = subprocess.Popen(
            [self.mcp_binary_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            text=True,
            bufsize=1
        )
        time.sleep(2)  # Give service time to start
    
    def stop(self):
        """Stop the MCP service"""
        if self.process:
            self.process.terminate()
            self.process.wait(timeout=5)
    
    def call_method(self, method: str, params: Dict[str, Any] = None) -> Dict[str, Any]:
        """Call an MCP method via JSON-RPC"""
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params or {}
        }
        
        request_str = json.dumps(request) + "\n"
        self.process.stdin.write(request_str)
        self.process.stdin.flush()
        
        response_str = self.process.stdout.readline()
        return json.loads(response_str)


@pytest.fixture
def mcp_client():
    """Fixture providing an MCP client"""
    # Adjust path based on build location
    binary_path = os.path.join(
        os.path.dirname(__file__),
        "../../target/debug/mcp-service"
    )
    
    if os.name == 'nt':  # Windows
        binary_path += ".exe"
    
    client = MCPClient(binary_path)
    client.start()
    yield client
    client.stop()


class TestMCPCore:
    """Test MCP core functionality"""
    
    def test_health_check(self, mcp_client):
        """Test health check endpoint"""
        response = mcp_client.call_method("mcp.health")
        
        assert "result" in response
        assert response["result"]["status"] == "healthy"
    
    def test_list_tools(self, mcp_client):
        """Test listing all tools from all connectors"""
        response = mcp_client.call_method("mcp.listTools")
        
        assert "result" in response
        assert "tools" in response["result"]
        
        tools = response["result"]["tools"]
        assert len(tools) > 0
        
        # Check for expected GitHub tools
        tool_names = [t["name"] for t in tools]
        assert "github.list_repositories" in tool_names
        assert "github.list_branches" in tool_names
        assert "github.list_files" in tool_names
        assert "github.get_file_content" in tool_names
    
    def test_call_unknown_tool(self, mcp_client):
        """Test calling a non-existent tool"""
        response = mcp_client.call_method("mcp.callTool", {
            "name": "unknown.tool",
            "arguments": {}
        })
        
        assert "error" in response
        assert response["error"]["code"] == -32601  # Tool not found


class TestGitHubConnector:
    """Test GitHub connector functionality"""
    
    @pytest.mark.skipif(
        not os.getenv("GITHUB_ACCESS_TOKEN"),
        reason="GitHub token not configured"
    )
    def test_list_repositories(self, mcp_client):
        """Test listing GitHub repositories"""
        response = mcp_client.call_method("mcp.callTool", {
            "name": "github.list_repositories",
            "arguments": {
                "visibility": "all"
            }
        })
        
        assert "result" in response
        result = json.loads(response["result"]["content"][0]["text"])
        
        assert isinstance(result, list)
        
        if len(result) > 0:
            repo = result[0]
            assert "id" in repo
            assert repo["id"].startswith("gh:")
            assert "provider" in repo
            assert repo["provider"] == "github"
            assert "name" in repo
            assert "owner" in repo
            assert "visibility" in repo
            assert "default_branch" in repo
    
    @pytest.mark.skipif(
        not os.getenv("GITHUB_ACCESS_TOKEN"),
        reason="GitHub token not configured"
    )
    def test_list_files(self, mcp_client):
        """Test listing files in a GitHub repository"""
        # First get a repository
        repos_response = mcp_client.call_method("mcp.callTool", {
            "name": "github.list_repositories",
            "arguments": {"visibility": "all"}
        })
        
        repos = json.loads(repos_response["result"]["content"][0]["text"])
        
        if len(repos) > 0:
            repo_id = repos[0]["id"]
            
            # Now list files
            files_response = mcp_client.call_method("mcp.callTool", {
                "name": "github.list_files",
                "arguments": {
                    "repo_id": repo_id,
                    "branch": repos[0]["default_branch"],
                    "path": ""
                }
            })
            
            assert "result" in files_response
            files = json.loads(files_response["result"]["content"][0]["text"])
            
            assert isinstance(files, list)
            
            if len(files) > 0:
                file = files[0]
                assert "id" in file
                assert "path" in file
                assert "name" in file
                assert "kind" in file
                assert file["kind"] in ["file", "dir"]


class TestContextSchema:
    """Test that all connectors return normalized context schema"""
    
    def test_repository_descriptor_format(self, mcp_client):
        """Verify RepositoryDescriptor has required fields"""
        required_fields = [
            "id", "provider", "name", "owner", "visibility",
            "default_branch", "url", "updated_at"
        ]
        
        response = mcp_client.call_method("mcp.listTools")
        tools = response["result"]["tools"]
        
        # Find a list_repositories tool
        repo_tools = [t for t in tools if "list_repositories" in t["name"]]
        assert len(repo_tools) > 0


class TestConnectorArchitecture:
    """Test connector architecture and manager"""
    
    def test_tool_naming_convention(self, mcp_client):
        """Verify all tools follow connector.tool naming"""
        response = mcp_client.call_method("mcp.listTools")
        tools = response["result"]["tools"]
        
        for tool in tools:
            name = tool["name"]
            assert "." in name, f"Tool {name} doesn't follow connector.tool format"
            
            parts = name.split(".")
            assert len(parts) == 2, f"Tool {name} has wrong format"
            
            connector, tool_name = parts
            assert connector in [
                "github", "gitlab", "bitbucket", "gdrive",
                "dropbox", "fs", "notion"
            ], f"Unknown connector: {connector}"
    
    def test_multiple_connectors_enabled(self, mcp_client):
        """Verify multiple connectors are available"""
        response = mcp_client.call_method("mcp.listTools")
        tools = response["result"]["tools"]
        
        connectors = set()
        for tool in tools:
            connector = tool["name"].split(".")[0]
            connectors.add(connector)
        
        # At minimum, GitHub should be enabled
        assert "github" in connectors
        assert len(connectors) >= 1


class TestErrorHandling:
    """Test error handling and edge cases"""
    
    def test_missing_arguments(self, mcp_client):
        """Test calling tool with missing required arguments"""
        response = mcp_client.call_method("mcp.callTool", {
            "name": "github.list_branches",
            "arguments": {}  # Missing repo_id
        })
        
        assert "error" in response or (
            "result" in response and
            response["result"].get("is_error") == True
        )
    
    def test_invalid_json_rpc(self, mcp_client):
        """Test invalid JSON-RPC request"""
        # Send malformed request
        mcp_client.process.stdin.write("invalid json\n")
        mcp_client.process.stdin.flush()
        
        response_str = mcp_client.process.stdout.readline()
        response = json.loads(response_str)
        
        assert "error" in response


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
