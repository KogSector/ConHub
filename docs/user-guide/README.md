# ConHub User Guide

Welcome to the ConHub user documentation! This guide will help you get the most out of ConHub's AI-powered knowledge layer.

## Quick Links

### Getting Started
- **[Getting Started Guide](./getting-started.md)** - Your first steps with ConHub
- **[Connecting Repositories](./connecting-repositories.md)** - Connect GitHub, GitLab, Bitbucket
- **[AI Agent Setup](./ai-agent-setup.md)** - Integrate with Cursor, Windsurf, Cline, etc.

### Core Concepts
- **[Fusion Embeddings](./fusion-embeddings.md)** - Understanding our multi-model embedding system
- **[Security and Privacy](./security-and-privacy.md)** - Zero-trust architecture and data protection

### Advanced Topics
- **[Custom Model Configuration](./custom-models.md)** - Configure embedding models
- **[Webhook Integration](./webhooks.md)** - Real-time sync with webhooks
- **[API Reference](../api/README.md)** - Complete API documentation

## What is ConHub?

ConHub is an AI-powered knowledge layer that:

1. **Connects** your data sources (repos, docs, chat)
2. **Embeds** content using state-of-the-art AI models
3. **Indexes** for fast semantic search
4. **Serves** context to AI agents via MCP

```
┌─────────────┐
│ Your Data   │ GitHub, GitLab, Google Drive, Slack, etc.
└──────┬──────┘
       │
       ↓
┌─────────────┐
│   ConHub    │ Embedding + Indexing
└──────┬──────┘
       │
       ↓
┌─────────────┐
│ AI Agents   │ Cursor, Cline, Copilot, Windsurf, etc.
└─────────────┘
```

## Key Features

### 🔗 Universal Connectivity
Connect any data source:
- **Code:** GitHub, GitLab, Bitbucket
- **Docs:** Google Drive, Dropbox, OneDrive
- **Chat:** Slack, Microsoft Teams
- **Web:** URL scraper for documentation sites
- **Local:** Direct file uploads

### 🧠 Fusion Embeddings
Multiple AI models combined for superior accuracy:
- **Voyage-Code:** Best for programming code
- **Qwen:** Multilingual and code understanding
- **OpenAI:** General-purpose semantic understanding
- **Cohere:** Conversational content
- **JinaAI:** Long-context documents

### 🔐 Zero-Trust Security
Granular access control:
- Explicit authorization required
- Read-only by default
- Time-bound access
- Conditional policies
- Complete audit logs

### 🚀 AI Agent Integration
Works with all MCP-compatible tools:
- GitHub Copilot
- Cursor
- Windsurf
- Cline
- Continue
- Any MCP tool

## Quick Start

### 1. Sign Up

```bash
# Visit ConHub
https://app.conhub.ai/signup

# Or use CLI
npx @conhub/cli auth login
```

### 2. Connect a Data Source

```bash
# Via dashboard
1. Go to Connections
2. Click "Connect GitHub"
3. Authorize ConHub
4. Select repositories

# Or use CLI
npx @conhub/cli connect github
```

### 3. Wait for Sync

```bash
# Check sync status
npx @conhub/cli sync status

# Monitor in dashboard
https://app.conhub.ai/dashboard/sources
```

### 4. Connect AI Agent

```bash
# Install MCP server
npm install -g @conhub/mcp-server

# Configure your AI agent
# See AI Agent Setup guide for details
```

### 5. Start Using!

```
@conhub How does authentication work in our backend?
@conhub search database migration patterns
@conhub get src/auth/login.rs
```

## Common Workflows

### Code Review

```mermaid
graph LR
    A[Create PR] --> B[AI Reviews]
    B --> C[@conhub Check Patterns]
    C --> D[AI Suggests Improvements]
```

**Usage:**
```
You: "Review this authentication implementation"
AI: "@conhub search authentication patterns"
AI: [Provides feedback based on your codebase]
```

### Bug Fixing

```mermaid
graph LR
    A[Bug Report] --> B[Search Similar Code]
    B --> C[@conhub Find Related]
    C --> D[Identify Issue]
    D --> E[Suggest Fix]
```

**Usage:**
```
You: "This login flow is broken"
AI: "@conhub search login implementation"
AI: [Finds similar code and identifies the bug]
```

### Feature Development

```mermaid
graph LR
    A[Feature Request] --> B[Research Existing]
    B --> C[@conhub Search Patterns]
    C --> D[Generate Code]
    D --> E[Follow Conventions]
```

**Usage:**
```
You: "Add user profile endpoint"
AI: "@conhub search API endpoint patterns"
AI: [Generates code matching your style]
```

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                          ConHub Platform                         │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   Auth      │  │    Data     │  │  Embedding  │            │
│  │  Service    │  │   Service   │  │   Service   │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
│         │                 │                 │                    │
│  ┌──────▼─────────────────▼─────────────────▼──────────┐       │
│  │              Microservices Layer                      │       │
│  └───────────────────────────────────────────────────────┘       │
│         │                 │                 │                    │
│  ┌──────▼─────────────────▼─────────────────▼──────────┐       │
│  │   PostgreSQL    │    Qdrant     │     Redis         │       │
│  │   (Metadata)    │   (Vectors)   │    (Cache)        │       │
│  └───────────────────────────────────────────────────────┘       │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

## Support & Community

### Documentation
- **User Guide:** You're here!
- **API Docs:** [docs.conhub.ai/api](https://docs.conhub.ai/api)
- **Examples:** [github.com/conhub/examples](https://github.com/conhub/examples)

### Community
- **Discord:** [discord.gg/conhub](https://discord.gg/conhub)
- **GitHub:** [github.com/conhub/conhub](https://github.com/conhub/conhub)
- **Twitter:** [@conhub_ai](https://twitter.com/conhub_ai)

### Support
- **Email:** support@conhub.ai
- **Status:** [status.conhub.ai](https://status.conhub.ai)
- **Security:** security@conhub.ai

## Feedback

We love feedback! Help us improve ConHub:

- **Feature Requests:** [github.com/conhub/conhub/issues](https://github.com/conhub/conhub/issues)
- **Bug Reports:** [github.com/conhub/conhub/issues](https://github.com/conhub/conhub/issues)
- **Surveys:** [conhub.ai/feedback](https://conhub.ai/feedback)

## Contributing

Want to contribute?

- **Documentation:** Submit PRs to improve docs
- **Connectors:** Build new data source connectors
- **MCP Tools:** Create MCP extensions
- **Examples:** Share usage patterns

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

ConHub documentation is licensed under CC BY 4.0.

The ConHub platform uses a mixed license:
- **Open Source Components:** MIT License
- **Proprietary Components:** Commercial License

See [LICENSE.md](../../LICENSE.md) for details.

---

**Ready to get started?** Head to [Getting Started](./getting-started.md) →
