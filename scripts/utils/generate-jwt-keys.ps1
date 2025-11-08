# PowerShell script to generate JWT RSA keys
# This script generates RSA key pair for JWT authentication

Write-Host "🔑 Generating JWT RSA Key Pair..." -ForegroundColor Cyan

# Check if keys directory exists
if (-not (Test-Path "keys")) {
    New-Item -ItemType Directory -Path "keys" | Out-Null
    Write-Host "✅ Created keys directory" -ForegroundColor Green
}

# Check if OpenSSL is available
$opensslPath = Get-Command openssl -ErrorAction SilentlyContinue

if ($null -eq $opensslPath) {
    Write-Host "❌ OpenSSL not found. Please install OpenSSL first." -ForegroundColor Red
    Write-Host ""
    Write-Host "Options to install OpenSSL on Windows:" -ForegroundColor Yellow
    Write-Host "1. Using Chocolatey: choco install openssl" -ForegroundColor White
    Write-Host "2. Using Scoop: scoop install openssl" -ForegroundColor White
    Write-Host "3. Download from: https://slproweb.com/products/Win32OpenSSL.html" -ForegroundColor White
    Write-Host ""
    Write-Host "Alternatively, you can use online tools:" -ForegroundColor Yellow
    Write-Host "- https://cryptotools.net/rsagen" -ForegroundColor White
    Write-Host "- https://travistidwell.com/jsencrypt/demo/" -ForegroundColor White
    Write-Host ""
    Write-Host "After generating keys, save them as:" -ForegroundColor Yellow
    Write-Host "  - keys/private_key.pem (Private Key)" -ForegroundColor White
    Write-Host "  - keys/public_key.pem (Public Key)" -ForegroundColor White
    exit 1
}

# Generate private key
Write-Host "Generating private key..." -ForegroundColor Yellow
& openssl genrsa -out keys/private_key.pem 2048 2>&1 | Out-Null

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Private key generated: keys/private_key.pem" -ForegroundColor Green
} else {
    Write-Host "❌ Failed to generate private key" -ForegroundColor Red
    exit 1
}

# Generate public key
Write-Host "Generating public key..." -ForegroundColor Yellow
& openssl rsa -in keys/private_key.pem -pubout -out keys/public_key.pem 2>&1 | Out-Null

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Public key generated: keys/public_key.pem" -ForegroundColor Green
} else {
    Write-Host "❌ Failed to generate public key" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎉 JWT RSA Key Pair generated successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Copy .env.example to .env in each microservice folder" -ForegroundColor White
Write-Host "2. Run the setup-env.ps1 script to populate .env files with the keys" -ForegroundColor White
Write-Host ""
Write-Host "⚠️  IMPORTANT: Keep these keys secure and never commit them to git!" -ForegroundColor Yellow
