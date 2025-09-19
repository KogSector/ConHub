# ConHub Database Connection Test Script
# Run this script to test your PostgreSQL database connection

Write-Host "ConHub Database Connection Test" -ForegroundColor Cyan
Write-Host "===============================" -ForegroundColor Cyan

# Check if PostgreSQL is running
$postgresService = Get-Service -Name "*postgres*" -ErrorAction SilentlyContinue
if ($postgresService -and $postgresService.Status -eq "Running") {
    Write-Host "✅ PostgreSQL service is running" -ForegroundColor Green
} else {
    Write-Host "❌ PostgreSQL service is not running. Please start PostgreSQL first." -ForegroundColor Red
    exit 1
}

# Check if psql is accessible
try {
    $null = psql --version 2>$null
    Write-Host "✅ psql command is accessible" -ForegroundColor Green
} catch {
    Write-Host "❌ psql command not found. Please ensure PostgreSQL bin is in your PATH." -ForegroundColor Red
    exit 1
}

# Get password once and set environment variable
Write-Host "`nEnter your PostgreSQL password (you'll only need to enter it once):" -ForegroundColor Yellow
$password = Read-Host "Password for user 'postgres'" -AsSecureString
$plainPassword = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($password))

# Set PGPASSWORD environment variable for this session
$env:PGPASSWORD = $plainPassword

# Test if ConHub database exists
Write-Host "`nChecking if ConHub database exists..." -ForegroundColor Yellow
$dbCheck = psql -U postgres -l 2>$null | Select-String "conhub"
if ($dbCheck) {
    Write-Host "✅ ConHub database exists" -ForegroundColor Green
} else {
    Write-Host "❌ ConHub database not found. Creating it now..." -ForegroundColor Yellow
    $createResult = psql -U postgres -c "CREATE DATABASE conhub;" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ ConHub database created successfully" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to create ConHub database: $createResult" -ForegroundColor Red
        $env:PGPASSWORD = $null
        exit 1
    }
}

# Test database schema
Write-Host "`nChecking database schema..." -ForegroundColor Yellow
$tableCount = psql -U postgres -d conhub -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" 2>$null
if ($tableCount -and $tableCount.Trim() -gt 0) {
    Write-Host "✅ Database schema exists ($($tableCount.Trim()) tables found)" -ForegroundColor Green
    
    # List tables
    Write-Host "`nTables in ConHub database:" -ForegroundColor Cyan
    psql -U postgres -d conhub -c "\dt"
} else {
    Write-Host "⚠️  No tables found. Run the schema setup:" -ForegroundColor Yellow
    Write-Host "psql -U postgres -d conhub -f database/schema.sql" -ForegroundColor Cyan
}

# Get database password for .env update
$updateEnv = Read-Host "`nDo you want to update the .env file with your PostgreSQL password? (y/N)"
if ($updateEnv.ToLower() -eq 'y' -or $updateEnv.ToLower() -eq 'yes') {
    # Update .env file
    if (Test-Path ".env") {
        $envContent = Get-Content ".env" -Raw
        $newEnvContent = $envContent -replace "DATABASE_URL=postgresql://postgres:.*@localhost:5432/conhub", "DATABASE_URL=postgresql://postgres:$plainPassword@localhost:5432/conhub"
        $newEnvContent | Set-Content ".env"
        Write-Host "✅ Updated .env file with your database password" -ForegroundColor Green
    } else {
        Write-Host "❌ .env file not found" -ForegroundColor Red
    }
}

# Clear password from memory and environment
$plainPassword = $null
$env:PGPASSWORD = $null

Write-Host "`n🎉 Database test complete!" -ForegroundColor Green
Write-Host "You can now run the backend with: cargo run --bin conhub-backend" -ForegroundColor Cyan