# ConHub Scripts Directory

This directory contains platform-specific scripts for managing ConHub services.

## Core Scripts

### Platform Detection
- `check-platform.js` - Detects OS and runs appropriate platform-specific script

### Windows Scripts (.ps1)
- `start.ps1` - Starts all ConHub services with port conflict detection
- `stop.ps1` - Stops all ConHub services 
- `check.ps1` - Health checks for all services with detailed endpoint testing
- `status.ps1` - Quick status check showing which services are running
- `test-settings.ps1` - Comprehensive API endpoint testing for settings

### Linux/macOS Scripts (.sh)
- `start.sh` - Starts all ConHub services on Unix-like systems
- `stop.sh` - Stops all ConHub services on Unix-like systems  
- `check.sh` - Health checks for all services on Unix-like systems

### Helper Scripts
- `monitor-services.js` - Background service to monitor startup and display URLs

## Usage

### Start Services
```bash
npm start              # Cross-platform (uses check-platform.js)
npm run start:windows  # Windows specific
npm run start:linux    # Linux/macOS specific
```

### Check Service Status
```bash
npm run status         # Quick status check
npm run check:services # Comprehensive health check
```

### Stop Services
```bash
npm run stop:windows   # Windows
npm run stop:linux     # Linux/macOS
```

### Test APIs
```bash
npm run test:settings  # Test settings API endpoints
```

## Service Ports
- Frontend (Next.js): 3000
- Backend (Rust): 3001  
- LangChain Service: 3002
- Haystack Service: 8001