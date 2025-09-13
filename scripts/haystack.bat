@echo off
REM ConHub - Start Haystack Service Only (Windows Batch)

echo 📚 Starting ConHub Haystack Service...

if not exist "package.json" (
    echo ❌ Error: Please run this script from the project root directory
    pause
    exit /b 1
)

REM Activate virtual environment
if exist ".venv\Scripts\activate.bat" (
    call .venv\Scripts\activate.bat
    echo 📦 Virtual environment activated
) else (
    echo ❌ Error: Virtual environment not found
    pause
    exit /b 1
)

cd haystack-service
echo 🔄 Starting Haystack service on port 8001...
python -m uvicorn app.main:app --host 0.0.0.0 --port 8001 --reload