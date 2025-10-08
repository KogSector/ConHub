# Clear ConHub PostgreSQL Database
# WARNING: This will delete ALL data from the database

param(
    [string]$DatabaseUrl = "postgresql://localhost:5432/conhub",
    [string]$DatabaseName = "conhub",
    [string]$Username = "postgres",
    [string]$Password = "",
    [switch]$Confirm
)

Write-Host "[DATABASE] Clearing ConHub database..." -ForegroundColor Yellow

if (-not $Confirm) {
    $response = Read-Host "Are you sure you want to delete ALL data? This cannot be undone! (type 'yes' to confirm)"
    if ($response -ne "yes") {
        Write-Host "[CANCELLED] Database clearing cancelled by user" -ForegroundColor Red
        exit 0
    }
}

# Check if psql is available
if (-not (Get-Command psql -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] PostgreSQL client (psql) not found. Please install PostgreSQL or add it to PATH." -ForegroundColor Red
    exit 1
}

# Build connection string
$env:PGPASSWORD = $Password

try {
    Write-Host "[EXECUTING] Running database cleanup script..." -ForegroundColor Cyan
    
    # Execute the SQL script
    psql -h localhost -p 5432 -U $Username -d $DatabaseName -f "$PSScriptRoot\clear-database.sql"
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[SUCCESS] Database cleared successfully!" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] Failed to clear database" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "[ERROR] Database operation failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
} finally {
    # Clear password from environment
    $env:PGPASSWORD = ""
}

Write-Host "[COMPLETE] Database cleanup completed" -ForegroundColor Green
