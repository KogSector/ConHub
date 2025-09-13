@echo off
REM ConHub Services Health Check (Windows Batch)

echo === ConHub Services Health Check ===

REM Test Frontend
curl -s http://localhost:3000 >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Frontend: Running
) else (
    echo ❌ Frontend: Not responding
)

REM Test Backend
curl -s http://localhost:3001/health >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Backend: Running
) else (
    echo ❌ Backend: Not responding
)

REM Test LangChain Service
curl -s http://localhost:3003/health >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ LangChain Service: Running
) else (
    echo ❌ LangChain Service: Not responding
)

REM Test Haystack Service
curl -s http://localhost:8001/health >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Haystack Service: Running
) else (
    echo ❌ Haystack Service: Not responding
)

echo.
echo 🔗 Services are running on:
echo    Frontend:  http://localhost:3000
echo    Backend:   http://localhost:3001
echo    LangChain: http://localhost:3003
echo    Haystack:  http://localhost:8001
pause