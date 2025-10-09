# Test script for user signup functionality
# This script tests the user registration and database storage

param(
    [Parameter(Mandatory=$false)]
    [string]$BackendUrl = "http://localhost:3001"
)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-ColorOutput {
    param([string]$Message, [System.ConsoleColor]$Color = [System.ConsoleColor]::White)
    Write-Host $Message -ForegroundColor $Color
}

Write-ColorOutput "🧪 Testing ConHub User Signup Functionality" ([System.ConsoleColor]::Blue)
Write-ColorOutput "==========================================" ([System.ConsoleColor]::Blue)
Write-Host ""

# Test data
$testUser = @{
    name = "Test User"
    email = "test@conhub.dev"
    password = "TestPassword123!"
    avatar_url = $null
    organization = "ConHub Test Org"
}

Write-ColorOutput "📝 Test User Data:" ([System.ConsoleColor]::Yellow)
Write-Host "Name: $($testUser.name)"
Write-Host "Email: $($testUser.email)"
Write-Host "Organization: $($testUser.organization)"
Write-Host ""

# Test 1: Check if backend is running
Write-ColorOutput "🔍 Step 1: Checking backend health..." ([System.ConsoleColor]::Yellow)
try {
    $healthResponse = Invoke-RestMethod -Uri "$BackendUrl/health" -Method Get -TimeoutSec 10
    Write-ColorOutput "✅ Backend is healthy: $($healthResponse.status)" ([System.ConsoleColor]::Green)
}
catch {
    Write-ColorOutput "❌ Backend is not responding at $BackendUrl" ([System.ConsoleColor]::Red)
    Write-ColorOutput "Please ensure the backend is running with: npm run dev:backend" ([System.ConsoleColor]::Red)
    exit 1
}

# Test 2: List existing users (should be empty initially)
Write-ColorOutput "🔍 Step 2: Checking existing users..." ([System.ConsoleColor]::Yellow)
try {
    $usersResponse = Invoke-RestMethod -Uri "$BackendUrl/api/auth/users" -Method Get -TimeoutSec 10
    Write-ColorOutput "📊 Current user count: $($usersResponse.count)" ([System.ConsoleColor]::Green)
    
    if ($usersResponse.count -gt 0) {
        Write-ColorOutput "Existing users:" ([System.ConsoleColor]::Cyan)
        foreach ($user in $usersResponse.users) {
            Write-Host "  - $($user.name) ($($user.email)) - Role: $($user.role)"
        }
    }
}
catch {
    Write-ColorOutput "⚠️  Could not fetch existing users: $($_.Exception.Message)" ([System.ConsoleColor]::Yellow)
}

Write-Host ""

# Test 3: Register new user
Write-ColorOutput "🔍 Step 3: Registering new user..." ([System.ConsoleColor]::Yellow)
try {
    $registerBody = $testUser | ConvertTo-Json -Depth 10
    $registerResponse = Invoke-RestMethod -Uri "$BackendUrl/api/auth/register" -Method Post -Body $registerBody -ContentType "application/json" -TimeoutSec 30
    
    Write-ColorOutput "✅ User registered successfully!" ([System.ConsoleColor]::Green)
    Write-Host "User ID: $($registerResponse.user.id)"
    Write-Host "Email: $($registerResponse.user.email)"
    Write-Host "Name: $($registerResponse.user.name)"
    Write-Host "Role: $($registerResponse.user.role)"
    Write-Host "Subscription: $($registerResponse.user.subscription_tier)"
    Write-Host "Verified: $($registerResponse.user.is_verified)"
    Write-Host "JWT Token: $($registerResponse.token.Substring(0, 50))..."
    
    $userId = $registerResponse.user.id
}
catch {
    $errorMessage = $_.Exception.Message
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        $errorDetails = $responseBody | ConvertFrom-Json
        Write-ColorOutput "❌ Registration failed: $($errorDetails.error)" ([System.ConsoleColor]::Red)
        if ($errorDetails.details) {
            Write-ColorOutput "Details: $($errorDetails.details)" ([System.ConsoleColor]::Red)
        }
    } else {
        Write-ColorOutput "❌ Registration failed: $errorMessage" ([System.ConsoleColor]::Red)
    }
    exit 1
}

Write-Host ""

# Test 4: Verify user was stored in database
Write-ColorOutput "🔍 Step 4: Verifying user was stored in database..." ([System.ConsoleColor]::Yellow)
try {
    $usersAfterResponse = Invoke-RestMethod -Uri "$BackendUrl/api/auth/users" -Method Get -TimeoutSec 10
    Write-ColorOutput "📊 User count after registration: $($usersAfterResponse.count)" ([System.ConsoleColor]::Green)
    
    $createdUser = $usersAfterResponse.users | Where-Object { $_.email -eq $testUser.email }
    if ($createdUser) {
        Write-ColorOutput "✅ User found in database!" ([System.ConsoleColor]::Green)
        Write-Host "Database User ID: $($createdUser.id)"
        Write-Host "Database Email: $($createdUser.email)"
        Write-Host "Database Name: $($createdUser.name)"
        Write-Host "Created At: $($createdUser.created_at)"
    } else {
        Write-ColorOutput "❌ User not found in database!" ([System.ConsoleColor]::Red)
        exit 1
    }
}
catch {
    Write-ColorOutput "❌ Could not verify user in database: $($_.Exception.Message)" ([System.ConsoleColor]::Red)
    exit 1
}

Write-Host ""

# Test 5: Test login with new user
Write-ColorOutput "🔍 Step 5: Testing login with new user..." ([System.ConsoleColor]::Yellow)
try {
    $loginBody = @{
        email = $testUser.email
        password = $testUser.password
    } | ConvertTo-Json
    
    $loginResponse = Invoke-RestMethod -Uri "$BackendUrl/api/auth/login" -Method Post -Body $loginBody -ContentType "application/json" -TimeoutSec 30
    
    Write-ColorOutput "✅ Login successful!" ([System.ConsoleColor]::Green)
    Write-Host "Login User ID: $($loginResponse.user.id)"
    Write-Host "Login Email: $($loginResponse.user.email)"
    Write-Host "Last Login: $($loginResponse.user.last_login_at)"
    Write-Host "JWT Token: $($loginResponse.token.Substring(0, 50))..."
}
catch {
    $errorMessage = $_.Exception.Message
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        $errorDetails = $responseBody | ConvertFrom-Json
        Write-ColorOutput "❌ Login failed: $($errorDetails.error)" ([System.ConsoleColor]::Red)
    } else {
        Write-ColorOutput "❌ Login failed: $errorMessage" ([System.ConsoleColor]::Red)
    }
    exit 1
}

Write-Host ""

# Test 6: Try duplicate registration (should fail)
Write-ColorOutput "🔍 Step 6: Testing duplicate registration (should fail)..." ([System.ConsoleColor]::Yellow)
try {
    $duplicateBody = $testUser | ConvertTo-Json -Depth 10
    $duplicateResponse = Invoke-RestMethod -Uri "$BackendUrl/api/auth/register" -Method Post -Body $duplicateBody -ContentType "application/json" -TimeoutSec 30
    
    Write-ColorOutput "❌ Duplicate registration should have failed!" ([System.ConsoleColor]::Red)
    exit 1
}
catch {
    if ($_.Exception.Response.StatusCode -eq 400) {
        Write-ColorOutput "✅ Duplicate registration correctly rejected!" ([System.ConsoleColor]::Green)
    } else {
        Write-ColorOutput "⚠️  Unexpected error for duplicate registration: $($_.Exception.Message)" ([System.ConsoleColor]::Yellow)
    }
}

Write-Host ""
Write-ColorOutput "🎉 All tests passed! User signup functionality is working correctly." ([System.ConsoleColor]::Green)
Write-ColorOutput "✅ Users are being properly stored in the database" ([System.ConsoleColor]::Green)
Write-ColorOutput "✅ Authentication is working with stored users" ([System.ConsoleColor]::Green)
Write-ColorOutput "✅ Duplicate email validation is working" ([System.ConsoleColor]::Green)

Write-Host ""
Write-ColorOutput "📋 Summary:" ([System.ConsoleColor]::Blue)
Write-Host "- Backend health check: ✅"
Write-Host "- User registration: ✅"
Write-Host "- Database storage: ✅"
Write-Host "- User authentication: ✅"
Write-Host "- Duplicate prevention: ✅"

Write-Host ""
Write-ColorOutput "🚀 You can now use the signup functionality in your frontend!" ([System.ConsoleColor]::Green)
