# ConHub - Stop All Services (Windows PowerShell)
Write-Host "Stopping ConHub Services..." -ForegroundColor Red

# Function to stop process on port
function Stop-ProcessOnPort {
    param([int]$Port)
    try {
        $connection = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
        if ($connection) {
            $process = Get-Process -Id $connection.OwningProcess -ErrorAction SilentlyContinue
            if ($process) {
                Write-Host "Stopping process on port $Port ($($process.ProcessName))..." -ForegroundColor Yellow
                Stop-Process -Id $process.Id -Force
                Write-Host "Port $Port freed" -ForegroundColor Green
            }
        }
    } catch {}
}

# Stop all ConHub-related processes
Write-Host "Stopping ConHub processes..." -ForegroundColor Yellow

# Stop Rust services
Get-Process -Name "conhub-backend" -ErrorAction SilentlyContinue | ForEach-Object { 
    Write-Host "Stopping conhub-backend (PID: $($_.Id))" -ForegroundColor Yellow
    Stop-Process -Id $_.Id -Force 
}
Get-Process -Name "lexor" -ErrorAction SilentlyContinue | ForEach-Object { 
    Write-Host "Stopping lexor (PID: $($_.Id))" -ForegroundColor Yellow
    Stop-Process -Id $_.Id -Force 
}

# Stop Python/AI service processes
Get-Process -Name "python" -ErrorAction SilentlyContinue | Where-Object { 
    $_.CommandLine -like "*uvicorn*" -or $_.CommandLine -like "*ai-service*" 
} | ForEach-Object {
    Write-Host "Stopping AI service (PID: $($_.Id))" -ForegroundColor Yellow
    Stop-Process -Id $_.Id -Force
}

# Stop Node.js processes (Next.js, concurrently, monitors)
Get-Process -Name "node" -ErrorAction SilentlyContinue | Where-Object {
    $_.CommandLine -like "*next*" -or 
    $_.CommandLine -like "*concurrently*" -or 
    $_.CommandLine -like "*monitor-services*"
} | ForEach-Object {
    Write-Host "Stopping Node.js service (PID: $($_.Id))" -ForegroundColor Yellow
    Stop-Process -Id $_.Id -Force
}

# Force stop by ports (catch any remaining processes)
Write-Host "Checking ports..." -ForegroundColor Yellow
Stop-ProcessOnPort -Port 3000  # Frontend
Stop-ProcessOnPort -Port 3001  # Backend
Stop-ProcessOnPort -Port 3002  # Lexor
Stop-ProcessOnPort -Port 8001  # AI Service

# Wait a moment for processes to fully terminate
Start-Sleep -Seconds 2

Write-Host "All ConHub services stopped." -ForegroundColor Green