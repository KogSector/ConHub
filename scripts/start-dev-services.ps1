# ConHub Development Services Startup Script
# This script starts all required backend services for local development

param(
    [switch]$Auth,
    [switch]$Data,
    [switch]$Chunker,
    [switch]$VectorRag,
    [switch]$All
)

$ErrorActionPreference = "Continue"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

# If no specific service is selected, start all
if (-not ($Auth -or $Data -or $Chunker -or $VectorRag)) {
    $All = $true
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  ConHub Development Services Startup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Function to start a service in a new terminal
function Start-Service {
    param(
        [string]$Name,
        [string]$Path,
        [string]$Port
    )
    
    Write-Host "Starting $Name on port $Port..." -ForegroundColor Yellow
    
    $servicePath = Join-Path $RepoRoot $Path
    
    if (-not (Test-Path $servicePath)) {
        Write-Host "  ERROR: Path not found: $servicePath" -ForegroundColor Red
        return
    }
    
    # Start in a new PowerShell window
    Start-Process pwsh -ArgumentList "-NoExit", "-Command", "cd '$servicePath'; Write-Host 'Starting $Name...' -ForegroundColor Green; cargo run" -WindowStyle Normal
    
    Write-Host "  Started $Name in new terminal" -ForegroundColor Green
}

# Start services based on flags
if ($All -or $Auth) {
    Start-Service -Name "Auth Service" -Path "auth" -Port "3010"
    Start-Sleep -Seconds 2
}

if ($All -or $VectorRag) {
    Start-Service -Name "Vector RAG (Embedding)" -Path "vector_rag" -Port "8082"
    Start-Sleep -Seconds 2
}

if ($All -or $Chunker) {
    Start-Service -Name "Chunker Service" -Path "chunker" -Port "3017"
    Start-Sleep -Seconds 2
}

if ($All -or $Data) {
    Start-Service -Name "Data Service" -Path "data" -Port "3013"
    Start-Sleep -Seconds 2
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Services Starting..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Service Ports:" -ForegroundColor White
Write-Host "  Auth Service:     http://localhost:3010" -ForegroundColor Gray
Write-Host "  Vector RAG:       http://localhost:8082" -ForegroundColor Gray
Write-Host "  Chunker Service:  http://localhost:3017" -ForegroundColor Gray
Write-Host "  Data Service:     http://localhost:3013" -ForegroundColor Gray
Write-Host ""
Write-Host "Frontend (start separately):" -ForegroundColor White
Write-Host "  cd frontend && npm run dev" -ForegroundColor Gray
Write-Host "  http://localhost:3000" -ForegroundColor Gray
Write-Host ""
Write-Host "To verify services are running:" -ForegroundColor White
Write-Host "  curl http://localhost:3010/health" -ForegroundColor Gray
Write-Host "  curl http://localhost:8082/health" -ForegroundColor Gray
Write-Host "  curl http://localhost:3017/health" -ForegroundColor Gray
Write-Host "  curl http://localhost:3013/health" -ForegroundColor Gray
Write-Host ""
