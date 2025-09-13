# ConHub - Stop All Services (Windows PowerShell)

Write-Host "🛑 Stopping ConHub Services" -ForegroundColor Red
Write-Host "============================"

# Function to stop processes by name
function Stop-ProcessByName {
    param([string]$ProcessName, [string]$ServiceName)
    
    $processes = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue
    if ($processes) {
        Write-Host "🔄 Stopping $ServiceName..." -ForegroundColor Yellow
        $processes | Stop-Process -Force
        Write-Host "✅ $ServiceName stopped" -ForegroundColor Green
    } else {
        Write-Host "⚠️  $ServiceName was not running" -ForegroundColor Yellow
    }
}

# Stop services by process name
Stop-ProcessByName -ProcessName "node" -ServiceName "Frontend (Next.js)"
Stop-ProcessByName -ProcessName "conhub-backend" -ServiceName "Backend (Rust)"
Stop-ProcessByName -ProcessName "ts-node" -ServiceName "LangChain Service"
Stop-ProcessByName -ProcessName "python" -ServiceName "Haystack Service"

# Also try to stop by port (alternative method)
Write-Host "🧹 Cleaning up any remaining processes..." -ForegroundColor Yellow

# Kill processes using specific ports
$ports = @(3000, 3001, 3003, 8001)
foreach ($port in $ports) {
    $processes = netstat -ano | findstr ":$port"
    if ($processes) {
        $pids = $processes | ForEach-Object { ($_ -split '\s+')[-1] } | Sort-Object -Unique
        foreach ($processId in $pids) {
            if ($processId -match '^\d+$') {
                try {
                    Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
                    Write-Host "🔄 Stopped process on port $port (PID: $processId)" -ForegroundColor Cyan
                } catch {
                    # Ignore errors for processes that are already stopped
                }
            }
        }
    }
}

Write-Host ""
Write-Host "✅ All ConHub services have been stopped" -ForegroundColor Green