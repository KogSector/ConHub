# GitHub OAuth Debugging Playbook

This document explains how to trace a GitHub OAuth token from creation to usage when debugging "Bad credentials" or other authentication errors.

## Quick Reference: Log Tags

| Service | Tag Pattern | Purpose |
|---------|-------------|---------|
| Auth | `[OAuth Exchange][{correlation_id}]` | Token exchange from GitHub |
| Auth | `[Internal Token][{correlation_id}]` | Token retrieval for data service |
| Auth | `[GitHub OAuth]` | Low-level GitHub API calls in OAuthService |
| Data | `[Repo Check][{correlation_id}]` | Repository access validation |
| Data | `[AuthClient][{correlation_id}]` | Auth service communication |
| Data | `[GITHUB]` | GitHub API calls in connector |

## Token Debug Format

All logs use a safe token debug format that never exposes the full token:

```
token_debug=len=40, prefix=gho_Xy..., sha256_prefix=a1b2c3d4e5f6
```

- **len**: Token length (GitHub OAuth tokens are typically 40 chars)
- **prefix**: First 6 characters
- **sha256_prefix**: First 12 chars of SHA256 hash (for matching across logs)

## Step-by-Step Debugging

### 1. Start Services with Debug Logging

```bash
# Terminal 1: Auth service
cd auth
RUST_LOG=info,conhub=debug cargo run

# Terminal 2: Data service
cd data
RUST_LOG=info,conhub=debug cargo run

# Terminal 3: Frontend
cd frontend
npm run dev
```

### 2. Connect GitHub (Fresh Connection)

1. Go to `http://localhost:3000/dashboard/connections`
2. Click **Connect** on GitHub
3. Complete OAuth flow in popup

**Watch auth-service logs for:**

```
[OAuth Exchange][abc12345] 🔄 Starting exchange: provider=github, user_id=..., code_prefix=...
[GitHub OAuth] Starting token exchange: client_id=Ov23li..., redirect_uri=...
[GitHub OAuth] ✅ Token exchange successful: token_type=bearer, scope=Some("repo read:user user:email"), token_debug=len=40, prefix=gho_Xy..., sha256_prefix=a1b2c3d4e5f6
[OAuth Exchange][abc12345] ✅ User info fetched: provider=github, platform_user_id=12345678, email=user@example.com
[OAuth Exchange][abc12345] 💾 Storing connection: connection_id=..., scope=repo read:user user:email, token_debug=...
[OAuth Exchange][abc12345] ✅✅ CONNECTION STORED SUCCESSFULLY: ...
```

**Key things to verify:**
- `scope` includes `repo` (required for private repos)
- `token_type` is `bearer`
- `token_debug` sha256_prefix matches what you'll see later

### 3. Check Repository Access

1. Go to Repositories page
2. Enter a GitHub repo URL
3. Click **Check**

**Watch data-service logs for:**

```
[Repo Check][def67890] 🔍 Starting: repo_url=https://github.com/owner/repo, provider=github
[Repo Check][def67890] ✅ User authenticated: user_id=...
[Repo Check][def67890] 🔑 Fetching OAuth token from auth service: user_id=..., provider=github
[AuthClient][def67890] 🔑 Fetching github token: user_id=..., url=http://localhost:3010/internal/oauth/github/token?user_id=...
```

**Watch auth-service logs for:**

```
[Internal Token][def67890] 🔍 Looking up token: provider=github, user_id=...
[Internal Token][def67890] ✅ FOUND VALID TOKEN: provider=github, connection_id=..., platform_user_id=..., scope=Some("repo read:user user:email"), token_debug=len=40, prefix=gho_Xy..., sha256_prefix=a1b2c3d4e5f6
```

**Back to data-service logs:**

```
[AuthClient][def67890] ✅ Got token from auth service: provider=github, token_len=40, token_prefix=gho_Xy...
[Repo Check][def67890] ✅ Got OAuth token from auth service: provider=github, token_debug=...
[Repo Check][def67890] 🌐 Calling GitHub API to validate repo access: repo_url=...
[GITHUB] 🌐 Calling GitHub API: url=https://api.github.com/repos/owner/repo, token_debug=...
[GITHUB] 📡 GitHub API response: status=200, x-oauth-scopes=repo, read:user, user:email, x-accepted-oauth-scopes=repo, x-ratelimit-remaining=4999
[Repo Check][def67890] ✅✅ REPO ACCESS VALIDATED: full_name=owner/repo, private=false, ...
```

### 4. Diagnosing "Bad Credentials" Error

If you see:

```
[GITHUB] 📡 GitHub API response: status=401, x-oauth-scopes=<not present>, x-accepted-oauth-scopes=repo, x-ratelimit-remaining=<not present>
[GITHUB] ❌ AUTHENTICATION FAILED (401): error_body={"message":"Bad credentials"...}, token_debug=...
```

**Check these things:**

1. **Compare token_debug across logs:**
   - Does the `sha256_prefix` in `[OAuth Exchange]` match `[Internal Token]` match `[GITHUB]`?
   - If they differ, the wrong token is being used somewhere.

2. **Check x-oauth-scopes header:**
   - If `<not present>`, the token is completely invalid (revoked, wrong app, etc.)
   - If present but missing `repo`, the token doesn't have the right scopes

3. **Check the database directly:**
   ```sql
   SELECT id, platform, username, scope, token_expires_at, is_active, created_at, updated_at
   FROM social_connections
   WHERE platform = 'github'
   ORDER BY updated_at DESC
   LIMIT 5;
   ```
   - Is `is_active = true`?
   - Does `scope` contain `repo`?
   - Is `token_expires_at` NULL or in the future?

4. **Verify GitHub OAuth App:**
   - Go to GitHub → Settings → Developer settings → OAuth Apps
   - Find your ConHub OAuth app
   - Check that Client ID matches `GITHUB_CLIENT_ID` in `auth/.env`
   - Check that the callback URL is `http://localhost:3000/auth/callback`

### 5. Common Issues and Fixes

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| `x-oauth-scopes=<not present>` | Token revoked or wrong OAuth app | Disconnect and reconnect GitHub |
| `scope` missing `repo` | OAuth app scopes changed | Update OAuth app scopes, reconnect |
| Token mismatch in sha256_prefix | Multiple connections, wrong one selected | Check DB for duplicate rows |
| `token_expires_at` in past | Token expired | Reconnect GitHub |
| `is_active = false` | Connection was disconnected | Reconnect GitHub |

### 6. Force Fresh Connection

If all else fails:

1. **Delete the connection from DB:**
   ```sql
   DELETE FROM social_connections 
   WHERE user_id = 'your-user-id' AND platform = 'github';
   ```

2. **Reconnect GitHub in the UI**

3. **Watch the logs for the full flow**

## Environment Variables Reference

### auth/.env
```env
GITHUB_CLIENT_ID=Ov23li...        # From GitHub OAuth App
GITHUB_CLIENT_SECRET=...           # From GitHub OAuth App
OAUTH_REDIRECT_URI=http://localhost:3000/auth/callback
```

### data/.env
```env
AUTH_SERVICE_URL=http://localhost:3010
```

## Log Level Configuration

For maximum detail:
```bash
RUST_LOG=debug,conhub=trace cargo run
```

For production-safe logging:
```bash
RUST_LOG=info cargo run
```
