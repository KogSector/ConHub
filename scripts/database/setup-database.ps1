# Setup ConHub Database
Write-Host "Setting up ConHub Database..." -ForegroundColor Blue

# Test if we can connect with postgres user
Write-Host "Testing database connection..." -ForegroundColor Yellow

try {
    # Create the conhub user and database if they don't exist
    $createUserSql = @"
-- Create user if not exists
DO `$`$
BEGIN
   IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = 'conhub') THEN
      CREATE USER conhub WITH PASSWORD 'conhub_password';
   END IF;
END
`$`$;

-- Grant privileges
ALTER USER conhub CREATEDB;
GRANT ALL PRIVILEGES ON DATABASE conhub TO conhub;
"@

    Write-Host "Creating database user and setting permissions..." -ForegroundColor Yellow
    
    # Save SQL to temp file
    $tempSqlFile = [System.IO.Path]::GetTempFileName() + ".sql"
    $createUserSql | Out-File -FilePath $tempSqlFile -Encoding UTF8
    
    # Execute SQL
    $result = psql -U postgres -d conhub -f $tempSqlFile 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Database user setup completed" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Database setup may have issues: $result" -ForegroundColor Yellow
    }
    
    # Clean up temp file
    Remove-Item $tempSqlFile -Force
    
    # Test connection with conhub user
    Write-Host "Testing connection with conhub user..." -ForegroundColor Yellow
    $testResult = psql -U conhub -d conhub -c "SELECT 1;" 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Connection test successful" -ForegroundColor Green
    } else {
        Write-Host "❌ Connection test failed: $testResult" -ForegroundColor Red
        Write-Host "You may need to run: psql -U postgres" -ForegroundColor Yellow
        Write-Host "Then execute: CREATE USER conhub WITH PASSWORD 'conhub_password'; GRANT ALL PRIVILEGES ON DATABASE conhub TO conhub;" -ForegroundColor Yellow
    }
    
} catch {
    Write-Host "❌ Database setup failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "Database setup completed!" -ForegroundColor Blue
