# Check ConHub Services Status
Write-Host "[STATUS] Checking ConHub services..." -ForegroundColor Cyan

$services = @(
    @{Name="Frontend"; Port=3000},
    @{Name="Backend"; Port=3001},
    @{Name="Lexor"; Port=3002},
    @{Name="Doc Search"; Port=8001},
    @{Name="Langchain Service"; Port=8002}
)

foreach ($service in $services) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:$($service.Port)" -TimeoutSec 2 -ErrorAction SilentlyContinue
        if ($response.StatusCode -eq 200) {
            Write-Host "✓ $($service.Name) - Running on port $($service.Port)" -ForegroundColor Green
        } else {
            Write-Host "✗ $($service.Name) - Not responding on port $($service.Port)" -ForegroundColor Red
        }
    } catch {
        Write-Host "✗ $($service.Name) - Not running on port $($service.Port)" -ForegroundColor Red
    }
}