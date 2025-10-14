
Write-Host "[STOP] Stopping ConHub services..." -ForegroundColor Yellow


& "$PSScriptRoot\..\maintenance\force-stop.ps1"

Write-Host "[OK] All services stopped" -ForegroundColor Green