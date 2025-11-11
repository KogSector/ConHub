# PowerShell script to set up ConHub with NeonDB
# This script generates JWT keys, configures NeonDB, and updates environment files

param(
    [string]$NeonDbUrl = ""
)

$ErrorActionPreference = "Stop"

# ANSI color codes for PowerShell
$colors = @{
    Cyan = "`e[36m"
    Green = "`e[32m"
    Red = "`e[31m"
    Yellow = "`e[33m"
    Magenta = "`e[35m"
    Reset = "`e[0m"
    Bright = "`e[1m"
}

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "Reset"
    )
    Write-Host "$($colors[$Color])$Message$($colors.Reset)"
}

function Test-OpenSSL {
    $opensslPath = Get-Command openssl -ErrorAction SilentlyContinue
    return $null -ne $opensslPath
}

function New-JwtKeys {
    Write-ColorOutput "`n🔑 Generating JWT RSA Key Pair..." "Cyan"
    
    $projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    $keysDir = Join-Path $projectRoot "keys"
    
    # Create keys directory if it doesn't exist
    if (-not (Test-Path $keysDir)) {
        New-Item -ItemType Directory -Path $keysDir | Out-Null
        Write-ColorOutput "✅ Created keys directory" "Green"
    }
    
    # Check if keys already exist
    $privateKeyPath = Join-Path $keysDir "private_key.pem"
    $publicKeyPath = Join-Path $keysDir "public_key.pem"
    
    if ((Test-Path $privateKeyPath) -and (Test-Path $publicKeyPath)) {
        Write-ColorOutput "⚠️  JWT keys already exist. Do you want to regenerate them? (y/n)" "Yellow"
        $response = Read-Host
        if ($response -ne "y") {
            Write-ColorOutput "✅ Using existing JWT keys" "Green"
            return $true
        }
    }
    
    # Check if OpenSSL is available
    if (-not (Test-OpenSSL)) {
        Write-ColorOutput "❌ OpenSSL not found. Please install OpenSSL first." "Red"
        Write-ColorOutput "`nOptions to install OpenSSL on Windows:" "Yellow"
        Write-ColorOutput "1. Using Chocolatey: choco install openssl" "Reset"
        Write-ColorOutput "2. Using Scoop: scoop install openssl" "Reset"
        Write-ColorOutput "3. Download from: https://slproweb.com/products/Win32OpenSSL.html" "Reset"
        return $false
    }
    
    # Generate private key
    Write-ColorOutput "Generating private key..." "Yellow"
    & openssl genrsa -out $privateKeyPath 2048 2>&1 | Out-Null
    
    if ($LASTEXITCODE -ne 0) {
        Write-ColorOutput "❌ Failed to generate private key" "Red"
        return $false
    }
    Write-ColorOutput "✅ Private key generated: keys/private_key.pem" "Green"
    
    # Generate public key
    Write-ColorOutput "Generating public key..." "Yellow"
    & openssl rsa -in $privateKeyPath -pubout -out $publicKeyPath 2>&1 | Out-Null
    
    if ($LASTEXITCODE -ne 0) {
        Write-ColorOutput "❌ Failed to generate public key" "Red"
        return $false
    }
    Write-ColorOutput "✅ Public key generated: keys/public_key.pem" "Green"
    
    return $true
}

function Read-EnvFile {
    param([string]$Path)
    
    $envVars = [ordered]@{}
    if (Test-Path $Path) {
        $lines = Get-Content $Path
        $currentKey = $null
        $currentValue = ""
        $inMultiline = $false
        
        foreach ($line in $lines) {
            # Skip empty lines and comments
            if ([string]::IsNullOrWhiteSpace($line) -or $line.TrimStart().StartsWith("#")) {
                if ($null -ne $currentKey) {
                    $envVars[$currentKey] = $currentValue.TrimEnd()
                    $currentKey = $null
                    $currentValue = ""
                    $inMultiline = $false
                }
                continue
            }
            
            # Check if this is a new key=value pair
            if ($line -match '^([A-Z_][A-Z0-9_]*)=(.*)$') {
                # Save previous key if exists
                if ($null -ne $currentKey) {
                    $envVars[$currentKey] = $currentValue.TrimEnd()
                }
                
                $currentKey = $matches[1]
                $currentValue = $matches[2]
                
                # Check if value starts with quote
                if ($currentValue -match '^"') {
                    $inMultiline = $true
                    # Check if it also ends with quote on same line
                    if ($currentValue -match '"$' -and $currentValue.Length -gt 1) {
                        $inMultiline = $false
                    }
                } else {
                    $inMultiline = $false
                }
            } elseif ($inMultiline) {
                # Continue multiline value
                $currentValue += "`n" + $line
                if ($line -match '"$') {
                    $inMultiline = $false
                }
            }
        }
        
        # Save last key
        if ($null -ne $currentKey) {
            $envVars[$currentKey] = $currentValue.TrimEnd()
        }
    }
    
    return $envVars
}

function Update-RootEnvFile {
    param(
        [string]$NeonDbUrl,
        [string]$PrivateKey,
        [string]$PublicKey
    )
    
    $projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    $envPath = Join-Path $projectRoot ".env"
    $envExamplePath = Join-Path $projectRoot ".env.example"
    
    Write-ColorOutput "`n📝 Updating root .env file..." "Cyan"
    
    # Read existing .env or create from example
    if (-not (Test-Path $envPath)) {
        if (Test-Path $envExamplePath) {
            Copy-Item $envExamplePath $envPath
            Write-ColorOutput "✅ Created .env from .env.example" "Green"
        } else {
            Write-ColorOutput "❌ No .env.example found" "Red"
            return $false
        }
    }
    
    # Read .env file line by line to preserve structure
    $content = Get-Content $envPath -Raw
    
    # Update JWT keys if provided
    if ($PrivateKey) {
        # Escape special regex characters and prepare for replacement
        $privateKeyEscaped = $PrivateKey -replace '[\r\n]+', '\n'
        $content = $content -replace 'JWT_PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----[^"]*-----END RSA PRIVATE KEY-----"', "JWT_PRIVATE_KEY=`"$privateKeyEscaped`""
        $content = $content -replace 'JWT_PRIVATE_KEY=YOUR_PRIVATE_KEY_HERE', "JWT_PRIVATE_KEY=`"$privateKeyEscaped`""
        Write-ColorOutput "✅ Updated JWT_PRIVATE_KEY" "Green"
    }
    
    if ($PublicKey) {
        $publicKeyEscaped = $PublicKey -replace '[\r\n]+', '\n'
        $content = $content -replace 'JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----[^"]*-----END PUBLIC KEY-----"', "JWT_PUBLIC_KEY=`"$publicKeyEscaped`""
        $content = $content -replace 'JWT_PUBLIC_KEY=YOUR_PUBLIC_KEY_HERE', "JWT_PUBLIC_KEY=`"$publicKeyEscaped`""
        Write-ColorOutput "✅ Updated JWT_PUBLIC_KEY" "Green"
    }
    
    # Update NeonDB URL if provided
    if ($NeonDbUrl) {
        # Ensure sslmode=require for Neon
        if ($NeonDbUrl -notmatch 'sslmode=require') {
            if ($NeonDbUrl -match '\?') {
                $NeonDbUrl += '&sslmode=require'
            } else {
                $NeonDbUrl += '?sslmode=require'
            }
        }
        
        $content = $content -replace 'DATABASE_URL_NEON=.*', "DATABASE_URL_NEON=$NeonDbUrl"
        Write-ColorOutput "✅ Updated DATABASE_URL_NEON" "Green"
    }
    
    # Write back to file
    Set-Content -Path $envPath -Value $content -NoNewline
    
    return $true
}

function Sort-EnvFile {
    param([string]$Path)
    
    if (-not (Test-Path $Path)) {
        return
    }
    
    Write-ColorOutput "📋 Sorting environment variables in $Path..." "Cyan"
    
    $lines = Get-Content $Path
    $sections = @{}
    $currentSection = "HEADER"
    $sectionLines = @()
    
    foreach ($line in $lines) {
        if ($line -match '^# === (.+) ===$') {
            # Save previous section
            if ($sectionLines.Count -gt 0) {
                $sections[$currentSection] = $sectionLines
                $sectionLines = @()
            }
            # Start new section
            $currentSection = $matches[1]
            $sectionLines += $line
        } else {
            $sectionLines += $line
        }
    }
    
    # Save last section
    if ($sectionLines.Count -gt 0) {
        $sections[$currentSection] = $sectionLines
    }
    
    # Sort sections alphabetically (except HEADER)
    $sortedSections = @("HEADER") + ($sections.Keys | Where-Object { $_ -ne "HEADER" } | Sort-Object)
    
    # Rebuild file
    $newContent = @()
    foreach ($section in $sortedSections) {
        if ($sections.ContainsKey($section)) {
            $newContent += $sections[$section]
        }
    }
    
    Set-Content -Path $Path -Value $newContent
    Write-ColorOutput "✅ Sorted environment variables" "Green"
}

# Main execution
Write-ColorOutput "$($colors.Bright)$($colors.Magenta)🚀 ConHub NeonDB Setup Script$($colors.Reset)" "Reset"
Write-ColorOutput "This script will configure your ConHub installation with NeonDB" "Reset"

# Step 1: Generate JWT keys
if (-not (New-JwtKeys)) {
    Write-ColorOutput "`n❌ Failed to generate JWT keys. Please fix the errors and try again." "Red"
    exit 1
}

# Step 2: Read generated keys
$projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$privateKeyPath = Join-Path $projectRoot "keys\private_key.pem"
$publicKeyPath = Join-Path $projectRoot "keys\public_key.pem"

$privateKey = $null
$publicKey = $null

if (Test-Path $privateKeyPath) {
    $privateKey = Get-Content $privateKeyPath -Raw
}

if (Test-Path $publicKeyPath) {
    $publicKey = Get-Content $publicKeyPath -Raw
}

# Step 3: Get NeonDB URL if not provided
if ([string]::IsNullOrWhiteSpace($NeonDbUrl)) {
    Write-ColorOutput "`n🔗 NeonDB Configuration" "Cyan"
    Write-ColorOutput "Enter your NeonDB connection string (or press Enter to skip):" "Yellow"
    Write-ColorOutput "Example: postgresql://user:password@ep-xxxx.region.neon.tech/dbname" "Reset"
    $NeonDbUrl = Read-Host
}

# Step 4: Update root .env file
if (-not (Update-RootEnvFile -NeonDbUrl $NeonDbUrl -PrivateKey $privateKey -PublicKey $publicKey)) {
    Write-ColorOutput "`n❌ Failed to update environment file" "Red"
    exit 1
}

# Step 5: Sort environment file
$envPath = Join-Path $projectRoot ".env"
Sort-EnvFile -Path $envPath

# Step 6: Update feature toggles to ensure Auth is enabled
$togglesPath = Join-Path $projectRoot "feature-toggles.json"
if (Test-Path $togglesPath) {
    $toggles = Get-Content $togglesPath | ConvertFrom-Json
    if ($toggles.Auth -ne $true) {
        $toggles.Auth = $true
        $toggles | ConvertTo-Json -Depth 10 | Set-Content $togglesPath
        Write-ColorOutput "`n✅ Enabled Auth feature toggle" "Green"
    }
}

# Summary
Write-ColorOutput "`n$($colors.Bright)$($colors.Green)🎉 Setup Complete!$($colors.Reset)" "Reset"
Write-ColorOutput "`nConfiguration Summary:" "Cyan"
Write-ColorOutput "✅ JWT keys generated and configured" "Green"

if ($NeonDbUrl) {
    Write-ColorOutput "✅ NeonDB connection configured" "Green"
} else {
    Write-ColorOutput "⚠️  NeonDB URL not provided - using local PostgreSQL" "Yellow"
}

Write-ColorOutput "✅ Environment variables sorted" "Green"
Write-ColorOutput "✅ Auth feature enabled" "Green"

Write-ColorOutput "`nNext Steps:" "Cyan"
Write-ColorOutput "1. Review your .env file and update any other necessary values" "Reset"
Write-ColorOutput "2. Ensure Redis is running (required for Auth service)" "Reset"
Write-ColorOutput "3. Ensure Qdrant is running (required for Data service)" "Reset"
Write-ColorOutput "4. Run 'npm start' to start all services" "Reset"

Write-ColorOutput "`n⚠️  IMPORTANT: Keep your .env file and keys secure!" "Yellow"
Write-ColorOutput "   Never commit .env files or keys to version control" "Yellow"
