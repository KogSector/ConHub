# ConHub - Start All Services (Windows PowerShell)
# Run this script from the project root directory

Write-Host "🚀 Starting ConHub - All Services" -ForegroundColor Green
Write-Host "=================================="

# Check if we're in the right directory
if (-not (Test-Path "package.json")) {
    Write-Host "❌ Error: Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

# Activate virtual environment
Write-Host "📦 Activating Python virtual environment..." -ForegroundColor Yellow
if (Test-Path ".venv\Scripts\Activate.ps1") {
    & .\.venv\Scripts\Activate.ps1
} else {
    Write-Host "❌ Error: Virtual environment not found. Please create .venv first." -ForegroundColor Red
    exit 1
}

# Function to start service in new window
function Start-ServiceInNewWindow {
    param(
        [string]$Title,
        [string]$Command,
        [string]$WorkingDirectory = (Get-Location)
    )
    
    Write-Host "🔄 Starting $Title..." -ForegroundColor Cyan
    
    $encodedCommand = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($Command))
    
    Start-Process powershell -ArgumentList @(
        "-NoExit",
        "-WindowStyle", "Normal",
        "-EncodedCommand", $encodedCommand
    ) -WorkingDirectory $WorkingDirectory
    
    Start-Sleep -Seconds 2
}

# Start each service in a new PowerShell window
Write-Host "🎯 Starting services in separate windows..." -ForegroundColor Yellow

# Frontend (Next.js)
Start-ServiceInNewWindow -Title "Frontend" -Command @"
Write-Host 'ConHub Frontend - Port 3000' -ForegroundColor Green
cd frontend
npm run dev
"@

# Backend (Rust)
Start-ServiceInNewWindow -Title "Backend" -Command @"
Write-Host 'ConHub Backend - Port 3001' -ForegroundColor Blue
cd backend
cargo run
"@

# LangChain Service (TypeScript)
Start-ServiceInNewWindow -Title "LangChain" -Command @"
Write-Host 'ConHub LangChain Service - Port 3003' -ForegroundColor Magenta
cd langchain-service
set PORT=3003
nodemon --exec ts-node src/index.ts
"@

# Haystack Service (Python)
Start-ServiceInNewWindow -Title "Haystack" -Command @"
Write-Host 'ConHub Haystack Service - Port 8001' -ForegroundColor Yellow
& .\.venv\Scripts\Activate.ps1
cd haystack-service
python -m uvicorn app.main:app --host 0.0.0.0 --port 8001 --reload
"@

Write-Host ""
Write-Host "✅ All services are starting up!" -ForegroundColor Green
Write-Host "🌐 Frontend will be available at: http://localhost:3000" -ForegroundColor Cyan
Write-Host "🔧 Backend API available at: http://localhost:3001" -ForegroundColor Cyan
Write-Host "🤖 LangChain Service at: http://localhost:3003" -ForegroundColor Cyan
Write-Host "📚 Haystack Service at: http://localhost:8001" -ForegroundColor Cyan
Write-Host ""
Write-Host "⏳ Please wait for all services to fully start up..." -ForegroundColor Yellow
Write-Host "🔄 Check each window for startup completion status" -ForegroundColor Yellow