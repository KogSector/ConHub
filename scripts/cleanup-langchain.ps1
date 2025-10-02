# ConHub - Cleanup LangChain Service (Phase 3 of Architecture Refactoring)
Write-Host "🧹 Cleaning up LangChain Service..." -ForegroundColor Yellow

# Check if we're in the right directory
if (-not (Test-Path "package.json")) {
    Write-Host "Error: Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

# Stop any running langchain processes
Write-Host "🛑 Stopping LangChain service processes..." -ForegroundColor Cyan
Get-Process -Name "node" -ErrorAction SilentlyContinue | Where-Object { 
    $_.CommandLine -like "*langchain-service*" -or 
    $_.CommandLine -like "*ts-node*langchain*" 
} | Stop-Process -Force -ErrorAction SilentlyContinue

# Remove langchain-service directory
if (Test-Path "langchain-service") {
    Write-Host "📁 Removing langchain-service directory..." -ForegroundColor Yellow
    Remove-Item -Path "langchain-service" -Recurse -Force
    Write-Host "✅ Removed langchain-service directory" -ForegroundColor Green
} else {
    Write-Host "ℹ️  langchain-service directory not found" -ForegroundColor Blue
}

# Update package.json scripts
Write-Host "📝 Updating package.json scripts..." -ForegroundColor Cyan
$packageJsonPath = "package.json"
if (Test-Path $packageJsonPath) {
    $packageJson = Get-Content $packageJsonPath -Raw | ConvertFrom-Json
    
    # Remove langchain-related scripts
    $scriptsToRemove = @(
        "dev:langchain",
        "start:langchain", 
        "build:langchain"
    )
    
    foreach ($script in $scriptsToRemove) {
        if ($packageJson.scripts.PSObject.Properties.Name -contains $script) {
            $packageJson.scripts.PSObject.Properties.Remove($script)
            Write-Host "  ✅ Removed script: $script" -ForegroundColor Green
        }
    }
    
    # Update the package.json file
    $packageJson | ConvertTo-Json -Depth 10 | Set-Content $packageJsonPath
    Write-Host "✅ Updated package.json" -ForegroundColor Green
} else {
    Write-Host "⚠️  package.json not found" -ForegroundColor Yellow
}

# Update start scripts
Write-Host "📝 Updating start scripts..." -ForegroundColor Cyan

# Update start.ps1
$startScriptPath = "scripts\start.ps1"
if (Test-Path $startScriptPath) {
    $content = Get-Content $startScriptPath -Raw
    
    # Remove LangChain from concurrently command
    $content = $content -replace '--names "Frontend,Backend,LangChain,AI"', '--names "Frontend,Backend,AI"'
    $content = $content -replace '--prefix-colors "cyan,blue,magenta,yellow"', '--prefix-colors "cyan,blue,yellow"'
    $content = $content -replace '"npm run dev:frontend" "npm run dev:backend" "npm run dev:langchain" "npm run dev:ai"', '"npm run dev:frontend" "npm run dev:backend" "npm run dev:ai"'
    
    # Remove port check for LangChain
    $content = $content -replace 'Stop-ProcessOnPort -Port 3002 -ServiceName "LangChain"[^\r\n]*[\r\n]*', ''
    
    Set-Content $startScriptPath $content
    Write-Host "✅ Updated start.ps1" -ForegroundColor Green
} else {
    Write-Host "⚠️  start.ps1 not found" -ForegroundColor Yellow
}

# Remove TypeScript config for langchain
$tsConfigLangchain = "tsconfig.langchain.json"
if (Test-Path $tsConfigLangchain) {
    Remove-Item $tsConfigLangchain
    Write-Host "✅ Removed tsconfig.langchain.json" -ForegroundColor Green
}

$tsConfigBuildInfo = "tsconfig.langchain.tsbuildinfo"
if (Test-Path $tsConfigBuildInfo) {
    Remove-Item $tsConfigBuildInfo
    Write-Host "✅ Removed tsconfig.langchain.tsbuildinfo" -ForegroundColor Green
}

Write-Host ""
Write-Host "🎉 LangChain service cleanup completed!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Summary of changes:" -ForegroundColor Cyan
Write-Host "  • Removed langchain-service directory" -ForegroundColor White
Write-Host "  • Updated package.json scripts" -ForegroundColor White
Write-Host "  • Updated start.ps1 script" -ForegroundColor White
Write-Host "  • Removed TypeScript configuration files" -ForegroundColor White
Write-Host ""
Write-Host "✨ The architecture refactoring is now complete!" -ForegroundColor Green
Write-Host "   • Rust Backend: Handles all business logic and data connectors" -ForegroundColor White
Write-Host "   • Python AI Service: Handles all AI operations and vector search" -ForegroundColor White
Write-Host "   • Lexor Service: Specialized code search and indexing" -ForegroundColor White
Write-Host "   • Next.js Frontend: User interface" -ForegroundColor White