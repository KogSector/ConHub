# Test PostgreSQL Connection
# This script helps you test different connection parameters

param(
    [string]$Username = "postgres",
    [string]$Password = "",
    [string]$DbHost = "localhost",
    [string]$Port = "5432",
    [string]$Database = "postgres"
)

Write-Host "[TEST] Testing PostgreSQL Connection" -ForegroundColor Cyan
Write-Host "Host: $Host" -ForegroundColor White
Write-Host "Port: $Port" -ForegroundColor White
Write-Host "Username: $Username" -ForegroundColor White
Write-Host "Database: $Database" -ForegroundColor White
Write-Host ""

# Set password environment variable
$env:PGPASSWORD = $Password

try {
    Write-Host "[TESTING] Attempting connection..." -ForegroundColor Yellow
    
    # Test connection
    $result = psql -h $Host -p $Port -U $Username -d $Database -c "SELECT version();" 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[SUCCESS] Connection successful!" -ForegroundColor Green
        Write-Host $result -ForegroundColor White
        Write-Host ""
        Write-Host "You can now use these parameters:" -ForegroundColor Cyan
        Write-Host "npm run db:clear -Username '$Username' -Password '$Password'" -ForegroundColor White
    } else {
        Write-Host "[FAILED] Connection failed" -ForegroundColor Red
        Write-Host $result -ForegroundColor Red
        Write-Host ""
        Write-Host "Try these solutions:" -ForegroundColor Yellow
        Write-Host "1. Check if PostgreSQL service is running" -ForegroundColor White
        Write-Host "2. Try different username (postgres, admin, your_username)" -ForegroundColor White
        Write-Host "3. Try empty password: npm run db:clear -Password ''" -ForegroundColor White
        Write-Host "4. Use Docker: npm run db:docker" -ForegroundColor White
    }
} catch {
    Write-Host "[ERROR] Connection test failed: $($_.Exception.Message)" -ForegroundColor Red
} finally {
    # Clear password from environment
    $env:PGPASSWORD = ""
}
