# Stop ConHub Services
Write-Host "[STOP] Stopping ConHub services..." -ForegroundColor Yellow

# Use force-stop to clean up all processes
& "$PSScriptRoot\force-stop.ps1"

Write-Host "[OK] All services stopped" -ForegroundColor Green