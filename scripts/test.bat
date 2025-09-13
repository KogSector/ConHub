@echo off
REM Test script for ConHub services (Windows Batch)

echo === ConHub Services Health Check ===

echo 🔄 Testing Frontend...
curl -s http://localhost:3000 >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Frontend: Running
) else (
    echo ❌ Frontend: Not responding
)

echo 🔄 Testing Backend...
curl -s http://localhost:3001/health >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Backend: Running
) else (
    echo ❌ Backend: Not responding
)

echo 🔄 Testing LangChain Service...
curl -s http://localhost:3003/health >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ LangChain Service: Running
) else (
    echo ❌ LangChain Service: Not responding
)

echo 🔄 Testing Haystack Service...
curl -s http://localhost:8001/health >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Haystack Service: Running
) else (
    echo ❌ Haystack Service: Not responding
)

echo.
echo === Service Endpoints ===
echo 🔗 Frontend: http://localhost:3000
echo 🔗 Backend: http://localhost:3001
echo    - Health: GET /health
echo 🔗 LangChain Service: http://localhost:3003
echo    - Health: GET /health
echo    - Index Repository: POST /index/repository
echo    - Search: POST /search
echo 🔗 Haystack Service: http://localhost:8001
echo    - Health: GET /health
echo    - Upload Document: POST /documents/upload
echo    - Search: POST /search
pause