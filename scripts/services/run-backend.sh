#!/bin/bash

export RUST_LOG="info"
export RUST_BACKTRACE="1"

if [ ! -f "target/debug/conhub-backend" ]; then
    echo -e "\033[0;36m[BUILD] Building backend...\033[0m"
    cargo build --bin conhub-backend --quiet
fi

echo -e "\033[0;34m[BACKEND] Starting on port 3001...\033[0m"
./target/debug/conhub-backend
