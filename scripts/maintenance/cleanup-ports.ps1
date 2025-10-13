
$ports = @(3000, 3001, 3002, 8001, 8003)

foreach ($port in $ports) {
    try {
        $connection = Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($connection) {
            $processId = $connection.OwningProcess
            Write-Host "Port $port is in use by PID $processId. Terminating process..." -ForegroundColor Yellow
            Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
        }
    } catch {
        Write-Host "An error occurred while cleaning up port ${port}: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Corrected lock file path
$lockFile = "lexor_data\index\.tantivy-writer.lock"
if (Test-Path $lockFile) {
    Write-Host "Found lock file at $lockFile. Removing..." -ForegroundColor Yellow
    Remove-Item $lockFile -Force
}

# Give the OS a moment to release resources
Start-Sleep -Seconds 3
