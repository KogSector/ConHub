#!/bin/bash

export RUST_LOG="info"

if [ ! -f "target/debug/lexor" ]; then
    echo -e "\033[0;36m[BUILD] Building lexor...\033[0m"
    cargo build --bin lexor --quiet
fi

echo -e "\033[0;35m[LEXOR] Starting on port 3002...\033[0m"
./target/debug/lexor
