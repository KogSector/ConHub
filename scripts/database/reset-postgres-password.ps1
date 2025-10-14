


param(
    [string]$Username = "postgres"
)

Write-Host "[POSTGRES] PostgreSQL Password Reset Helper" -ForegroundColor Cyan
Write-Host ""


$pgPath = Get-Command psql -ErrorAction SilentlyContinue
if (-not $pgPath) {
    Write-Host "[ERROR] PostgreSQL not found in PATH. Please ensure PostgreSQL is installed." -ForegroundColor Red
    Write-Host "Common installation paths:" -ForegroundColor Yellow
    Write-Host "  - C:\Program Files\PostgreSQL\15\bin" -ForegroundColor White
    Write-Host "  - C:\Program Files\PostgreSQL\14\bin" -ForegroundColor White
    Write-Host "  - C:\Program Files\PostgreSQL\13\bin" -ForegroundColor White
    exit 1
}

Write-Host "[INFO] PostgreSQL found at: $($pgPath.Source)" -ForegroundColor Green


$pgService = Get-Service -Name "postgresql*" -ErrorAction SilentlyContinue
if ($pgService) {
    Write-Host "[INFO] PostgreSQL service status: $($pgService.Status)" -ForegroundColor Green
} else {
    Write-Host "[WARNING] Could not find PostgreSQL service" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Choose an option:" -ForegroundColor Cyan
Write-Host "1. Try common default passwords" -ForegroundColor White
Write-Host "2. Reset password using pg_hba.conf (requires service restart)" -ForegroundColor White
Write-Host "3. Connect without password (trust method)" -ForegroundColor White
Write-Host "4. Use different connection method" -ForegroundColor White
Write-Host ""

$choice = Read-Host "Enter your choice (1-4)"

switch ($choice) {
    "1" {
        Write-Host "[TESTING] Trying common default passwords..." -ForegroundColor Cyan
        
        $commonPasswords = @("", "postgres", "admin", "password", "root", "123456")
        
        foreach ($password in $commonPasswords) {
            Write-Host "Trying password: '$password'" -ForegroundColor Yellow
            
            $env:PGPASSWORD = $password
            $result = psql -h localhost -U $Username -d postgres -c "SELECT version();" 2>$null
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "[SUCCESS] Password found: '$password'" -ForegroundColor Green
                Write-Host "You can now use: npm run db:clear -Password '$password'" -ForegroundColor Green
                $env:PGPASSWORD = ""
                exit 0
            }
        }
        
        Write-Host "[FAILED] None of the common passwords worked" -ForegroundColor Red
        $env:PGPASSWORD = ""
    }
    
    "2" {
        Write-Host "[INFO] To reset password using pg_hba.conf:" -ForegroundColor Cyan
        Write-Host "1. Stop PostgreSQL service" -ForegroundColor White
        Write-Host "2. Edit pg_hba.conf and change 'md5' to 'trust' for local connections" -ForegroundColor White
        Write-Host "3. Start PostgreSQL service" -ForegroundColor White
        Write-Host "4. Connect and change password: ALTER USER postgres PASSWORD 'newpassword';" -ForegroundColor White
        Write-Host "5. Change pg_hba.conf back to 'md5'" -ForegroundColor White
        Write-Host "6. Restart service" -ForegroundColor White
        Write-Host ""
        Write-Host "pg_hba.conf location: C:\Program Files\PostgreSQL\[version]\data\pg_hba.conf" -ForegroundColor Yellow
    }
    
    "3" {
        Write-Host "[TESTING] Trying to connect without password..." -ForegroundColor Cyan
        
        $result = psql -h localhost -U $Username -d postgres -c "SELECT version();" 2>$null
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[SUCCESS] Connected without password!" -ForegroundColor Green
            Write-Host "You can now run: npm run db:clear -Password ''" -ForegroundColor Green
        } else {
            Write-Host "[FAILED] Cannot connect without password" -ForegroundColor Red
        }
    }
    
    "4" {
        Write-Host "[INFO] Alternative connection methods:" -ForegroundColor Cyan
        Write-Host "1. Use pgAdmin (GUI tool)" -ForegroundColor White
        Write-Host "2. Use different username (try: postgres, admin, root)" -ForegroundColor White
        Write-Host "3. Check if PostgreSQL is running on different port (5433, 5434, etc.)" -ForegroundColor White
        Write-Host "4. Use Docker PostgreSQL: docker run -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres" -ForegroundColor White
    }
    
    default {
        Write-Host "[ERROR] Invalid choice" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "[TIP] You can also try:" -ForegroundColor Cyan
Write-Host "npm run db:clear -Username 'your_username' -Password 'your_password'" -ForegroundColor White
