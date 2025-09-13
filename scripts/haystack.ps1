# ConHub - Start Haystack Service Only (PowerShell)

Write-Host "📚 Starting ConHub Haystack Service..." -ForegroundColor Yellow

if (-not (Test-Path "package.json")) {
    Write-Host "❌ Error: Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

# Activate virtual environment
if (Test-Path ".venv\Scripts\Activate.ps1") {
    & .\.venv\Scripts\Activate.ps1
    Write-Host "📦 Virtual environment activated" -ForegroundColor Green
} else {
    Write-Host "❌ Error: Virtual environment not found" -ForegroundColor Red
    exit 1
}

Set-Location haystack-service
Write-Host "🔄 Starting Haystack service on port 8001..." -ForegroundColor Cyan
python -m uvicorn app.main:app --host 0.0.0.0 --port 8001 --reload