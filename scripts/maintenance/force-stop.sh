#!/bin/bash

PROCESSES=("node" "next-server" "conhub-backend" "lexor" "python" "uvicorn")
PORTS=(3000 3001 3002 8001 8003)

echo "Force stopping ConHub related processes..."

# Kill processes by name and command line arguments
for proc in "${PROCESSES[@]}"; do
    pids=$(pgrep -f "$proc")
    if [ -n "$pids" ]; then
        for pid in $pids; do
            cmdline=$(ps -p $pid -o command=)
            if [[ $cmdline == *3000* || $cmdline == *3001* || $cmdline == *3002* || $cmdline == *8001* || $cmdline == *8003* || $cmdline == *conhub* || $cmdline == *lexor* || $cmdline == *uvicorn* ]]; then
                echo "Killing process $proc with PID $pid"
                kill -9 $pid
            fi
        done
    fi
done

# Kill processes by port
for port in "${PORTS[@]}"; do
    if command -v lsof &> /dev/null; then
        pids=$(lsof -t -i:$port)
        if [ -n "$pids" ]; then
            for pid in $pids; do
                echo "Killing process on port $port with PID $pid"
                kill -9 $pid
            done
        fi
    elif command -v netstat &> /dev/null; then
        pid=$(netstat -aon | grep ":$port" | awk '{print $5}' | head -n 1)
        if [ -n "$pid" ] && [ "$pid" -ne 0 ]; then
            echo "Killing process on port $port with PID $pid"
            taskkill //F //PID $pid
        fi
    fi
done

echo "Force stop complete."
