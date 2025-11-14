# Branch Fetching Fix - ConHub

## Issue Summary
The repository branch fetching functionality was failing due to credential format mismatch between frontend and backend, and missing file extension detection.

## Root Causes Identified

### 1. Credential Format Mismatch
- **Frontend**: Sending credentials as `HashMap<String, String>` with keys like `accessToken`
- **Backend**: Expecting `RepositoryCredentials` struct with `CredentialType` enum
- **Impact**: Authentication failures when trying to fetch branches

### 2. Missing VCS Connector Integration
- The data sources handler was using the old `DataSourceFactory` instead of the new `VcsConnector`
- This caused issues with GitHub API authentication and branch detection

### 3. Missing File Extension Detection
- No functionality to detect and return file types present in repositories
- Frontend needed this information for better user experience

## Fixes Implemented

### 1. Updated Data Sources Handler (`data/src/handlers/data_sources.rs`)
```rust
// Before: Using DataSourceFactory with HashMap credentials
let credentials = req.credentials.clone().unwrap_or_default();
match DataSourceFactory::create_connector(source_type) {
    // ... old implementation
}

// After: Using VcsConnector with proper RepositoryCredentials
let credentials = match req.credentials.as_ref() {
    Some(creds) => {
        if let Some(access_token) = creds.get("accessToken") {
            RepositoryCredentials {
                credential_type: CredentialType::PersonalAccessToken {
                    token: access_token.clone(),
                },
                expires_at: None,
            }
        }
        // ... handle other credential types
    }
    // ... fallback to None
};

let connector = VcsConnectorFactory::create_connector(&vcs_type);
```

### 2. Enhanced VCS Connector (`data/src/services/data/vcs_connector.rs`)
- Added `get_file_extensions()` method to `VcsConnector` trait
- Implemented file extension detection by scanning repository files
- Fixed base64 decoding for modern Rust versions
- Added proper error handling for different authentication scenarios

### 3. Updated Frontend (`frontend/components/sources/connectors/repositories/ConnectRepositoryDialog.tsx`)
- Enhanced response handling to include file extensions
- Updated UI to show detected file types
- Improved success messages with file extension information
- Better error handling for authentication issues

### 4. Feature Toggle Configuration
- Enabled `Auth: true` and `Redis: true` in `feature-toggles.json`
- This ensures proper database connections and authentication middleware

## Key Technical Improvements

### Authentication Flow
```
Frontend Credentials → Backend Conversion → VCS API
{                      RepositoryCredentials   GitHub/GitLab/etc
  "accessToken": "..."  {
}                        credential_type: PersonalAccessToken { token }
                        expires_at: None
                      }
```

### File Extension Detection
```rust
async fn get_file_extensions(&self, url: &str, branch: &str, credentials: &RepositoryCredentials) -> VcsResult<Vec<String>> {
    let files = self.list_files(url, "", branch, credentials, true).await?;
    let mut extensions = HashSet::new();
    
    for file_path in files {
        let path = file_path.splitn(3, '/').nth(2).unwrap_or(&file_path);
        if let Some(ext_start) = path.rfind('.') {
            let ext = &path[ext_start..];
            if ext.len() > 1 && ext.len() <= 10 {
                extensions.insert(ext.to_lowercase());
            }
        }
    }
    
    let mut result: Vec<String> = extensions.into_iter().collect();
    result.sort();
    Ok(result)
}
```

## API Response Format
```json
{
  "success": true,
  "message": "Retrieved 5 branches",
  "data": {
    "branches": ["main", "develop", "feature/auth", "hotfix/security", "release/v1.0"],
    "defaultBranch": "main",
    "file_extensions": [".js", ".ts", ".jsx", ".tsx", ".md", ".json", ".yml"]
  },
  "error": null
}
```

## Testing

### Manual Testing Steps
1. Start the data service: `npm run dev:data`
2. Open the frontend repository connection dialog
3. Enter a GitHub repository URL
4. Add a valid GitHub token
5. Click "Check" button
6. Verify branches and file extensions are displayed

### Automated Testing
Run the test script:
```bash
node test-branch-fetch.js
```

## Error Handling Improvements

### Authentication Errors
- **401 Unauthorized**: "Authentication failed: Invalid credentials"
- **403 Forbidden**: "Permission denied: Access denied"
- **404 Not Found**: "Repository not found: Repository not found"

### Network Errors
- Proper timeout handling
- Retry logic for transient failures
- Clear error messages for users

## Security Considerations

### Token Handling
- Tokens are not logged in production
- Proper credential type detection (classic vs fine-grained GitHub tokens)
- Support for different authentication methods (PAT, App Password, etc.)

### API Rate Limiting
- Graceful handling of GitHub API rate limits
- Clear error messages when limits are exceeded
- Suggestions for users to wait before retrying

## Performance Optimizations

### Caching
- Branch information can be cached for short periods
- File extension detection results are cached
- Reduced API calls through intelligent batching

### Async Processing
- Non-blocking file extension detection
- Parallel processing of multiple repositories
- Efficient memory usage for large repositories

## Future Enhancements

### 1. Webhook Integration
- Real-time branch updates
- Automatic re-scanning when repositories change
- Push notification support

### 2. Advanced File Analysis
- Language detection beyond file extensions
- Code complexity metrics
- Repository health scoring

### 3. Multi-Provider Support
- Enhanced GitLab support
- Bitbucket Cloud/Server integration
- Azure DevOps repositories

## Deployment Notes

### Environment Variables
Ensure these are set:
```bash
DATABASE_URL_NEON=postgresql://user:pass@host/db
GITHUB_CLIENT_ID=your_github_client_id
GITHUB_CLIENT_SECRET=your_github_client_secret
```

### Service Dependencies
- PostgreSQL (local or Neon)
- Redis (for session management)
- Data service running on port 3013
- Frontend running on port 3000

## Monitoring and Logging

### Key Metrics
- Branch fetch success rate
- API response times
- Authentication failure rates
- File extension detection accuracy

### Log Messages
- `INFO: Successfully fetched X branches for repository: URL`
- `ERROR: Failed to fetch branches for repository URL: error`
- `INFO: Found X file types: extensions`

This fix resolves the core issue of branch fetching failures and adds valuable file extension detection functionality to enhance the user experience.