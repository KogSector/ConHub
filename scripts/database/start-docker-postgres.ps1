# Start PostgreSQL using Docker
# This creates a fresh PostgreSQL instance with known credentials

param(
    [string]$Password = "postgres",
    [string]$Database = "conhub",
    [int]$Port = 5432
)

Write-Host "[DOCKER] Starting PostgreSQL with Docker..." -ForegroundColor Cyan

# Check if Docker is available
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] Docker not found. Please install Docker Desktop." -ForegroundColor Red
    exit 1
}

# Stop existing container if running
Write-Host "[CLEANUP] Stopping existing PostgreSQL container..." -ForegroundColor Yellow
docker stop conhub-postgres 2>$null
docker rm conhub-postgres 2>$null

# Start new PostgreSQL container
Write-Host "[STARTING] Starting PostgreSQL container..." -ForegroundColor Green
docker run -d `
    --name conhub-postgres `
    -e POSTGRES_PASSWORD=$Password `
    -e POSTGRES_DB=$Database `
    -p $Port:5432 `
    postgres:15

if ($LASTEXITCODE -eq 0) {
    Write-Host "[SUCCESS] PostgreSQL started successfully!" -ForegroundColor Green
    Write-Host "Connection details:" -ForegroundColor Cyan
    Write-Host "  Host: localhost" -ForegroundColor White
    Write-Host "  Port: $Port" -ForegroundColor White
    Write-Host "  Database: $Database" -ForegroundColor White
    Write-Host "  Username: postgres" -ForegroundColor White
    Write-Host "  Password: $Password" -ForegroundColor White
    Write-Host ""
    Write-Host "Now you can run:" -ForegroundColor Green
    Write-Host "npm run db:clear -Password '$Password'" -ForegroundColor White
    Write-Host ""
    Write-Host "To stop the container later:" -ForegroundColor Yellow
    Write-Host "docker stop conhub-postgres" -ForegroundColor White
} else {
    Write-Host "[ERROR] Failed to start PostgreSQL container" -ForegroundColor Red
}
