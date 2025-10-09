# Debug script to check database connectivity and user registration
param(
    [Parameter(Mandatory=$false)]
    [string]$BackendUrl = "http://localhost:3001"
)

$ErrorActionPreference = "Stop"

function Write-ColorOutput {
    param([string]$Message, [System.ConsoleColor]$Color = [System.ConsoleColor]::White)
    Write-Host $Message -ForegroundColor $Color
}

Write-ColorOutput "🔍 ConHub Database Debug Script" -Color Blue
Write-ColorOutput "==============================" -Color Blue
Write-Host ""

# Test 1: Backend Health Check
Write-ColorOutput "Step 1: Checking backend health..." ([System.ConsoleColor]::Yellow)
try {
    $healthResponse = Invoke-RestMethod -Uri "$BackendUrl/health" -Method Get -TimeoutSec 10
    Write-ColorOutput "✅ Backend Status: $($healthResponse.status)" ([System.ConsoleColor]::Green)
    Write-ColorOutput "✅ Database Status: $($healthResponse.database)" ([System.ConsoleColor]::Green)
    Write-Host "Timestamp: $($healthResponse.timestamp)"
    
    if ($healthResponse.database -ne "connected") {
        Write-ColorOutput "❌ Database is not connected!" ([System.ConsoleColor]::Red)
        exit 1
    }
}
catch {
    Write-ColorOutput "❌ Backend health check failed: $($_.Exception.Message)" ([System.ConsoleColor]::Red)
    exit 1
}

Write-Host ""

# Test 2: Database Structure Check
Write-ColorOutput "Step 2: Checking database structure..." ([System.ConsoleColor]::Yellow)
try {
    $dbTestResponse = Invoke-RestMethod -Uri "$BackendUrl/api/auth/test-db" -Method Get -TimeoutSec 10
    
    Write-ColorOutput "Database Connectivity:" ([System.ConsoleColor]::Cyan)
    $connectivity = $dbTestResponse.database_tests.connectivity
    Write-Host "  Status: $($connectivity.status)"
    if ($connectivity.status -eq "success") {
        Write-Host "  Test Value: $($connectivity.test_value)"
        Write-Host "  DB Time: $($connectivity.db_time)"
    } else {
        Write-Host "  Error: $($connectivity.error)"
    }
    
    Write-Host ""
    Write-ColorOutput "Users Table Structure:" ([System.ConsoleColor]::Cyan)
    $tableTest = $dbTestResponse.database_tests.users_table
    Write-Host "  Status: $($tableTest.status)"
    if ($tableTest.status -eq "success") {
        Write-Host "  Columns:"
        foreach ($col in $tableTest.columns) {
            Write-Host "    - $($col.column): $($col.type)"
        }
    } else {
        Write-Host "  Error: $($tableTest.error)"
    }
    
    Write-Host ""
    Write-ColorOutput "User Count:" ([System.ConsoleColor]::Cyan)
    $countTest = $dbTestResponse.database_tests.user_count
    Write-Host "  Status: $($countTest.status)"
    if ($countTest.status -eq "success") {
        Write-Host "  Current Users: $($countTest.user_count)"
    } else {
        Write-Host "  Error: $($countTest.error)"
    }
}
catch {
    Write-ColorOutput "❌ Database test failed: $($_.Exception.Message)" ([System.ConsoleColor]::Red)
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        Write-Host "Response: $responseBody"
    }
    exit 1
}

Write-Host ""

# Test 3: List Current Users
Write-ColorOutput "Step 3: Listing current users..." ([System.ConsoleColor]::Yellow)
try {
    $usersResponse = Invoke-RestMethod -Uri "$BackendUrl/api/auth/users" -Method Get -TimeoutSec 10
    Write-ColorOutput "✅ Current user count: $($usersResponse.count)" ([System.ConsoleColor]::Green)
    
    if ($usersResponse.count -gt 0) {
        Write-ColorOutput "Existing users:" ([System.ConsoleColor]::Cyan)
        foreach ($user in $usersResponse.users) {
            Write-Host "  - ID: $($user.id)"
            Write-Host "    Name: $($user.name)"
            Write-Host "    Email: $($user.email)"
            Write-Host "    Role: $($user.role)"
            Write-Host "    Created: $($user.created_at)"
            Write-Host ""
        }
    } else {
        Write-ColorOutput "No users found in database." ([System.ConsoleColor]::Yellow)
    }
}
catch {
    Write-ColorOutput "❌ Failed to list users: $($_.Exception.Message)" ([System.ConsoleColor]::Red)
}

Write-Host ""

# Test 4: Try to register a test user
Write-ColorOutput "Step 4: Testing user registration..." ([System.ConsoleColor]::Yellow)
$testUser = @{
    name = "Debug Test User"
    email = "debug-test@conhub.dev"
    password = "DebugTest123!"
    organization = "Debug Test Org"
}

try {
    $registerBody = $testUser | ConvertTo-Json -Depth 10
    Write-Host "Sending registration request..."
    Write-Host "Body: $registerBody"
    
    $registerResponse = Invoke-RestMethod -Uri "$BackendUrl/api/auth/register" -Method Post -Body $registerBody -ContentType "application/json" -TimeoutSec 30
    
    Write-ColorOutput "✅ User registered successfully!" ([System.ConsoleColor]::Green)
    Write-Host "User ID: $($registerResponse.user.id)"
    Write-Host "Email: $($registerResponse.user.email)"
    Write-Host "Name: $($registerResponse.user.name)"
    Write-Host "Role: $($registerResponse.user.role)"
    Write-Host "Subscription: $($registerResponse.user.subscription_tier)"
    Write-Host "Verified: $($registerResponse.user.is_verified)"
    Write-Host "Created At: $($registerResponse.user.created_at)"
}
catch {
    Write-ColorOutput "❌ Registration failed!" ([System.ConsoleColor]::Red)
    Write-Host "Error: $($_.Exception.Message)"
    
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        Write-Host "Response Body: $responseBody"
        
        try {
            $errorDetails = $responseBody | ConvertFrom-Json
            Write-Host "Error Details: $($errorDetails.error)"
            if ($errorDetails.details) {
                Write-Host "Additional Details: $($errorDetails.details)"
            }
        } catch {
            Write-Host "Could not parse error response as JSON"
        }
    }
}

Write-Host ""

# Test 5: Check users again after registration attempt
Write-ColorOutput "Step 5: Checking users after registration attempt..." ([System.ConsoleColor]::Yellow)
try {
    $usersAfterResponse = Invoke-RestMethod -Uri "$BackendUrl/api/auth/users" -Method Get -TimeoutSec 10
    Write-ColorOutput "✅ User count after registration: $($usersAfterResponse.count)" ([System.ConsoleColor]::Green)
    
    if ($usersAfterResponse.count -gt 0) {
        Write-ColorOutput "Users after registration:" ([System.ConsoleColor]::Cyan)
        foreach ($user in $usersAfterResponse.users) {
            Write-Host "  - ID: $($user.id)"
            Write-Host "    Name: $($user.name)"
            Write-Host "    Email: $($user.email)"
            Write-Host "    Role: $($user.role)"
            Write-Host "    Created: $($user.created_at)"
            Write-Host ""
        }
    } else {
        Write-ColorOutput "⚠️  Still no users found in database after registration attempt!" ([System.ConsoleColor]::Yellow)
    }
}
catch {
    Write-ColorOutput "❌ Failed to list users after registration: $($_.Exception.Message)" ([System.ConsoleColor]::Red)
}

Write-Host ""
Write-ColorOutput "🏁 Debug script completed!" ([System.ConsoleColor]::Blue)
