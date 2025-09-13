#!/bin/bash
# ConHub - Stop All Services (Linux/macOS)

echo "🛑 Stopping ConHub Services"
echo "============================"

# Function to stop service by PID file
stop_service() {
    local service_name="$1"
    local pid_file="logs/${service_name}.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            echo "🔄 Stopping $service_name (PID: $pid)..."
            kill "$pid"
            rm "$pid_file"
            echo "✅ $service_name stopped"
        else
            echo "⚠️  $service_name was not running"
            rm "$pid_file"
        fi
    else
        echo "⚠️  No PID file found for $service_name"
    fi
}

# Stop all services
stop_service "frontend"
stop_service "backend"
stop_service "langchain"
stop_service "haystack"

# Clean up any remaining Node.js processes
echo "🧹 Cleaning up any remaining processes..."
pkill -f "next dev" 2>/dev/null || true
pkill -f "nodemon" 2>/dev/null || true
pkill -f "uvicorn" 2>/dev/null || true

echo ""
echo "✅ All ConHub services have been stopped"