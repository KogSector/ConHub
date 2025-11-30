-- Migration: Add last_sync column to social_connections table
-- This column tracks when each connection was last synchronized

ALTER TABLE social_connections 
ADD COLUMN IF NOT EXISTS last_sync TIMESTAMPTZ;

-- Create index for efficient querying by sync status
CREATE INDEX IF NOT EXISTS idx_social_connections_last_sync 
ON social_connections(last_sync);

-- Add comment for documentation
COMMENT ON COLUMN social_connections.last_sync IS 'Timestamp of the last successful sync operation for this connection';
