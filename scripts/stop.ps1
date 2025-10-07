# Stop all ConHub services
Write-Host "🛑 Stopping ConHub services..." -ForegroundColor Red

# Function to kill processes by port
function Stop-ProcessByPort {
    param([int]$Port)
    
    try {
        $processes = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess
        foreach ($processId in $processes) {
            if ($processId -and $processId -ne 0) {
                Write-Host "Stopping process $processId on port $Port" -ForegroundColor Yellow
                Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
            }
        }
    } catch {
        Write-Host "No process found on port $Port" -ForegroundColor Gray
    }
}

# Function to kill processes by name
function Stop-ProcessByName {
    param([string]$ProcessName)
    
    try {
        $processes = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue
        foreach ($process in $processes) {
            Write-Host "Stopping $ProcessName process (PID: $($process.Id))" -ForegroundColor Yellow
            Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        }
    } catch {
        Write-Host "No $ProcessName processes found" -ForegroundColor Gray
    }
}

# Stop processes by port
Write-Host "Stopping services by port..." -ForegroundColor Cyan
Stop-ProcessByPort -Port 3000  # Frontend
Stop-ProcessByPort -Port 3001  # Backend
Stop-ProcessByPort -Port 3002  # Lexor
Stop-ProcessByPort -Port 8001  # AI Service

# Stop processes by name
Write-Host "Stopping processes by name..." -ForegroundColor Cyan
Stop-ProcessByName -ProcessName "node"
Stop-ProcessByName -ProcessName "conhub-backend"
Stop-ProcessByName -ProcessName "lexor"
Stop-ProcessByName -ProcessName "python"
Stop-ProcessByName -ProcessName "uvicorn"

# Kill any remaining npm processes
Write-Host "Stopping npm processes..." -ForegroundColor Cyan
try {
    taskkill /f /im npm.cmd 2>$null
    taskkill /f /im node.exe 2>$null
} catch {
    # Ignore errors
}

# Wait a moment for processes to terminate
Start-Sleep -Seconds 2

Write-Host "✅ All ConHub services stopped" -ForegroundColor Green