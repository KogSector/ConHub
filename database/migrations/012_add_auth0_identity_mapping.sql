-- Add Auth0 identity mapping to users table
-- This allows us to map Auth0 sub (e.g., "auth0|abc123") to ConHub user IDs

ALTER TABLE users 
ADD COLUMN IF NOT EXISTS auth0_sub VARCHAR(255) UNIQUE;

-- Create index for faster Auth0 sub lookups
CREATE INDEX IF NOT EXISTS idx_users_auth0_sub ON users(auth0_sub) WHERE auth0_sub IS NOT NULL;

-- Add comment for documentation
COMMENT ON COLUMN users.auth0_sub IS 'Auth0 subject identifier (e.g., auth0|abc123, google-oauth2|xyz) for external identity mapping';

-- Update audit event types to include Auth0 events
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'auth0_exchange';
ALTER TYPE audit_event_type ADD VALUE IF NOT EXISTS 'auth0_link';
