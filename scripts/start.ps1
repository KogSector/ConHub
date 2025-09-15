# ConHub - Start All Services (Windows PowerShell)
Write-Host "Starting ConHub Services..." -ForegroundColor Green

# Check if we're in the right directory
if (-not (Test-Path "package.json")) {
    Write-Host "Error: Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

# Function to check if a port is in use and kill the process if needed
function Stop-ProcessOnPort {
    param([int]$Port, [string]$ServiceName)
    
    try {
        $connection = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
        if ($connection) {
            $process = Get-Process -Id $connection.OwningProcess -ErrorAction SilentlyContinue
            if ($process) {
                Write-Host "⚠️  Port $Port is already in use by process '$($process.ProcessName)' (PID: $($process.Id))" -ForegroundColor Yellow
                Write-Host "🔄 Stopping existing process to free up port for $ServiceName..." -ForegroundColor Cyan
                Stop-Process -Id $process.Id -Force
                Start-Sleep -Seconds 2
                Write-Host "✅ Port $Port is now available for $ServiceName" -ForegroundColor Green
            }
        }
    } catch {
        # Port is likely free, continue
    }
}

# Check and free up ports if needed
Write-Host "🔍 Checking for port conflicts..." -ForegroundColor Yellow
Stop-ProcessOnPort -Port 3000 -ServiceName "Frontend"
Stop-ProcessOnPort -Port 3001 -ServiceName "Backend" 
Stop-ProcessOnPort -Port 3002 -ServiceName "LangChain"
Stop-ProcessOnPort -Port 8001 -ServiceName "Haystack"

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
    npx concurrently --kill-others --names "Frontend,Backend,LangChain,Haystack" --prefix-colors "cyan,blue,magenta,yellow" --restart-tries 3 "npm run dev:frontend" "npm run dev:backend" "npm run dev:langchain" "npm run dev:haystack"
} catch {
    Write-Host "❌ Error starting services: $_" -ForegroundColor Red
    exit 1
}

# This point should only be reached if concurrently exits
Write-Host "`n🏁 ConHub services have stopped." -ForegroundColor Yellow