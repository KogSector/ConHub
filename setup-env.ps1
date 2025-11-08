# PowerShell script to set up environment files for all microservices
# This script copies .env.example to .env and populates JWT keys

Write-Host "🔧 Setting up environment files for all microservices..." -ForegroundColor Cyan
Write-Host ""

# List of microservices
$services = @("auth", "billing", "client", "data", "security", "webhook")

# Check if JWT keys exist
$privateKeyPath = "keys/private_key.pem"
$publicKeyPath = "keys/public_key.pem"

$privateKey = $null
$publicKey = $null

if (Test-Path $privateKeyPath) {
    $privateKey = Get-Content $privateKeyPath -Raw
    Write-Host "✅ Found private key" -ForegroundColor Green
} else {
    Write-Host "⚠️  Private key not found. Run generate-jwt-keys.ps1 first." -ForegroundColor Yellow
}

if (Test-Path $publicKeyPath) {
    $publicKey = Get-Content $publicKeyPath -Raw
    Write-Host "✅ Found public key" -ForegroundColor Green
} else {
    Write-Host "⚠️  Public key not found. Run generate-jwt-keys.ps1 first." -ForegroundColor Yellow
}

Write-Host ""

# Function to create .env file from .env.example
function Initialize-ServiceEnv {
    param (
        [string]$ServiceName
    )
    
    $examplePath = "$ServiceName\.env.example"
    $envPath = "$ServiceName\.env"
    
    if (-not (Test-Path $examplePath)) {
        Write-Host "⚠️  $ServiceName/.env.example not found, skipping..." -ForegroundColor Yellow
        return
    }
    
    # Copy .env.example to .env
    Copy-Item $examplePath $envPath -Force
    Write-Host "📝 Created $ServiceName/.env" -ForegroundColor Cyan
    
    # Read the content
    $content = Get-Content $envPath -Raw
    
    # Replace JWT keys if available
    if ($null -ne $privateKey -and $ServiceName -eq "auth") {
        # Auth service needs both private and public keys
        $content = $content -replace 'JWT_PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----\s*YOUR_PRIVATE_KEY_HERE\s*-----END RSA PRIVATE KEY-----"', "JWT_PRIVATE_KEY=`"$($privateKey.Trim())`""
        $content = $content -replace 'JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----\s*YOUR_PUBLIC_KEY_HERE\s*-----END PUBLIC KEY-----"', "JWT_PUBLIC_KEY=`"$($publicKey.Trim())`""
        Write-Host "  ✅ Populated JWT keys (private + public)" -ForegroundColor Green
    } elseif ($null -ne $publicKey) {
        # Other services only need public key
        $content = $content -replace 'JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----\s*YOUR_PUBLIC_KEY_HERE\s*-----END PUBLIC KEY-----"', "JWT_PUBLIC_KEY=`"$($publicKey.Trim())`""
        Write-Host "  ✅ Populated JWT public key" -ForegroundColor Green
    }
    
    # Write back to file
    Set-Content $envPath $content -NoNewline
}

# Set up each service
foreach ($service in $services) {
    Initialize-ServiceEnv -ServiceName $service
}

# Set up frontend
if (Test-Path "frontend\.env.example") {
    Copy-Item "frontend\.env.example" "frontend\.env.local" -Force
    Write-Host "📝 Created frontend/.env.local" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "🎉 Environment setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Review and update the .env files with your actual values" -ForegroundColor White
Write-Host "2. Make sure PostgreSQL is running on localhost:5432" -ForegroundColor White
Write-Host "3. Make sure Redis is running on localhost:6379" -ForegroundColor White
Write-Host "4. Make sure Qdrant is running on localhost:6333 (for AI/Data services)" -ForegroundColor White
Write-Host "5. Run 'npm start' to start all services" -ForegroundColor White
Write-Host ""
Write-Host "⚠️  IMPORTANT: Never commit .env files to git!" -ForegroundColor Yellow
