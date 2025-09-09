# Feature Toggles

ConHub uses feature toggles to enable/disable functionality during development.

## Configuration

Feature toggles are configured in `feature-toggles.json` at the project root:

```json
{
  "Login": false
}
```

## Available Toggles

### Login
- **Type**: Boolean
- **Default**: `false`
- **Description**: Controls authentication system

When `Login` is `true`:
- Full Auth0 authentication is enabled
- Users must sign in to access protected routes
- Real user data is used throughout the app

When `Login` is `false`:
- Authentication is bypassed
- Mock user data is provided
- All routes are accessible without authentication
- AuthGuard components allow access automatically

## Usage

### Checking Feature Status
```typescript
import { isLoginEnabled, isFeatureEnabled } from '@/lib/feature-toggles'

// Check specific features
const loginEnabled = isLoginEnabled()
const customFeature = isFeatureEnabled('CustomFeature')
```

### Protecting Routes
```typescript
import { AuthGuard } from '@/components/auth/AuthGuard'

export default function ProtectedPage() {
  return (
    <AuthGuard>
      <div>This content is protected</div>
    </AuthGuard>
  )
}
```

### Custom Fallback
```typescript
<AuthGuard fallback={<div>Please sign in</div>}>
  <ProtectedContent />
</AuthGuard>
```

## Development Workflow

1. **Working on non-auth features**: Set `"Login": false`
2. **Testing authentication**: Set `"Login": true`
3. **Production**: Always set `"Login": true`

## Mock Data

When login is disabled, the following mock user is provided:
```json
{
  "name": "Development User",
  "email": "dev@conhub.local",
  "picture": undefined
}
```