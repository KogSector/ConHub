@echo off
REM ConHub - Stop All Services (Windows Batch)

echo 🛑 Stopping ConHub Services
echo ============================

echo 🔄 Stopping Node.js processes...
taskkill /F /IM node.exe >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Node.js processes stopped
) else (
    echo ⚠️  No Node.js processes were running
)

echo 🔄 Stopping Rust backend...
taskkill /F /IM conhub-backend.exe >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Rust backend stopped
) else (
    echo ⚠️  Rust backend was not running
)

echo 🔄 Stopping Python processes...
taskkill /F /IM python.exe >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Python processes stopped
) else (
    echo ⚠️  No Python processes were running
)

echo 🧹 Cleaning up any remaining processes on ConHub ports...
for %%p in (3000 3001 3003 8001) do (
    for /f "tokens=5" %%a in ('netstat -ano ^| findstr ":%%p"') do (
        taskkill /F /PID %%a >nul 2>&1
    )
)

echo.
echo ✅ All ConHub services have been stopped
pause