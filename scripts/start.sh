#!/bin/bash
# ConHub - Start All Services (Linux/macOS)
# Run this script from the project root directory

echo "🚀 Starting ConHub - All Services"
echo "=================================="

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Check if virtual environment exists
if [ ! -d ".venv" ]; then
    echo "❌ Error: Virtual environment not found. Please create .venv first."
    exit 1
fi

# Activate virtual environment
echo "📦 Activating Python virtual environment..."
source .venv/bin/activate

# Function to start service in background
start_service() {
    local service_name="$1"
    local command="$2"
    local log_file="logs/${service_name}.log"
    
    echo "🔄 Starting $service_name..."
    mkdir -p logs
    
    # Start service in background and redirect output to log file
    eval "$command" > "$log_file" 2>&1 &
    local pid=$!
    
    # Store PID for later cleanup
    echo $pid > "logs/${service_name}.pid"
    
    echo "✅ $service_name started (PID: $pid, Log: $log_file)"
    sleep 2
}

echo "🎯 Starting services..."

# Start Frontend (Next.js)
start_service "frontend" "cd frontend && npm run dev"

# Start Backend (Rust)
start_service "backend" "cd backend && cargo run"

# Start LangChain Service (TypeScript)
start_service "langchain" "cd langchain-service && PORT=3003 nodemon --exec ts-node src/index.ts"

# Start Haystack Service (Python)
start_service "haystack" "cd haystack-service && python -m uvicorn app.main:app --host 0.0.0.0 --port 8001 --reload"

echo ""
echo "✅ All services are starting up!"
echo "🌐 Frontend will be available at: http://localhost:3000"
echo "🔧 Backend API available at: http://localhost:3001"
echo "🤖 LangChain Service at: http://localhost:3003"
echo "📚 Haystack Service at: http://localhost:8001"
echo ""
echo "📊 Service logs are in the 'logs/' directory"
echo "⏳ Please wait for all services to fully start up..."
echo ""
echo "🛑 To stop all services, run: ./scripts/stop-all.sh"