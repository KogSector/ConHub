Write-Host "Starting ConHub..." -ForegroundColor Green

if (-not (Test-Path "package.json")) {
    Write-Host "Error: Run from project root" -ForegroundColor Red
    exit 1
}

& "$PSScriptRoot\cleanup-ports.ps1"

$backendBinary = "target\debug\conhub-backend.exe"
$lexorBinary = "target\debug\lexor.exe"

if (-not (Test-Path $backendBinary) -or -not (Test-Path $lexorBinary)) {
    Write-Host "Building binaries..." -ForegroundColor Cyan
    cargo build --bin conhub-backend --bin lexor --quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed" -ForegroundColor Red
        exit 1
    }
}

# Start services using concurrently
.\node_modules\.bin\concurrently.cmd --names "Frontend,Backend,Lexor,DocSearch,Langchain" --prefix-colors "cyan,blue,magenta,yellow,green" --restart-tries 3 "npm run dev:frontend" "npm run dev:backend" "npm run dev:lexor" "npm run dev:doc-search" "npm run dev:langchain"