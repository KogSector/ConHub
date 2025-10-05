#!/bin/bash
# ConHub - Stop All Services (Bash)

echo "🛑 Stopping ConHub Services..."

# Function to stop process on port
stop_process_on_port() {
    local port=$1
    local pid=$(lsof -ti:$port 2>/dev/null)
    if [ -n "$pid" ]; then
        local process_name=$(ps -p $pid -o comm= 2>/dev/null)
        echo "🔄 Stopping process on port $port ($process_name)..."
        kill -9 $pid 2>/dev/null
        echo "✅ Port $port freed"
    fi
}

echo "🛑 Stopping ConHub processes..."

# Stop Rust services
if pgrep -f "conhub-backend" > /dev/null; then
    echo "Stopping conhub-backend processes..."
    pkill -9 -f "conhub-backend"
fi

if pgrep -f "lexor" > /dev/null; then
    echo "Stopping lexor processes..."
    pkill -9 -f "lexor"
fi

# Stop Python/AI service processes
if pgrep -f "uvicorn.*ai-service" > /dev/null; then
    echo "Stopping AI service processes..."
    pkill -9 -f "uvicorn.*ai-service"
fi

# Stop Node.js processes (Next.js, concurrently, monitors)
if pgrep -f "next" > /dev/null; then
    echo "Stopping Next.js processes..."
    pkill -9 -f "next"
fi

if pgrep -f "concurrently" > /dev/null; then
    echo "Stopping concurrently processes..."
    pkill -9 -f "concurrently"
fi

if pgrep -f "monitor-services" > /dev/null; then
    echo "Stopping monitor processes..."
    pkill -9 -f "monitor-services"
fi

# Force stop by ports (catch any remaining processes)
echo "🔍 Checking ports..."
stop_process_on_port 3000  # Frontend
stop_process_on_port 3001  # Backend
stop_process_on_port 3002  # Lexor
stop_process_on_port 8001  # AI Service

# Wait a moment for processes to fully terminate
sleep 2

echo "🏁 All ConHub services stopped."