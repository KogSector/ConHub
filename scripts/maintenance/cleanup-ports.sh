#!/bin/bash

PORTS=(3000 3001 3002 8001 8003)

for port in "${PORTS[@]}"; do
    echo -e "\033[1;33mChecking port $port...\033[0m"
    
    # For Linux and macOS
    if command -v lsof &> /dev/null; then
        PID=$(lsof -t -i:$port)
        if [ -n "$PID" ]; then
            echo -e "\033[1;33mPort $port is in use by PID $PID. Terminating process...\033[0m"
            kill -9 $PID
        fi
    # For Windows (Git Bash, etc.)
    elif command -v netstat &> /dev/null; then
        PID=$(netstat -aon | grep ":$port" | awk '{print $5}' | head -n 1)
        if [ -n "$PID" ] && [ "$PID" -ne 0 ]; then
            echo -e "\033[1;33mPort $port is in use by PID $PID. Terminating process...\033[0m"
            taskkill //F //PID $PID
        fi
    fi
done

LOCK_FILE="lexor_data/index/.tantivy-writer.lock"
if [ -f "$LOCK_FILE" ]; then
    echo -e "\033[1;33mFound lock file at $LOCK_FILE. Removing...\033[0m"
    rm -f "$LOCK_FILE"
fi

# Give the OS a moment to release resources
sleep 3
