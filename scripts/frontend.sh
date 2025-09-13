#!/bin/bash
# ConHub - Start Frontend Only (Linux/macOS)

echo "🌐 Starting ConHub Frontend..."

if [ ! -f "package.json" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

cd frontend
echo "🔄 Starting Next.js development server on port 3000..."
npm run dev