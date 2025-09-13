# Test script for ConHub services

Write-Host "=== ConHub Services Health Check ===" -ForegroundColor Green

# Test LangChain Service
try {
    $langchainHealth = Invoke-RestMethod -Uri "http://localhost:3001/health" -Method GET -ErrorAction Stop
    Write-Host "✅ LangChain Service: " -NoNewline -ForegroundColor Green
    Write-Host "$($langchainHealth.status) ($($langchainHealth.service))" -ForegroundColor Cyan
} catch {
    Write-Host "❌ LangChain Service: Not responding" -ForegroundColor Red
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Test Haystack Service  
try {
    $haystackHealth = Invoke-RestMethod -Uri "http://localhost:8001/health" -Method GET -ErrorAction Stop
    Write-Host "✅ Haystack Service: " -NoNewline -ForegroundColor Green
    Write-Host "$($haystackHealth.status) ($($haystackHealth.service))" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Haystack Service: Not responding" -ForegroundColor Red
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host "`n=== Service Endpoints ===" -ForegroundColor Green
Write-Host "🔗 LangChain Service: http://localhost:3001" -ForegroundColor Cyan
Write-Host "   - Health: GET /health"
Write-Host "   - Index Repository: POST /index/repository"
Write-Host "   - Search: POST /search"
Write-Host "   - Data Sources: GET /data-sources"

Write-Host "`n🔗 Haystack Service: http://localhost:8001" -ForegroundColor Cyan
Write-Host "   - Health: GET /health"
Write-Host "   - Upload Document: POST /upload"
Write-Host "   - Search Documents: POST /search"
Write-Host "   - Ask Question: POST /ask"
Write-Host "   - List Documents: GET /documents"

Write-Host "`n=== Usage Examples ===" -ForegroundColor Green
Write-Host "📝 Upload a document to Haystack:"
Write-Host '   curl -X POST "http://localhost:8001/upload" -F "file=@document.pdf"' -ForegroundColor Gray

Write-Host "`n🔍 Search documents:"
Write-Host '   curl -X POST "http://localhost:8001/search" -H "Content-Type: application/json" -d ''{"query": "your search query"}''' -ForegroundColor Gray

Write-Host "`n❓ Ask a question:"
Write-Host '   curl -X POST "http://localhost:8001/ask" -H "Content-Type: application/json" -d ''{"query": "What is this about?"}''' -ForegroundColor Gray

Write-Host "`n🏗️ Index a repository (LangChain):"
Write-Host '   curl -X POST "http://localhost:3001/index/repository" -H "Content-Type: application/json" -d ''{"url": "https://github.com/user/repo", "type": "github"}''' -ForegroundColor Gray
