@echo off
REM ConHub - Start Frontend Only (Windows Batch)

echo 🌐 Starting ConHub Frontend...

if not exist "package.json" (
    echo ❌ Error: Please run this script from the project root directory
    pause
    exit /b 1
)

cd frontend
echo 🔄 Starting Next.js development server on port 3000...
npm run dev