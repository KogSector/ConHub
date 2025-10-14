
$BackendUrl = "http://localhost:3001"

Write-Host "Testing ConHub Backend..." -ForegroundColor Blue
Write-Host ""


Write-Host "1. Checking backend health..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "$BackendUrl/health" -Method Get -TimeoutSec 5
    Write-Host "   Backend Status: $($health.status)" -ForegroundColor Green
    Write-Host "   Database Status: $($health.database)" -ForegroundColor Green
} catch {
    Write-Host "   Backend not responding: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}


Write-Host "2. Testing database operations..." -ForegroundColor Yellow
try {
    $dbTest = Invoke-RestMethod -Uri "$BackendUrl/api/auth/test-db" -Method Get -TimeoutSec 10
    $connectivity = $dbTest.database_tests.connectivity
    $userCount = $dbTest.database_tests.user_count
    
    Write-Host "   Database Connectivity: $($connectivity.status)" -ForegroundColor Green
    Write-Host "   Current User Count: $($userCount.user_count)" -ForegroundColor Green
} catch {
    Write-Host "   Database test failed: $($_.Exception.Message)" -ForegroundColor Red
}


Write-Host "3. Listing current users..." -ForegroundColor Yellow
try {
    $users = Invoke-RestMethod -Uri "$BackendUrl/api/auth/users" -Method Get -TimeoutSec 5
    Write-Host "   Users found: $($users.count)" -ForegroundColor Green
    
    if ($users.count -gt 0) {
        foreach ($user in $users.users) {
            Write-Host "   - $($user.name) ($($user.email))" -ForegroundColor Cyan
        }
    }
} catch {
    Write-Host "   Failed to list users: $($_.Exception.Message)" -ForegroundColor Red
}


Write-Host "4. Testing user registration..." -ForegroundColor Yellow
$testUser = @{
    name = "Quick Test User"
    email = "quicktest@conhub.dev"
    password = "QuickTest123!"
    organization = "Test Org"
} | ConvertTo-Json

try {
    $register = Invoke-RestMethod -Uri "$BackendUrl/api/auth/register" -Method Post -Body $testUser -ContentType "application/json" -TimeoutSec 10
    Write-Host "   Registration successful!" -ForegroundColor Green
    Write-Host "   User ID: $($register.user.id)" -ForegroundColor Cyan
    Write-Host "   Email: $($register.user.email)" -ForegroundColor Cyan
} catch {
    Write-Host "   Registration failed: $($_.Exception.Message)" -ForegroundColor Red
    
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $errorBody = $reader.ReadToEnd()
        Write-Host "   Error details: $errorBody" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "Test completed!" -ForegroundColor Blue
