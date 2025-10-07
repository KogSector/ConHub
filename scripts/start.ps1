# ConHub - Start All Services
Write-Host "[START] Starting ConHub Services..." -ForegroundColor Green

# Check if we're in the right directory
if (-not (Test-Path "package.json")) {
    Write-Host "[ERROR] Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

# Stop existing services
Write-Host "[STOP] Stopping existing services..." -ForegroundColor Yellow
& "$PSScriptRoot\force-stop.ps1" *>$null
Start-Sleep -Seconds 2

# Build Rust binaries if needed
$backendBinary = "target\debug\conhub-backend.exe"
$lexorBinary = "target\debug\lexor.exe"

if (-not (Test-Path $backendBinary) -or -not (Test-Path $lexorBinary)) {
    Write-Host "[BUILD] Building Rust binaries..." -ForegroundColor Cyan
    cargo build --bin conhub-backend --bin lexor --quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Failed to build binaries" -ForegroundColor Red
        exit 1
    }
}

Write-Host "[OK] All binaries ready" -ForegroundColor Green
Write-Host "[SERVICES] Starting all services..." -ForegroundColor Cyan

# Start services using concurrently
.\node_modules\.bin\concurrently.cmd --kill-others --names "Frontend,Backend,Lexor,AI" --prefix-colors "cyan,blue,magenta,yellow" --restart-tries 1 "npm run dev:frontend" "npm run dev:backend" "npm run dev:lexor" "npm run dev:ai"