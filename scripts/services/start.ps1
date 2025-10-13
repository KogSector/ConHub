Write-Host "[START] Starting ConHub..." -ForegroundColor Green

if (-not (Test-Path "package.json")) {
    Write-Host "[ERROR] Run from project root" -ForegroundColor Red
    exit 1
}

Write-Host "[CLEANUP] Cleaning up ports and locks..." -ForegroundColor Yellow
& "$PSScriptRoot\..\maintenance\cleanup-ports.ps1"

$backendBinary = "target\debug\conhub-backend.exe"
$lexorBinary = "target\debug\lexor.exe"

if (-not (Test-Path $backendBinary) -or -not (Test-Path $lexorBinary)) {
    Write-Host "[BUILD] Building binaries..." -ForegroundColor Cyan
    cargo build --bin conhub-backend --bin lexor --quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Build failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "[OK] Build completed" -ForegroundColor Green
}

Write-Host "[SERVICES] Starting all services..." -ForegroundColor Cyan
Write-Host "   Frontend: http://localhost:3000" -ForegroundColor White
Write-Host "   Backend:  http://localhost:3001" -ForegroundColor White
Write-Host "   Lexor:    http://localhost:3002" -ForegroundColor White
Write-Host ""


.\node_modules\.bin\concurrently.cmd --names "Frontend,Backend,Lexor" --prefix-colors "cyan,blue,magenta" --restart-tries 2 --kill-others-on-fail "npm run dev:frontend" "npm run dev:backend" "npm run dev:lexor"
