#!/bin/bash
# ConHub - Start Haystack Service Only (Linux/macOS)

echo "📚 Starting ConHub Haystack Service..."

if [ ! -f "package.json" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Activate virtual environment
if [ -d ".venv" ]; then
    source .venv/bin/activate
    echo "📦 Virtual environment activated"
else
    echo "❌ Error: Virtual environment not found"
    exit 1
fi

cd haystack-service
echo "🔄 Starting Haystack service on port 8001..."
python -m uvicorn app.main:app --host 0.0.0.0 --port 8001 --reload