@echo off
REM ConHub - Start All Services (Windows Batch)
REM Run this script from the project root directory

echo 🚀 Starting ConHub - All Services
echo ==================================

REM Check if we're in the right directory
if not exist "package.json" (
    echo ❌ Error: Please run this script from the project root directory
    pause
    exit /b 1
)

REM Check if virtual environment exists
if not exist ".venv\Scripts\activate.bat" (
    echo ❌ Error: Virtual environment not found. Please create .venv first.
    pause
    exit /b 1
)

echo 📦 Activating Python virtual environment...
call .venv\Scripts\activate.bat

echo 🎯 Starting services in separate windows...

REM Start Frontend (Next.js)
echo 🔄 Starting Frontend...
start "ConHub Frontend" cmd /k "cd frontend && echo ConHub Frontend - Port 3000 && npm run dev"

REM Start Backend (Rust)
echo 🔄 Starting Backend...
start "ConHub Backend" cmd /k "cd backend && echo ConHub Backend - Port 3001 && cargo run"

REM Start LangChain Service (TypeScript)
echo 🔄 Starting LangChain Service...
start "ConHub LangChain" cmd /k "cd langchain-service && echo ConHub LangChain Service - Port 3003 && set PORT=3003 && nodemon --exec ts-node src/index.ts"

REM Start Haystack Service (Python)
echo 🔄 Starting Haystack Service...
start "ConHub Haystack" cmd /k "call .venv\Scripts\activate.bat && cd haystack-service && echo ConHub Haystack Service - Port 8001 && python -m uvicorn app.main:app --host 0.0.0.0 --port 8001 --reload"

echo.
echo ✅ All services are starting up!
echo 🌐 Frontend will be available at: http://localhost:3000
echo 🔧 Backend API available at: http://localhost:3001
echo 🤖 LangChain Service at: http://localhost:3003
echo 📚 Haystack Service at: http://localhost:8001
echo.
echo ⏳ Please wait for all services to fully start up...
echo 🔄 Check each window for startup completion status
echo.
pause