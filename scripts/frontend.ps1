# ConHub - Start Frontend Only (PowerShell)

Write-Host "🌐 Starting ConHub Frontend..." -ForegroundColor Green

if (-not (Test-Path "package.json")) {
    Write-Host "❌ Error: Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

cd frontend
Write-Host "🔄 Starting Next.js development server on port 3000..." -ForegroundColor Cyan
npm run dev