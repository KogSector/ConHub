# Getting Started with ConHub

Welcome to ConHub! This guide will help you get started with our AI-powered knowledge layer and context engine.

## Table of Contents

1. [What is ConHub?](#what-is-conhub)
2. [Creating Your Account](#creating-your-account)
3. [Connecting Data Sources](#connecting-data-sources)
4. [Understanding the Dashboard](#understanding-the-dashboard)
5. [Embedding and Indexing](#embedding-and-indexing)
6. [Using with AI Agents](#using-with-ai-agents)

## What is ConHub?

ConHub is a powerful knowledge layer that connects your data sources (repositories, documents, chat channels) and makes them accessible to AI agents. Our platform:

- **Embeds and indexes** your data for AI understanding
- **Connects multiple data sources** (GitHub, GitLab, Google Drive, Slack, and more)
- **Works with any AI agent** via MCP (Model Context Protocol)
- **Implements zero-trust security** for granular access control
- **Uses fusion embeddings** for superior accuracy

## Creating Your Account

1. Navigate to [ConHub](https://app.conhub.ai)
2. Click **Sign Up** in the top-right corner
3. Enter your email and create a password
4. Verify your email address
5. Complete your profile setup

## Connecting Data Sources

ConHub supports multiple data source types. Here's how to connect them:

### Step 1: Navigate to Connections

1. Log in to your ConHub dashboard
2. Click on **Connections** in the sidebar
3. You'll see available platforms to connect

### Step 2: Choose a Platform

ConHub supports:

- **GitHub** - Connect repositories for code context
- **GitLab** - Access your GitLab projects
- **Google Drive** - Index documents and files
- **Dropbox** - Sync cloud storage
- **Slack** - Access channel conversations
- **Bitbucket** - Connect Bitbucket repositories
- **Local Files** - Upload files directly
- **URL Scraper** - Index web content

### Step 3: Authorize Access

1. Click **Connect** on your chosen platform
2. A popup window will open for OAuth authorization
3. **Important:** ConHub requests **read-only access** by default
4. Review the permissions and click **Authorize**
5. You'll be redirected back to ConHub

### Step 4: Select Resources

After connecting:

1. Navigate to **Data Sources** in your dashboard
2. Select which repositories, folders, or channels to index
3. **Zero-Trust Model:** You explicitly choose what gets indexed
4. Click **Start Sync** to begin embedding

## Understanding the Dashboard

Your ConHub dashboard provides several key sections:

### Sources Tab
- View all connected data sources
- See sync status and last updated time
- Start manual syncs or configure automatic syncing

### Documents Tab
- Browse all indexed documents
- Search across your knowledge base
- View embedding metadata

### Billing Tab
- Manage your subscription
- View usage statistics
- Upgrade or downgrade plans

### Connections Tab
- Manage platform connections
- Reconnect or disconnect services
- View connection health

## Embedding and Indexing

ConHub uses a **fusion embedding system** that intelligently combines multiple AI models:

### How It Works

1. **Automatic Detection:** ConHub detects your data source type
2. **Model Selection:** The best embedding models are chosen automatically
   - Code repositories use Voyage-Code + Qwen
   - Documents use OpenAI + Cohere
   - Chat messages use specialized conversational models
3. **Fusion:** Multiple embeddings are combined for superior accuracy
4. **Storage:** Embeddings are stored in our vector database (Qdrant)

### What Gets Embedded?

- **Code Files:** All text-based source code (`.rs`, `.py`, `.js`, etc.)
- **Documents:** Text files, PDFs, Google Docs, Markdown files
- **Chat Messages:** Slack conversations and channel history
- **Web Content:** Scraped and parsed HTML from URLs

### What Doesn't Get Embedded?

- Binary files (images, videos, executables)
- Files larger than 10MB (configurable)
- Files you haven't explicitly authorized

## Using with AI Agents

Once your data is indexed, connect it to your favorite AI coding assistant:

### Supported AI Agents

- **GitHub Copilot** (via MCP)
- **Cursor** (via MCP)
- **Windsurf** (via MCP)
- **Cline** (via MCP)
- **Any MCP-compatible tool**

### Setting Up MCP Connection

1. Navigate to **Settings** → **AI Agents**
2. Copy your MCP configuration:

```json
{
  "mcpServers": {
    "conhub": {
      "command": "node",
      "args": ["/path/to/conhub/mcp/build/index.js"],
      "env": {
        "CONHUB_API_KEY": "your-api-key",
        "CONHUB_API_URL": "https://api.conhub.ai"
      }
    }
  }
}
```

3. Add this to your AI agent's MCP settings
4. Restart your AI agent
5. ConHub context is now available!

### Using ConHub in Your IDE

Once connected, your AI agent can:

- Access code from all connected repositories
- Search through documents
- Reference Slack conversations
- Provide context-aware suggestions

Simply ask your AI agent questions like:
- "How does the authentication work in our backend?"
- "Show me similar implementations across our repositories"
- "What did the team discuss about this feature in Slack?"

## Best Practices

### Security

- **Read-Only Access:** Only grant read permissions when connecting
- **Selective Indexing:** Only index what you need
- **Regular Audits:** Review connected sources periodically
- **API Key Rotation:** Rotate keys every 90 days

### Performance

- **Incremental Sync:** Use automatic syncing for updates
- **Exclude Build Artifacts:** Don't index `node_modules`, `target/`, etc.
- **Filter by Language:** Only sync relevant file types
- **Organize Sources:** Group related repositories

### Cost Optimization

- Start with the Free tier for testing
- Monitor your usage dashboard
- Use incremental syncing to reduce API calls
- Archive old sources you don't need

## Getting Help

Need assistance?

- **Documentation:** [docs.conhub.ai](https://docs.conhub.ai)
- **Support Email:** support@conhub.ai
- **Discord Community:** [discord.gg/conhub](https://discord.gg/conhub)
- **GitHub Issues:** [github.com/conhub/issues](https://github.com/conhub/issues)

## Next Steps

- [Connect Your First Repository](./connecting-repositories.md)
- [Configure AI Agent Integration](./ai-agent-setup.md)
- [Understanding Fusion Embeddings](./fusion-embeddings.md)
- [Security and Privacy](./security-and-privacy.md)
