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
Write-Host "   DocSearch: http://localhost:8001" -ForegroundColor White
Write-Host "   LangChain: http://localhost:8002" -ForegroundColor White
Write-Host ""

# Start services using concurrently
.\node_modules\.bin\concurrently.cmd --names "Frontend,Backend,Lexor,DocSearch,Langchain" --prefix-colors "cyan,blue,magenta,yellow,green" --restart-tries 2 --kill-others-on-fail "npm run dev:frontend" "npm run dev:backend" "npm run dev:lexor" "npm run dev:doc-search" "npm run dev:langchain"