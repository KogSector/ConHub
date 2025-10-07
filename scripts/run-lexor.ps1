# Run Lexor Service
$env:RUST_LOG = "info"

# Ensure binary exists
if (-not (Test-Path "target\debug\lexor.exe")) {
    Write-Host "[BUILD] Building lexor..." -ForegroundColor Cyan
    cargo build --bin lexor --quiet
}

Write-Host "[LEXOR] Starting on port 3002..." -ForegroundColor Magenta
& "target\debug\lexor.exe"