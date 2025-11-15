# ConHub Scripts

This directory contains utility scripts for managing ConHub services and infrastructure.

## JavaScript Scripts (OS Independent)

### Service Management
```bash
# Check status of all services
node services/status.js

# Start all services with smart ordering and health checks
node services/start.js

# Start specific services
node services/start.js auth data frontend

# Stop all services
node services/stop.js

# Stop specific services
node services/stop.js auth frontend
```

## Python Scripts (Database & Testing)

### Database Management
```bash
# Check database health and connectivity
python database_manager.py health

# List all users
python database_manager.py list-users

# Delete a user by email
python database_manager.py delete-user user@example.com

# Run a migration file
python database_manager.py migrate ../database/migrations/001_create_auth_tables.sql
```

## Setup

Install Python dependencies:
```bash
pip install -r requirements.txt
```

## Directory Structure

- `database/` - Database migration files and schemas
- `deployment/` - Deployment configurations and scripts
- `docker/` - Docker-related configurations
- `maintenance/` - Maintenance and cleanup scripts
- `services/` - Service deployment scripts
- `setup/` - Initial setup and configuration scripts
- `test/` - Test utilities and fixtures
- `utils/` - General utility scripts

## Environment Variables

Make sure you have the following environment variables set:
- `DATABASE_URL_NEON` - Neon database connection string
- `NODE_ENV` - Environment mode (development/production)

## Architecture

- **JavaScript**: Service management (start/stop/status) - OS independent
- **Python**: Database operations and testing - Consistent testing framework
- **No PowerShell/Shell**: Avoid OS-specific scripts for better portability

## Notes

- Service management uses Node.js for cross-platform compatibility
- Database operations and tests use Python for consistency
- Smart-start handles service dependencies and optimal startup order
- Health checks verify service availability on their respective ports
