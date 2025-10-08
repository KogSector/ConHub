# Force stop all ConHub services
$processes = @("node", "next-server", "conhub-backend", "lexor", "python", "uvicorn")

foreach ($proc in $processes) {
    try {
        Get-Process -Name $proc -ErrorAction SilentlyContinue | Where-Object {
            $_.ProcessName -eq $proc -and (
                $_.CommandLine -like "*3000*" -or
                $_.CommandLine -like "*3001*" -or
                $_.CommandLine -like "*3002*" -or
                $_.CommandLine -like "*8001*" -or
                $_.CommandLine -like "*8002*" -or
                $_.CommandLine -like "*conhub*" -or
                $_.CommandLine -like "*lexor*" -or
                $_.CommandLine -like "*uvicorn*"
            )
        } | Stop-Process -Force -ErrorAction SilentlyContinue
    } catch {
        # Ignore errors
    }
}

# Kill by port
$ports = @(3000, 3001, 3002, 8001, 8002)
foreach ($port in $ports) {
    try {
        $pid = (netstat -ano | findstr ":$port " | ForEach-Object { ($_ -split '\s+')[-1] } | Select-Object -First 1)
        if ($pid) {
            taskkill /PID $pid /F *>$null
        }
    } catch {
        # Ignore errors
    }
}