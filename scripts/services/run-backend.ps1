# Run ConHub Backend
$env:RUST_LOG = "info"
$env:DATABASE_URL = "postgresql://postgres:password@localhost:5432/conhub"

# Ensure binary exists
if (-not (Test-Path "target\debug\conhub-backend.exe")) {
    Write-Host "[BUILD] Building backend..." -ForegroundColor Cyan
    cargo build --bin conhub-backend --quiet
}

Write-Host "[BACKEND] Starting on port 3001..." -ForegroundColor Blue
& "target\debug\conhub-backend.exe"