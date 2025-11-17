# Connecting Repositories to ConHub

This guide explains how to connect code repositories from GitHub, GitLab, and Bitbucket to ConHub.

## Prerequisites

- Active ConHub account
- Repository access (read permissions minimum)
- Understanding of OAuth authorization

## Supported Repository Platforms

### GitHub
- Public and private repositories
- Organization repositories (with proper permissions)
- GitHub Enterprise (contact us for setup)

### GitLab
- GitLab.com repositories
- Self-hosted GitLab instances
- Group projects

### Bitbucket
- Bitbucket Cloud
- Bitbucket Data Center (enterprise)

## Connecting GitHub

### Step 1: Navigate to Connections

1. Go to your ConHub dashboard
2. Click **Connections** in the sidebar
3. Find **GitHub** in the available platforms list

### Step 2: Authorize ConHub

1. Click the **Connect** button for GitHub
2. A popup window opens with GitHub's authorization page
3. ConHub requests these permissions:
   - `repo` (read): Access to repository code
   - `read:user`: Basic profile information
4. Click **Authorize ConHub**

### Step 3: Verify Access

Before you can connect a repository, ConHub validates that you have **read access**:

```
✓ Checking repository permissions...
✓ User has read access
✓ Repository is accessible
```

If validation fails, ensure:
- You have at least **Read** permissions on the repository
- The repository exists and is not deleted
- Your GitHub token hasn't expired

### Step 4: Select Repositories

1. After authorization, go to **Data Sources** → **Add Source**
2. Choose **GitHub Repository**
3. Enter the repository URL (e.g., `https://github.com/user/repo`)
4. ConHub validates your access to this specific repository
5. Select which branches to sync (default: all branches)
6. Choose file filters (optional):
   - Include: `**/*.rs`, `**/*.py` (only Rust and Python files)
   - Exclude: `**/node_modules/**`, `**/target/**`

### Step 5: Start Sync

1. Click **Start Sync**
2. ConHub begins indexing your repository
3. Progress is shown in real-time:
   ```
   📦 Fetching repository structure...
   📄 Processing 1,234 files...
   🔄 Generating embeddings (45% complete)...
   ✅ Sync complete! 1,234 files indexed
   ```

## Connecting GitLab

### Step 1: Authorize GitLab

1. Go to **Connections** → **GitLab**
2. Click **Connect**
3. ConHub requests:
   - `read_user`: User information
   - `read_api`: API access
   - `read_repository`: Repository content

### Step 2: Select GitLab Instance

For self-hosted GitLab:

1. Enter your GitLab URL (e.g., `https://gitlab.company.com`)
2. Configure OAuth application in your GitLab instance:
   - Go to **User Settings** → **Applications**
   - Add a new application:
     - Name: `ConHub`
     - Redirect URI: `https://app.conhub.ai/auth/gitlab/callback`
     - Scopes: `read_user`, `read_api`, `read_repository`
   - Copy Application ID and Secret to ConHub settings

### Step 3: Access Validation

ConHub validates your access level:

- **Guest (10):** ❌ Insufficient access
- **Reporter (20):** ✅ Can read code
- **Developer (30):** ✅ Can read code
- **Maintainer (40):** ✅ Can read code
- **Owner (50):** ✅ Can read code

### Step 4: Configure Sync

Similar to GitHub, select:
- Specific projects
- Branches to sync
- File filters
- Sync frequency

## Connecting Bitbucket

### Step 1: OAuth Setup

1. Navigate to **Connections** → **Bitbucket**
2. Click **Connect**
3. Authorize ConHub with:
   - `repository`: Repository access
   - `account`: Account information

### Step 2: Repository Selection

1. Choose Bitbucket workspace
2. Select repositories to sync
3. Configure branch and file filters

## Advanced Configuration

### File Filters

Use glob patterns to include/exclude files:

**Include Patterns:**
```
**/*.rs          # All Rust files
**/*.py          # All Python files
src/**           # Everything in src/
docs/**/*.md     # Markdown files in docs/
```

**Exclude Patterns:**
```
**/node_modules/**    # No dependencies
**/target/**          # No build artifacts
**/.git/**            # No Git metadata
**/*.min.js           # No minified files
```

### Branch Selection

- **All branches:** Sync every branch
- **Main branches only:** Sync `main`, `master`, `develop`
- **Custom:** Specify branch patterns

### Sync Frequency

- **Manual:** Sync only when you trigger it
- **Hourly:** Check for updates every hour
- **Daily:** Once per day at scheduled time
- **On Push:** Via webhooks (requires webhook setup)

## Webhook Setup (Optional)

For real-time sync on push events:

### GitHub Webhooks

1. Go to repository **Settings** → **Webhooks**
2. Add webhook:
   - Payload URL: `https://api.conhub.ai/webhooks/github`
   - Content type: `application/json`
   - Secret: [Get from ConHub Settings]
   - Events: `push`, `pull_request`

### GitLab Webhooks

1. Project **Settings** → **Webhooks**
2. Configure:
   - URL: `https://api.conhub.ai/webhooks/gitlab`
   - Secret Token: [From ConHub]
   - Trigger: Push events

### Bitbucket Webhooks

1. Repository **Settings** → **Webhooks**
2. Add webhook:
   - URL: `https://api.conhub.ai/webhooks/bitbucket`
   - Status: Active
   - Triggers: Repository push

## Access Validation

ConHub uses **zero-trust security**. Before syncing any repository:

1. **Token Validation:** Checks if your access token is valid
2. **Permission Check:** Verifies you have read access
3. **Repository Access:** Confirms the repository exists and is accessible
4. **User Confirmation:** You must explicitly approve each repository

### What Happens If Validation Fails?

- ❌ **Invalid Token:** Reconnect your account
- ❌ **No Access:** Request access from repository owner
- ❌ **Repository Not Found:** Check URL and repository status
- ❌ **Rate Limited:** Wait for rate limit reset

## Managing Connected Repositories

### View Sync Status

Dashboard shows for each repository:
- Last sync time
- Number of files indexed
- Sync duration
- Any errors

### Manual Re-sync

1. Go to **Data Sources**
2. Find your repository
3. Click **Sync Now**

### Disconnect Repository

1. Go to **Data Sources**
2. Click repository settings
3. Click **Disconnect**
4. Confirm deletion of indexed data

## Troubleshooting

### "Access Denied" Error

**Cause:** Insufficient permissions

**Solution:**
1. Verify you have **Read** access to the repository
2. Check if your token has expired
3. Reconnect your account in **Connections**

### "Rate Limit Exceeded"

**Cause:** Too many API requests

**Solution:**
1. Wait for rate limit reset (shown in error message)
2. Reduce sync frequency
3. Upgrade to higher tier for more API quota

### "Large Repository Timeout"

**Cause:** Repository is too large to sync in one request

**Solution:**
1. Enable incremental sync
2. Use file filters to exclude unnecessary files
3. Contact support for large repository optimization

### Files Not Indexed

**Possible Causes:**
- File is in excluded pattern
- File is binary (not text)
- File exceeds size limit (default 10MB)
- File type not supported

**Check:**
1. Review file filters
2. Check file size
3. Verify file extension is supported

## Best Practices

1. **Start Small:** Connect one repository first, verify it works
2. **Use Filters:** Don't sync build artifacts or dependencies
3. **Monitor Usage:** Check sync duration and API usage
4. **Regular Updates:** Enable automatic syncing for active projects
5. **Archive Old Repos:** Disconnect repositories you no longer need

## Next Steps

- [Configure AI Agent Integration](./ai-agent-setup.md)
- [Understanding File Filters](./file-filters.md)
- [Managing Sync Jobs](./sync-management.md)
