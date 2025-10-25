-- ConHub Database Initialization Script
-- This script runs automatically when the postgres container is first created

-- Ensure the conhub database is created (done by POSTGRES_DB env var)
-- This file can be used for additional initialization if needed

-- Grant necessary permissions
GRANT ALL PRIVILEGES ON DATABASE conhub TO conhub;

-- Connect to conhub database
\c conhub

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Grant schema privileges
GRANT ALL ON SCHEMA public TO conhub;
