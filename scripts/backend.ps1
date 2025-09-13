#!/usr/bin/env pwsh

Write-Host "🚀 Starting ConHub Backend Services..." -ForegroundColor Green

# Check if Rust is installed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust/Cargo not found. Please install Rust first." -ForegroundColor Red
    exit 1
}

# Check if Node.js is installed
if (!(Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Node.js not found. Please install Node.js first." -ForegroundColor Red
    exit 1
}

# Check if Python is installed
if (!(Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Python not found. Please install Python first." -ForegroundColor Red
    exit 1
}

Write-Host "✅ All prerequisites found" -ForegroundColor Green

# Start Rust Backend (Main orchestrator)
Write-Host "🦀 Starting Rust Backend on port 3001..." -ForegroundColor Yellow
Start-Process -FilePath "cargo" -ArgumentList "run" -WorkingDirectory "backend" -WindowStyle Minimized

# Start LangChain Service
Write-Host "🔗 Starting LangChain Service on port 3002..." -ForegroundColor Yellow
Start-Process -FilePath "npm" -ArgumentList "run", "dev" -WorkingDirectory "langchain-service" -WindowStyle Minimized

# Start Haystack Service
Write-Host "🌾 Starting Haystack Service on port 8001..." -ForegroundColor Yellow
Start-Process -FilePath "python" -ArgumentList "main.py" -WorkingDirectory "haystack-service" -WindowStyle Minimized

# Start Lexor Service
Write-Host "🔍 Starting Lexor Service on port 3002..." -ForegroundColor Yellow
Start-Process -FilePath "cargo" -ArgumentList "run" -WorkingDirectory "lexor" -WindowStyle Minimized

Write-Host ""
Write-Host "🎉 All ConHub Backend Services Started!" -ForegroundColor Green
Write-Host ""
Write-Host "📊 Service URLs:" -ForegroundColor Cyan
Write-Host "  • Main Backend:    http://localhost:3001" -ForegroundColor White
Write-Host "  • LangChain:       http://localhost:3002" -ForegroundColor White
Write-Host "  • Haystack:        http://localhost:8001" -ForegroundColor White
Write-Host "  • Lexor:           http://localhost:3002" -ForegroundColor White
Write-Host ""
Write-Host "🔍 Health Checks:" -ForegroundColor Cyan
Write-Host "  • curl http://localhost:3001/health" -ForegroundColor White
Write-Host ""
Write-Host "Press Ctrl+C to stop all services" -ForegroundColor Yellow

# Keep script running
try {
    while ($true) {
        Start-Sleep -Seconds 1
    }
} catch {
    Write-Host "🛑 Stopping all services..." -ForegroundColor Red
}