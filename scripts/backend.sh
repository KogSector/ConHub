#!/bin/bash
# ConHub - Start Backend Only (Linux/macOS)

echo "🔧 Starting ConHub Backend..."

if [ ! -f "package.json" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

cd backend
echo "🔄 Starting Rust backend server on port 3001..."
cargo run