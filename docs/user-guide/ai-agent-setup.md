# AI Agent Setup Guide

Connect ConHub to your favorite AI coding assistants using the Model Context Protocol (MCP).

## Supported AI Agents

ConHub works with any MCP-compatible AI agent:

✅ **GitHub Copilot** (with MCP extension)
✅ **Cursor**
✅ **Windsurf**
✅ **Cline** (formerly Claude Dev)
✅ **Continue**
✅ **Cody by Sourcegraph**
✅ **Any MCP-compatible tool**

## Prerequisites

- Active ConHub account with connected data sources
- At least one repository or document synced
- MCP-compatible AI agent installed
- Node.js 16+ (for MCP server)

## Quick Start

### 1. Get Your API Key

1. Log in to ConHub dashboard
2. Go to **Settings** → **API Keys**
3. Click **Create New Key**
4. Name it (e.g., "Cursor Integration")
5. Copy the key (starts with `chub_`)

⚠️ **Important:** Save your API key securely. You won't be able to see it again!

### 2. Install ConHub MCP Server

```bash
# Install globally
npm install -g @conhub/mcp-server

# Or use npx (no install needed)
npx @conhub/mcp-server
```

### 3. Configure Your AI Agent

Choose your AI agent and follow the specific instructions below.

## Cursor Setup

### Step 1: Open Cursor Settings

1. Open Cursor
2. Press `Cmd/Ctrl + Shift + P`
3. Type "Preferences: Open User Settings (JSON)"

### Step 2: Add MCP Configuration

Add to your `settings.json`:

```json
{
  "mcp.servers": {
    "conhub": {
      "command": "npx",
      "args": ["@conhub/mcp-server"],
      "env": {
        "CONHUB_API_KEY": "chub_your_api_key_here",
        "CONHUB_API_URL": "https://api.conhub.ai"
      }
    }
  }
}
```

### Step 3: Restart Cursor

1. Close Cursor completely
2. Reopen Cursor
3. You should see "ConHub connected" in the status bar

### Step 4: Test the Connection

Open a file and ask Cursor:

```
"@conhub How does authentication work in our backend?"
```

Cursor will now search your connected repositories!

## Windsurf Setup

### Configuration File

Windsurf uses a `.windsurf/mcp.json` configuration file:

```json
{
  "mcpServers": {
    "conhub": {
      "command": "node",
      "args": [
        "/absolute/path/to/mcp/server/build/index.js"
      ],
      "env": {
        "CONHUB_API_KEY": "chub_your_api_key_here"
      }
    }
  }
}
```

**Finding the MCP server path:**

```bash
# If installed globally
which @conhub/mcp-server

# Or use npx
npx which @conhub/mcp-server
```

### Using ConHub in Windsurf

Mention `@conhub` in your prompts:

```
"@conhub Find all API endpoints related to user authentication"
"@conhub Show me how we handle database migrations"
"@conhub What did the team discuss about the new feature?"
```

## Cline Setup

### Step 1: Open Cline Settings

1. Open VS Code
2. Install Cline extension if not already installed
3. Open Cline settings: `Cmd/Ctrl + Shift + P` → "Cline: Open Settings"

### Step 2: Add MCP Server

In Cline settings:

```json
{
  "cline.mcpServers": [
    {
      "name": "ConHub",
      "command": "npx",
      "args": ["@conhub/mcp-server"],
      "env": {
        "CONHUB_API_KEY": "chub_your_api_key_here"
      }
    }
  ]
}
```

### Step 3: Enable Context Provider

In Cline chat, use the context provider:

```
@conhub search for error handling patterns
```

## Continue Setup

### Configuration File Location

Continue stores config in:
- **macOS:** `~/.continue/config.json`
- **Windows:** `%USERPROFILE%\.continue\config.json`
- **Linux:** `~/.continue/config.json`

### Add ConHub to Config

Edit `config.json`:

```json
{
  "contextProviders": [
    {
      "name": "conhub",
      "params": {
        "type": "mcp",
        "serverCommand": "npx",
        "serverArgs": ["@conhub/mcp-server"],
        "serverEnv": {
          "CONHUB_API_KEY": "chub_your_api_key_here"
        }
      }
    }
  ]
}
```

### Usage

Use the `@conhub` context provider:

```
@conhub What are the main components of the auth service?
```

## GitHub Copilot Setup (Beta)

GitHub Copilot support for MCP is in beta. Follow these steps:

### Install MCP Extension

```bash
# Install the Copilot MCP bridge
npm install -g @github/copilot-mcp-bridge
```

### Configure Copilot

Add to your workspace `.vscode/settings.json`:

```json
{
  "github.copilot.advanced": {
    "mcp": {
      "enabled": true,
      "servers": {
        "conhub": {
          "command": "npx",
          "args": ["@conhub/mcp-server"],
          "env": {
            "CONHUB_API_KEY": "chub_your_api_key_here"
          }
        }
      }
    }
  }
}
```

## Advanced Configuration

### Multiple Environments

Separate keys for development and production:

```json
{
  "mcp.servers": {
    "conhub-dev": {
      "command": "npx",
      "args": ["@conhub/mcp-server"],
      "env": {
        "CONHUB_API_KEY": "chub_dev_key",
        "CONHUB_API_URL": "https://dev-api.conhub.ai"
      }
    },
    "conhub-prod": {
      "command": "npx",
      "args": ["@conhub/mcp-server"],
      "env": {
        "CONHUB_API_KEY": "chub_prod_key",
        "CONHUB_API_URL": "https://api.conhub.ai"
      }
    }
  }
}
```

### Custom Search Scope

Limit search to specific sources:

```json
{
  "env": {
    "CONHUB_API_KEY": "chub_your_key",
    "CONHUB_SEARCH_SOURCES": "github:company/backend,github:company/frontend",
    "CONHUB_MAX_RESULTS": "10"
  }
}
```

### Logging and Debugging

Enable debug logs:

```json
{
  "env": {
    "CONHUB_API_KEY": "chub_your_key",
    "CONHUB_LOG_LEVEL": "debug",
    "CONHUB_LOG_FILE": "/tmp/conhub-mcp.log"
  }
}
```

View logs:

```bash
tail -f /tmp/conhub-mcp.log
```

## Available Commands

Once configured, you can use these commands with your AI agent:

### Search

```
@conhub search [query]
```

Search across all your connected repositories and documents.

**Examples:**
- `@conhub search error handling in API routes`
- `@conhub search database migration scripts`
- `@conhub search how we implement authentication`

### Get File

```
@conhub get [file_path]
```

Retrieve specific file content.

**Examples:**
- `@conhub get src/auth/login.rs`
- `@conhub get backend/README.md`

### List Sources

```
@conhub sources
```

List all connected data sources.

### Search by Source

```
@conhub search in [source] [query]
```

Search within a specific repository or document.

**Examples:**
- `@conhub search in company/backend error handling`
- `@conhub search in google-drive project requirements`

### Get Recent

```
@conhub recent
```

Get recently modified documents.

### Semantic Search

```
@conhub semantic [query]
```

Use semantic search for conceptual queries.

**Examples:**
- `@conhub semantic patterns for handling user input validation`
- `@conhub semantic how to optimize database queries`

## Usage Patterns

### Code Review

```
AI: "Review this PR and check against our existing patterns"

You: "@conhub search similar implementations of user registration"
AI: [Analyzes similar code from your repos and provides feedback]
```

### Bug Fixing

```
You: "This authentication flow is failing"

AI: "@conhub search authentication flow implementation"
AI: [Finds related code and identifies the issue]
```

### Feature Development

```
You: "Implement a new API endpoint for user profiles"

AI: "@conhub search existing API endpoints"
AI: [Shows patterns and generates code following your conventions]
```

### Documentation

```
You: "Document this function"

AI: "@conhub search similar documented functions"
AI: [Generates documentation in your team's style]
```

## Troubleshooting

### Connection Failed

**Error:** "ConHub MCP server not responding"

**Solutions:**
1. Check API key is correct
2. Verify ConHub service is running: https://status.conhub.ai
3. Check network connectivity
4. Review MCP server logs

### Authentication Error

**Error:** "Invalid API key"

**Solutions:**
1. Regenerate API key in ConHub dashboard
2. Update configuration with new key
3. Restart AI agent

### No Results

**Error:** "No documents found"

**Solutions:**
1. Verify data sources are connected
2. Check sync status in ConHub dashboard
3. Ensure sources have finished indexing
4. Try more general search terms

### Slow Response

**Symptoms:** Queries take >10 seconds

**Solutions:**
1. Reduce `CONHUB_MAX_RESULTS` in config
2. Be more specific in queries
3. Use source filtering to limit search scope
4. Check your internet connection

### MCP Server Crashes

**Error:** MCP server exits unexpectedly

**Solutions:**
1. Enable debug logging
2. Check for Node.js version compatibility (need 16+)
3. Update MCP server: `npm update -g @conhub/mcp-server`
4. Report issue with logs to support

## Performance Optimization

### Caching

MCP server caches results for 5 minutes by default. Configure:

```json
{
  "env": {
    "CONHUB_CACHE_TTL": "300",  // 5 minutes
    "CONHUB_CACHE_SIZE": "100"  // Max cached queries
  }
}
```

### Batch Requests

The MCP server batches multiple requests:

```json
{
  "env": {
    "CONHUB_BATCH_SIZE": "5",
    "CONHUB_BATCH_DELAY_MS": "100"
  }
}
```

### Request Timeout

Adjust timeout for slow connections:

```json
{
  "env": {
    "CONHUB_TIMEOUT_MS": "10000"  // 10 seconds
  }
}
```

## Security Best Practices

1. **Never commit API keys** to version control
2. **Use environment variables** for API keys
3. **Rotate keys regularly** (every 90 days)
4. **Use separate keys** for each team member
5. **Monitor API usage** in ConHub dashboard
6. **Revoke compromised keys** immediately

## FAQ

**Q: Can multiple team members use the same API key?**

A: Not recommended. Create individual keys for better security and usage tracking.

**Q: Does the AI agent send my code to ConHub?**

A: No. Only search queries are sent. Your code stays in your editor.

**Q: How much does MCP access cost?**

A: MCP queries count toward your API quota. Check pricing page for details.

**Q: Can I use ConHub offline?**

A: No, ConHub requires internet connection to search your indexed data.

**Q: Which AI agent is best for ConHub?**

A: All MCP-compatible agents work well. Choice depends on your editor preference.

## Next Steps

- [Search Query Syntax](./search-syntax.md)
- [MCP Command Reference](../api/mcp-commands.md)
- [Optimizing Search Results](./search-optimization.md)

## Support

Having trouble?

- **Documentation:** https://docs.conhub.ai
- **Discord:** https://discord.gg/conhub
- **Email:** support@conhub.ai
