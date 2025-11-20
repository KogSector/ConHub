# ConHub Quick Start - AI Agent Connection

## Goal
Connect your local ConHub MCP server (from this repository) to an AI agent like Cursor, Windsurf, or Cline so it can use your ConHub database and file system.

> For the full, product-level MCP integration using the published `@conhub/mcp-server` package, see the [AI Agent Setup Guide](./ai-agent-setup.md).

## 5-Minute Local Setup

### Step 1: Build the MCP Server
```powershell
cd c:\Users\risha\Desktop\Work\ConHub\mcp
cargo build --release
```
**Result:** Binary at `target\release\mcp-service.exe`.

### Step 2: Configure the MCP Server
Create `mcp\.env`:
```env
DATABASE_URL=postgresql://conhub:conhub_password@localhost:5432/conhub
FS_ROOT_PATHS=c:\Users\risha\Desktop\Work\ConHub
ENABLE_FS=true
RUST_LOG=info
```

**OR** if using Neon DB:
```env
DATABASE_URL_NEON=postgresql://neondb_owner:npg_w8jLMEkgsxc9@ep-wispy-credit-aazkw4fu-pooler.westus3.azure.neon.tech/neondb?sslmode=require&channel_binding=require
FS_ROOT_PATHS=c:\Users\risha\Desktop\Work\ConHub
ENABLE_FS=true
RUST_LOG=info
```

### Step 3: Point Your AI Agent to the Local MCP Server

Use the configuration patterns from the [AI Agent Setup Guide](./ai-agent-setup.md), but instead of running the npm package (`npx @conhub/mcp-server`), point the agent directly at your local binary:

```json
{
  "command": "c:/Users/risha/Desktop/Work/ConHub/mcp/target/release/mcp-service.exe",
  "args": [],
  "cwd": "c:/Users/risha/Desktop/Work/ConHub/mcp",
  "env": {
    "DATABASE_URL": "postgresql://conhub:conhub_password@localhost:5432/conhub",
    "FS_ROOT_PATHS": "c:/Users/risha/Desktop/Work/ConHub",
    "ENABLE_FS": "true",
    "RUST_LOG": "info"
  }
}
```

Apply this pattern in your agent’s MCP configuration (Cursor, Cline, Windsurf, etc.) wherever the guide shows a `command` of `npx` with `@conhub/mcp-server`.

### Step 4: Restart Your IDE
Close and reopen Cursor/VS Code/Windsurf for changes to take effect.

### Step 5: Test It
Ask your AI agent:
- "What files are in the ConHub project?"
- "Read the README.md file"
- "Search for 'MCP' in the codebase"

You should see the agent using tools like:
- `fs.list_files`
- `fs.read_file`
- `fs.search_files`

## Success Indicators

You'll know it's working when:
1. IDE starts without errors.
2. Agent responds to file-related questions.
3. Agent mentions using MCP tools in its response.
4. Console shows MCP server logs (if you run it separately).

## Troubleshooting

**"Command not found"**
- Check the `command` path is correct.
- Verify the `.exe` file exists after `cargo build --release`.

**"Database connection failed"**
- Ensure PostgreSQL is running (or use Neon DB URL).
- Check `DATABASE_URL` / `DATABASE_URL_NEON` is correct.
- Run migrations: `cd database && sqlx migrate run`.

**"No tools available"**
- Check `.env` file exists in `mcp/` directory.
- Verify `ENABLE_FS=true` is set.
- Check logs with `RUST_LOG=debug`.

**"Agent doesn't respond to file questions"**
- Restart IDE after configuration changes.
- Check MCP server is in the agent's tools list.
- Try asking explicitly: "Use the fs.list_files tool to list files".

## Next Steps

### Enable More Connectors
In `mcp\.env`:
```env
ENABLE_GITHUB=true
GITHUB_TOKEN=ghp_your_token_here

ENABLE_GITLAB=true
GITLAB_TOKEN=glpat_your_token_here

ENABLE_NOTION=true
NOTION_TOKEN=secret_your_token_here
```

Then restart your IDE.

### Full Local Development
See `LOCAL_DEV_SETUP.md` for running all services.

### Understanding the System
See `IMPLEMENTATION_SUMMARY.md` for architecture overview.

## Documentation

- `mcp/README.md` - Complete MCP server docs.
- `mcp/mcp-client-config.example.json` - More configuration examples.
- `LOCAL_DEV_SETUP.md` - Full local development guide.
- `IMPLEMENTATION_SUMMARY.md` - What we built and why.

## Pro Tips

1. **Multiple Projects:** Add multiple paths in `FS_ROOT_PATHS`:
   ```env
   FS_ROOT_PATHS=c:\Projects\ConHub,c:\Projects\OtherProject,c:\Documents
   ```

2. **Debug Mode:** See what's happening:
   ```env
   RUST_LOG=debug
   ```

3. **GitHub Integration:** After enabling the GitHub connector, ask:
   - "List my GitHub repositories"
   - "Show me recent commits in [repo]"
   - "Search for [term] in my GitHub code"

4. **Context Awareness:** The agent now has access to:
   - All files in configured paths.
   - Your GitHub repositories (if configured).
   - Your GitLab projects (if configured).
   - Your Notion workspace (if configured).
   - All indexed documents in ConHub.

Enjoy your AI agent with full ConHub context!
