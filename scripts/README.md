# ConHub Scripts

This directory contains automation scripts for managing the ConHub development environment.

## Database Scripts

### Setup Database
Sets up the PostgreSQL database and applies the schema:

**Windows (PowerShell):**
```powershell
.\scripts\setup-database.ps1
```

**Linux/macOS (Bash):**
```bash
./scripts/setup-database.sh
```

### Test Database Connection
Tests the PostgreSQL connection and verifies the database setup:

**Windows (PowerShell):**
```powershell
.\scripts\test-database.ps1
```

**Linux/macOS (Bash):**
```bash
./scripts/test-database.sh
```

## Service Management Scripts

### Start Services
Starts all ConHub services (backend, frontend, etc.):

**Windows (PowerShell):**
```powershell
.\scripts\start.ps1
```

**Linux/macOS (Bash):**
```bash
./scripts/start.sh
```

### Stop Services
Stops all running ConHub services:

**Windows (PowerShell):**
```powershell
.\scripts\stop.ps1
```

**Linux/macOS (Bash):**
```bash
./scripts/stop.sh
```

### Check Status
Checks the status of all ConHub services:

**Windows (PowerShell):**
```powershell
.\scripts\status.ps1
```

## Testing Scripts

### Test Settings
Tests the settings configuration:

**Windows (PowerShell):**
```powershell
.\scripts\test-settings.ps1
```

### Test URLs
Tests URL connectivity:

**Windows (PowerShell):**
```powershell
.\scripts\test-urls.ps1
```

### Check Platform
Checks platform compatibility:

**Node.js:**
```bash
node scripts/check-platform.js
```

## Prerequisites

- PostgreSQL 17+ installed and running
- Node.js 18+ for frontend services
- Rust 1.70+ for backend services
- Git for version control

## Database Setup Workflow

1. **First Time Setup:**
   ```bash
   # Windows
   .\scripts\setup-database.ps1
   
   # Linux/macOS
   ./scripts/setup-database.sh
   ```

2. **Test Connection:**
   ```bash
   # Windows
   .\scripts\test-database.ps1
   
   # Linux/macOS
   ./scripts/test-database.sh
   ```

3. **Update .env file** with your PostgreSQL password when prompted

4. **Start Services:**
   ```bash
   # Windows
   .\scripts\start.ps1
   
   # Linux/macOS
   ./scripts/start.sh
   ```

## Notes

- All scripts should be run from the project root directory
- Ensure PostgreSQL is running before executing database scripts
- Scripts will prompt for credentials when needed
- Log files are created in the project root for debugging