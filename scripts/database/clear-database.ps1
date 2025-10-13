


param(
    [string]$DatabaseUrl = "postgresql://localhost:5432/conhub",
    [string]$DatabaseName = "conhub",
    [string]$Username = "postgres",
    [string]$Password = "",
    [string]$DatabaseHost = "localhost", # Renamed from 'Host' to avoid conflict with the built-in PowerShell automatic variable
    [string]$Port = "5432",
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


if (-not (Get-Command psql -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] PostgreSQL client (psql) not found. Please install PostgreSQL or add it to PATH." -ForegroundColor Red
    exit 1
}


$env:PGPASSWORD = $Password

try {
    Write-Host "[EXECUTING] Running database cleanup script..." -ForegroundColor Cyan
    Write-Host "[CONNECTION] Connecting to: ${DatabaseHost}:$Port as user: $Username" -ForegroundColor Yellow
    
    # Execute the SQL script
    psql -h $DatabaseHost -p $Port -U $Username -d $DatabaseName -f "$PSScriptRoot\clear-database.sql"
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
