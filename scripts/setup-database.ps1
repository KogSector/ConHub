# ConHub Database Setup Script
# This script creates the ConHub database and applies the schema

Write-Host "ConHub Database Setup" -ForegroundColor Cyan
Write-Host "=====================" -ForegroundColor Cyan

# Check if PostgreSQL is running
$postgresService = Get-Service -Name "*postgres*" -ErrorAction SilentlyContinue
if (-not $postgresService -or $postgresService.Status -ne "Running") {
    Write-Host "❌ PostgreSQL service is not running. Please start PostgreSQL first." -ForegroundColor Red
    exit 1
}

Write-Host "✅ PostgreSQL service is running" -ForegroundColor Green

# Get password once and set environment variable
Write-Host "`nEnter your PostgreSQL password (you'll only need to enter it once):" -ForegroundColor Yellow
$password = Read-Host "Password for user 'postgres'" -AsSecureString
$plainPassword = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($password))

# Set PGPASSWORD environment variable for this session
$env:PGPASSWORD = $plainPassword

# Create database if it doesn't exist
Write-Host "`nCreating ConHub database..." -ForegroundColor Yellow
$createResult = psql -U postgres -c "SELECT 1 FROM pg_database WHERE datname = 'conhub'" 2>&1
if ($createResult -like "*1*") {
    Write-Host "✅ ConHub database already exists" -ForegroundColor Green
} else {
    $createResult = psql -U postgres -c "CREATE DATABASE conhub;" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ ConHub database created successfully" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to create database: $createResult" -ForegroundColor Red
        $env:PGPASSWORD = $null
        exit 1
    }
}

# Apply schema
if (Test-Path "database\schema.sql") {
    Write-Host "`nApplying database schema..." -ForegroundColor Yellow
    $schemaResult = psql -U postgres -d conhub -f "database\schema.sql" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Database schema applied successfully" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to apply schema: $schemaResult" -ForegroundColor Red
        $env:PGPASSWORD = $null
        exit 1
    }
} else {
    Write-Host "❌ Schema file not found: database\schema.sql" -ForegroundColor Red
    $env:PGPASSWORD = $null
    exit 1
}

# Verify setup
Write-Host "`nVerifying database setup..." -ForegroundColor Yellow
$tableCount = psql -U postgres -d conhub -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" 2>$null
if ($tableCount -and $tableCount.Trim() -gt 0) {
    Write-Host "✅ Database setup complete! Found $($tableCount.Trim()) tables" -ForegroundColor Green
    
    # Show tables
    Write-Host "`nCreated tables:" -ForegroundColor Cyan
    psql -U postgres -d conhub -c "\dt"
} else {
    Write-Host "❌ Database setup verification failed" -ForegroundColor Red
    $env:PGPASSWORD = $null
    exit 1
}

# Clear password from memory and environment
$plainPassword = $null
$env:PGPASSWORD = $null

Write-Host "`n🎉 ConHub database is ready!" -ForegroundColor Green
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Update your .env file with the correct PostgreSQL password" -ForegroundColor White
Write-Host "2. Run: .\scripts\test-database.ps1 to test the connection" -ForegroundColor White
Write-Host "3. Start the backend: cargo run --bin conhub-backend" -ForegroundColor White