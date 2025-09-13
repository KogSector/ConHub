@echo off
REM ConHub - Start Backend Only (Windows Batch)

echo 🔧 Starting ConHub Backend...

if not exist "package.json" (
    echo ❌ Error: Please run this script from the project root directory
    pause
    exit /b 1
)

cd backend
echo 🔄 Starting Rust backend server on port 3001...
cargo run