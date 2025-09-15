# ConHub - Show Service URLs (PowerShell)

function Show-ServiceStatus {
    Write-Host "" 
    Write-Host "Checking ConHub services status..." -ForegroundColor Cyan
    
    $services = @(
        @{Name="Frontend (Next.js)"; Port=3000; URL="http://localhost:3000"}
        @{Name="Backend (Rust)"; Port=3001; URL="http://localhost:3001"}
        @{Name="LangChain Service"; Port=3002; URL="http://localhost:3002"}
        @{Name="Haystack Service"; Port=8001; URL="http://localhost:8001"}
    )
    
    $runningServices = @()
    $stoppedServices = @()
    
    foreach ($service in $services) {
        try {
            $connection = Get-NetTCPConnection -LocalPort $service.Port -State Listen -ErrorAction SilentlyContinue
            if ($connection) {
                $runningServices += $service
            } else {
                $stoppedServices += $service
            }
        } catch {
            $stoppedServices += $service
        }
    }
    
    if ($runningServices.Count -gt 0) {
        Write-Host ""
        Write-Host "Running Services:" -ForegroundColor Green
        Write-Host "================================================================" -ForegroundColor Cyan
        foreach ($service in $runningServices) {
            $paddedName = $service.Name.PadRight(25)
            Write-Host "   OK  $paddedName $($service.URL)" -ForegroundColor White
        }
        Write-Host "================================================================" -ForegroundColor Cyan
    }
    
    if ($stoppedServices.Count -gt 0) {
        Write-Host ""
        Write-Host "Stopped Services:" -ForegroundColor Red
        foreach ($service in $stoppedServices) {
            $paddedName = $service.Name.PadRight(25)
            Write-Host "   --  $paddedName $($service.URL) (Not running)" -ForegroundColor Gray
        }
    }
    
    if ($runningServices.Count -eq $services.Count) {
        Write-Host ""
        Write-Host "All ConHub services are running perfectly!" -ForegroundColor Green
        Write-Host "Open your browser and navigate to http://localhost:3000 to start using ConHub" -ForegroundColor Yellow
    } elseif ($runningServices.Count -gt 0) {
        Write-Host ""
        Write-Host "Some services are not running. Run 'npm start' to start all services." -ForegroundColor Yellow
    } else {
        Write-Host ""
        Write-Host "No ConHub services are currently running. Run 'npm start' to start all services." -ForegroundColor Red
    }
    
    Write-Host ""
}

# Show the service status
Show-ServiceStatus