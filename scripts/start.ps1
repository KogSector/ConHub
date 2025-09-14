# ConHub - Start All Services (Windows PowerShell)
Write-Host "Starting ConHub Services..." -ForegroundColor Green

# Check if we're in the right directory
if (-not (Test-Path "package.json")) {
    Write-Host "Error: Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

# Use concurrently to start all services
Write-Host "Starting all services with concurrently..." -ForegroundColor Yellow

npx concurrently --kill-others --names "Frontend,Backend,LangChain,Haystack" --prefix-colors "cyan,blue,magenta,yellow" "npm run dev:frontend" "npm run dev:backend" "npm run dev:langchain" "npm run dev:haystack"