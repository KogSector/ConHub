# ConHub - Start All Services (Windows PowerShell)
Write-Host "Starting ConHub Services..." -ForegroundColor Green

# Check if we're in the right directory
if (-not (Test-Path "package.json")) {
    Write-Host "Error: Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

# First, ensure all services are stopped
Write-Host "🛑 Ensuring all services are stopped before starting..." -ForegroundColor Yellow
& "$PSScriptRoot\stop.ps1"
Start-Sleep -Seconds 3

# Start the service monitor in background
Write-Host "🔍 Starting service monitor..." -ForegroundColor Cyan
Start-Process -FilePath "node" -ArgumentList "scripts/monitor-services.js" -WindowStyle Hidden

# Use concurrently to start all services
Write-Host "🚀 Starting all services with concurrently..." -ForegroundColor Yellow

# Register cleanup handler for Ctrl+C
$null = Register-ObjectEvent -InputObject ([Console]) -EventName "CancelKeyPress" -Action {
    Write-Host "`n🛑 Shutting down all services..." -ForegroundColor Red
    # Kill any node processes that might be running the monitor
    Get-Process -Name "node" -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -like "*monitor-services*" } | Stop-Process -Force -ErrorAction SilentlyContinue
}

try {
    # Start concurrently with better error handling
    npx concurrently --kill-others --names "Frontend,Backend,Lexor,AI" --prefix-colors "cyan,blue,magenta,yellow" --restart-tries 3 "npm run dev:frontend" "npm run dev:backend" "npm run dev:lexor" "npm run dev:ai"
} catch {
    Write-Host "❌ Error starting services: $_" -ForegroundColor Red
    exit 1
}

# This point should only be reached if concurrently exits
Write-Host "`n🏁 ConHub services have stopped." -ForegroundColor Yellow
