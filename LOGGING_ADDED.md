# Comprehensive Logging Added to ConHub

## Overview
Added extensive logging throughout the branch fetching flow to debug the "Bad Request" issue.

## Logging Points Added

### 1. Frontend Logging (`ConnectRepositoryDialog.tsx`)
```javascript
// Request initiation
console.log('[FRONTEND] handleFetchBranches called');
console.log('[FRONTEND] repositoryUrl:', repositoryUrl);
console.log('[FRONTEND] provider:', provider);
console.log('[FRONTEND] credentials:', credentials);

// Request payload
console.log('[FRONTEND] Request payload:', requestPayload);
console.log('[FRONTEND] Making API call to /api/data/sources/branches');

// Response handling
console.log('[FRONTEND] API response:', resp);
console.error('[FRONTEND] API error:', resp.error);
console.log('[FRONTEND] Parsed response data:', { fetchedBranches, default_branch, file_extensions });

// Error handling
console.error('[FRONTEND] Branch fetching error:', err);
console.error('[FRONTEND] Error message:', errorMessage);
```

### 2. Data Service Main Handler (`main.rs`)
```rust
// Request interception
tracing::info!("[MAIN] Received fetch_branches request");
tracing::info!("[MAIN] Request body: {:?}", req);

// Response handling
tracing::info!("[MAIN] fetch_branches succeeded");
tracing::error!("[MAIN] fetch_branches failed: {:?}", e);
```

### 3. Data Sources Handler (`handlers/data_sources.rs`)
```rust
// Request start
info!("[FETCH_BRANCHES] Starting request for: {}", req.repo_url);
info!("[FETCH_BRANCHES] Credentials provided: {}", req.credentials.is_some());
info!("[FETCH_BRANCHES] Credential keys: {:?}", creds.keys().collect::<Vec<_>>());

// VCS Detection
info!("[FETCH_BRANCHES] Detecting VCS type for URL: {}", req.repo_url);
info!("[FETCH_BRANCHES] Detected VCS: {:?}, Provider: {:?}", result.0, result.1);
error!("[FETCH_BRANCHES] VCS detection failed: {}", e);

// Credential Conversion
info!("[FETCH_BRANCHES] Converting credentials");
info!("[FETCH_BRANCHES] Using PersonalAccessToken (length: {})", access_token.len());
info!("[FETCH_BRANCHES] Using AppPassword for user: {}", username);
info!("[FETCH_BRANCHES] No valid credentials found, using None");

// VCS Connector
info!("[FETCH_BRANCHES] Creating VCS connector for type: {:?}", vcs_type);
info!("[FETCH_BRANCHES] Calling list_branches");

// Success Path
info!("[FETCH_BRANCHES] Successfully got {} branches", branch_info.len());
info!("[FETCH_BRANCHES] Branch names: {:?}", branches);
info!("[FETCH_BRANCHES] Default branch: {:?}", default_branch);

// File Extensions
info!("[FETCH_BRANCHES] Fetching file extensions");
info!("[FETCH_BRANCHES] Getting file extensions for branch: {}", branch);
info!("[FETCH_BRANCHES] Found {} file extensions: {:?}", extensions.len(), extensions);
warn!("[FETCH_BRANCHES] Could not fetch file extensions: {}", e);

// Final Response
info!("[FETCH_BRANCHES] SUCCESS: {} branches for {}", branches.len(), req.repo_url);
info!("[FETCH_BRANCHES] Response: {:?}", response);

// Error Handling
error!("[FETCH_BRANCHES] ERROR: Failed for {}: {}", req.repo_url, e);
error!("[FETCH_BRANCHES] Authentication failed: {}", msg);
error!("[FETCH_BRANCHES] Repository not found: {}", msg);
error!("[FETCH_BRANCHES] Permission denied: {}", msg);
error!("[FETCH_BRANCHES] Generic error: {}", e);
error!("[FETCH_BRANCHES] Returning BadRequest: {}", error_msg);
```

### 4. VCS Connector (`services/data/vcs_connector.rs`)
```rust
// Method Entry
tracing::info!("[VCS_CONNECTOR] list_branches called for: {}", url);

// URL Processing
tracing::error!("[VCS_CONNECTOR] URL detection failed: {}", e);
tracing::info!("[VCS_CONNECTOR] Detected - VCS: {:?}, Provider: {:?}", vcs_type, provider);
tracing::error!("[VCS_CONNECTOR] Repo info extraction failed: {}", e);
tracing::info!("[VCS_CONNECTOR] Extracted - Owner: {}, Repo: {}", owner, repo);

// API Request Setup
tracing::info!("[VCS_CONNECTOR] API base URL: {}", api_base);
tracing::error!("[VCS_CONNECTOR] Unsupported VCS type: {:?}", vcs_type);
tracing::info!("[VCS_CONNECTOR] Making API request to: {}", api_url);

// API Request Execution
tracing::error!("[VCS_CONNECTOR] API request failed: {}", e);
tracing::info!("[VCS_CONNECTOR] API request successful, parsing response");

// Response Processing
tracing::info!("[VCS_CONNECTOR] GitHub: Processing {} branches", branch_array.len());
tracing::debug!("[VCS_CONNECTOR] GitHub: Found branch {}", name);
tracing::error!("[VCS_CONNECTOR] GitHub: Response is not an array");

// Final Result
tracing::info!("[VCS_CONNECTOR] Returning {} branches", branches.len());
```

### 5. API Request Logging (`make_api_request`)
```rust
// Request Setup
tracing::info!("[VCS_CONNECTOR] make_api_request to: {}", url);
tracing::info!("[VCS_CONNECTOR] Credential type: {:?}", std::mem::discriminant(&credentials.credential_type));

// Authentication
tracing::info!("[VCS_CONNECTOR] Using PersonalAccessToken (length: {})", token.len());
tracing::info!("[VCS_CONNECTOR] Using GitLab Bearer auth");
tracing::info!("[VCS_CONNECTOR] Using Bearer auth for fine-grained GitHub token");
tracing::info!("[VCS_CONNECTOR] Using token auth for classic GitHub token");
tracing::info!("[VCS_CONNECTOR] Using Bearer auth for unknown token type");
tracing::info!("[VCS_CONNECTOR] Using UsernamePassword for: {}", username);
tracing::info!("[VCS_CONNECTOR] Using AppPassword for: {}", username);
tracing::info!("[VCS_CONNECTOR] No credentials provided");
tracing::warn!("[VCS_CONNECTOR] Unsupported credential type");

// HTTP Request/Response
tracing::info!("[VCS_CONNECTOR] Sending HTTP request");
tracing::error!("[VCS_CONNECTOR] Network error: {}", e);
tracing::info!("[VCS_CONNECTOR] HTTP response status: {}", status);
tracing::info!("[VCS_CONNECTOR] Parsing JSON response");
tracing::error!("[VCS_CONNECTOR] JSON parsing error: {}", e);
tracing::info!("[VCS_CONNECTOR] Successfully parsed JSON response");
tracing::error!("[VCS_CONNECTOR] HTTP error {}: {}", status, error_body);
```

## Debug Tools Created

### 1. Debug Script (`debug-branch-fetch.js`)
- Tests the exact API endpoint
- Shows raw HTTP response
- Displays parsed JSON
- Checks service health first

### 2. Test Script (`test-branch-fetch.js`)
- Tests multiple scenarios
- Public and private repositories
- Different credential types
- Comprehensive error reporting

## How to Use the Logging

### 1. Start the Data Service
```bash
npm run dev:data
```

### 2. Check Logs in Terminal
The data service will output detailed logs showing:
- Request reception
- Credential processing
- VCS detection
- API calls to GitHub/GitLab
- Response processing
- Error details

### 3. Check Frontend Console
Open browser dev tools to see:
- Request payload being sent
- API response received
- Error messages
- Credential validation

### 4. Run Debug Script
```bash
node debug-branch-fetch.js
```

## Expected Log Flow for Successful Request

```
[FRONTEND] handleFetchBranches called
[FRONTEND] repositoryUrl: https://github.com/microsoft/vscode
[FRONTEND] provider: github
[FRONTEND] credentials: { accessToken: "ghp_..." }
[FRONTEND] Request payload: { repoUrl: "...", credentials: { accessToken: "..." } }
[FRONTEND] Making API call to /api/data/sources/branches

[MAIN] Received fetch_branches request
[MAIN] Request body: FetchBranchesRequest { repo_url: "...", credentials: Some(...) }

[FETCH_BRANCHES] Starting request for: https://github.com/microsoft/vscode
[FETCH_BRANCHES] Credentials provided: true
[FETCH_BRANCHES] Credential keys: ["accessToken"]
[FETCH_BRANCHES] Detecting VCS type for URL: https://github.com/microsoft/vscode
[FETCH_BRANCHES] Detected VCS: Git, Provider: GitHub
[FETCH_BRANCHES] Converting credentials
[FETCH_BRANCHES] Using PersonalAccessToken (length: 40)
[FETCH_BRANCHES] Creating VCS connector for type: Git
[FETCH_BRANCHES] Calling list_branches

[VCS_CONNECTOR] list_branches called for: https://github.com/microsoft/vscode
[VCS_CONNECTOR] Detected - VCS: Git, Provider: GitHub
[VCS_CONNECTOR] Extracted - Owner: microsoft, Repo: vscode
[VCS_CONNECTOR] API base URL: https://api.github.com
[VCS_CONNECTOR] Making API request to: https://api.github.com/repos/microsoft/vscode/branches
[VCS_CONNECTOR] make_api_request to: https://api.github.com/repos/microsoft/vscode/branches
[VCS_CONNECTOR] Using PersonalAccessToken (length: 40)
[VCS_CONNECTOR] Using token auth for classic GitHub token
[VCS_CONNECTOR] Sending HTTP request
[VCS_CONNECTOR] HTTP response status: 200 OK
[VCS_CONNECTOR] Parsing JSON response
[VCS_CONNECTOR] Successfully parsed JSON response
[VCS_CONNECTOR] API request successful, parsing response
[VCS_CONNECTOR] GitHub: Processing 5 branches
[VCS_CONNECTOR] Returning 5 branches

[FETCH_BRANCHES] Successfully got 5 branches
[FETCH_BRANCHES] Branch names: ["main", "release/1.85", "release/1.84", ...]
[FETCH_BRANCHES] Default branch: Some("main")
[FETCH_BRANCHES] SUCCESS: 5 branches for https://github.com/microsoft/vscode

[MAIN] fetch_branches succeeded

[FRONTEND] API response: { success: true, data: { branches: [...], defaultBranch: "main" } }
[FRONTEND] Parsed response data: { fetchedBranches: [...], default_branch: "main", file_extensions: [...] }
```

## Common Error Patterns to Look For

### 1. Authentication Issues
```
[VCS_CONNECTOR] HTTP response status: 401 Unauthorized
[VCS_CONNECTOR] HTTP error 401: {"message":"Bad credentials"}
```

### 2. Repository Not Found
```
[VCS_CONNECTOR] HTTP response status: 404 Not Found
[VCS_CONNECTOR] HTTP error 404: {"message":"Not Found"}
```

### 3. Rate Limiting
```
[VCS_CONNECTOR] HTTP response status: 403 Forbidden
[VCS_CONNECTOR] HTTP error 403: {"message":"API rate limit exceeded"}
```

### 4. Network Issues
```
[VCS_CONNECTOR] Network error: Connection refused
```

### 5. Invalid URL
```
[FETCH_BRANCHES] VCS detection failed: Invalid URL format
```

This comprehensive logging will help identify exactly where the "Bad Request" is occurring and why.