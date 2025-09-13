# ConHub Scripts

This folder contains platform-specific scripts to manage the ConHub application services.

## Quick Start

### All Platforms (Automatic Detection)
```bash
# Start all services
npm start

# Test all services
npm run test:services

# Check service health
npm run check:services

# Stop services (Linux/macOS only)
npm run stop:linux
```

### Windows
```powershell
# PowerShell (Recommended)
.\scripts\start.ps1        # Start all services
.\scripts\stop.ps1         # Stop all services
.\scripts\test.ps1         # Test all services
.\scripts\check.ps1        # Check service health

# Individual services
.\scripts\frontend.ps1     # Start frontend only
.\scripts\backend.ps1      # Start backend only
.\scripts\haystack.ps1     # Start Haystack only

# Command Prompt
scripts\start.bat          # Start all services
scripts\stop.bat           # Stop all services
scripts\test.bat           # Test all services
scripts\check.bat          # Check service health
```

### Linux/macOS
```bash
# All services
./scripts/start.sh         # Start all services
./scripts/stop.sh          # Stop all services
./scripts/test.sh          # Test all services
./scripts/check.sh         # Check service health

# Individual services
./scripts/frontend.sh      # Start frontend only
./scripts/backend.sh       # Start backend only
./scripts/haystack.sh      # Start Haystack only
```

## Available Scripts

| Script | Windows | Linux/macOS | Description |
|--------|---------|-------------|-------------|
| **start** | `.ps1`, `.bat` | `.sh` | Start all services |
| **stop** | `.ps1`, `.bat` | `.sh` | Stop all services |
| **test** | `.ps1`, `.bat` | `.sh` | Test service endpoints |
| **check** | `.ps1`, `.bat` | `.sh` | Quick health check |
| **frontend** | `.ps1`, `.bat` | `.sh` | Frontend only |
| **backend** | `.ps1`, `.bat` | `.sh` | Backend only |
| **haystack** | `.ps1`, `.bat` | `.sh` | Haystack service only |

## Service Ports

- **Frontend**: http://localhost:3000 (Next.js)
- **Backend**: http://localhost:3001 (Rust API)
- **LangChain**: http://localhost:3003 (TypeScript Service)
- **Haystack**: http://localhost:8001 (Python API)

## Platform Detection

The `check-platform.js` script automatically detects your platform:
- Windows: Runs `.ps1` scripts by default, falls back to `.bat`
- Linux/macOS: Runs `.sh` scripts

## Notes

- All services run in background mode
- LangChain service is started but may need additional configuration
- Lexor service is currently disabled due to compilation issues
- Python services use the `.venv` virtual environment

## Services and Ports

| Service | Port | Description |
|---------|------|-------------|
| Frontend (Next.js) | 3000 | Web application interface |
| Backend (Rust) | 3001 | Main API server |
| LangChain (TypeScript) | 3003 | Language processing service |
| Haystack (Python) | 8001 | Document search and QA service |

## Prerequisites

1. **Node.js** and **npm** installed
2. **Rust** and **Cargo** installed
3. **Python** virtual environment (`.venv`) created in project root
4. All dependencies installed:
   ```bash
   npm install
   pip install -r requirements.txt
   ```

## Platform-Specific Notes

### Windows
- Uses PowerShell for better process management
- Each service runs in a separate window
- Virtual environment: `.venv\Scripts\Activate.ps1`

### Linux/macOS
- Services run in background with logging
- Logs saved to `logs/` directory
- Virtual environment: `.venv/bin/activate`
- Use `./scripts/stop-all.sh` to stop all services

## Troubleshooting

### Windows: Execution Policy Error
If you get an execution policy error, run:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Linux/macOS: Permission Denied
Make scripts executable:
```bash
chmod +x scripts/*.sh
```

### Virtual Environment Issues
Make sure the Python virtual environment is created and activated:
```bash
# Create virtual environment
python -m venv .venv

# Windows
.venv\Scripts\activate

# Linux/macOS
source .venv/bin/activate

# Install dependencies
pip install -r requirements.txt
```