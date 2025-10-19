#!/bin/bash

echo -e "\033[0;32m[START] Starting ConHub...\033[0m"

if [ ! -f "package.json" ]; then
    echo -e "\033[0;31m[ERROR] Run from project root\033[0m"
    exit 1
fi

echo -e "\033[1;33m[CLEANUP] Cleaning up ports and locks...\033[0m"
SCRIPT_DIR=$(dirname "$0")
"$SCRIPT_DIR/../maintenance/cleanup-ports.sh"

BACKEND_BINARY="target/debug/conhub-backend"
LEXOR_BINARY="target/debug/lexor"

if [ ! -f "$BACKEND_BINARY" ] || [ ! -f "$LEXOR_BINARY" ]; then
    echo -e "\033[0;36m[BUILD] Building binaries...\033[0m"
    cargo build --bin conhub-backend --bin lexor --quiet
    if [ $? -ne 0 ]; then
        echo -e "\033[0;31m[ERROR] Build failed\033[0m"
        exit 1
    fi
    echo -e "\033[0;32m[OK] Build completed\033[0m"
fi

echo -e "\033[0;36m[SERVICES] Starting all services...\033[0m"
echo "   Frontend: http://localhost:3000"
echo "   Backend:  http://localhost:3001"
echo "   Lexor:    http://localhost:3002"
echo ""

./node_modules/.bin/concurrently --names "Frontend,Backend,Lexor" --prefix-colors "cyan,blue,magenta" --restart-tries 2 --kill-others-on-fail "npm run dev:frontend" "npm run dev:backend" "npm run dev:lexor"
