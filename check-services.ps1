Write-Host "=== ConHub Services Health Check ===" -ForegroundColor Green

# Test LangChain Service
try {
    $langchainHealth = Invoke-RestMethod -Uri "http://localhost:3001/health" -Method GET -ErrorAction Stop
    Write-Host "✅ LangChain Service: $($langchainHealth.status)" -ForegroundColor Green
} catch {
    Write-Host "❌ LangChain Service: Not responding" -ForegroundColor Red
}

# Test Haystack Service  
try {
    $haystackHealth = Invoke-RestMethod -Uri "http://localhost:8001/health" -Method GET -ErrorAction Stop
    Write-Host "✅ Haystack Service: $($haystackHealth.status)" -ForegroundColor Green
} catch {
    Write-Host "❌ Haystack Service: Not responding" -ForegroundColor Red
}

Write-Host "`n🔗 Services are running on:" -ForegroundColor Cyan
Write-Host "   LangChain: http://localhost:3001"
Write-Host "   Haystack:  http://localhost:8001"
