# Clean up ports before starting services
$ports = @(3000, 3001, 3002, 8001, 8002)

foreach ($port in $ports) {
    try {
        $connections = netstat -ano | findstr ":$port "
        if ($connections) {
            $pids = ($connections | ForEach-Object { ($_ -split '\s+')[-1] } | Sort-Object -Unique)
            foreach ($pid in $pids) {
                if ($pid -and $pid -ne "0") {
                    taskkill /PID $pid /F *>$null
                }
            }
        }
    } catch { }
}

# Clean up Lexor index lock
$lockFile = "indexers\lexor_data\index\.tantivy-writer.lock"
if (Test-Path $lockFile) {
    Remove-Item $lockFile -Force
    Write-Host "Removed Lexor index lock" -ForegroundColor Yellow
}

Start-Sleep -Seconds 2