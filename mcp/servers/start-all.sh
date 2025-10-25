#!/bin/bash

# ConHub MCP Servers Startup Script
# Starts all external MCP servers for Google Drive, Dropbox, and Filesystem

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="${SCRIPT_DIR}/logs"
PID_DIR="${SCRIPT_DIR}/pids"

# Create directories for logs and PIDs
mkdir -p "$LOG_DIR" "$PID_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================="
echo "ConHub MCP Servers Startup"
echo "========================================="
echo ""

# Function to start a single MCP server
start_server() {
  local name=$1
  local dir=$2
  local port=$3
  local pid_file="${PID_DIR}/${name}.pid"
  local log_file="${LOG_DIR}/${name}.log"

  echo -n "Starting ${name} MCP server on port ${port}... "

  # Check if server is already running
  if [ -f "$pid_file" ]; then
    local old_pid=$(cat "$pid_file")
    if ps -p "$old_pid" > /dev/null 2>&1; then
      echo -e "${YELLOW}Already running (PID: ${old_pid})${NC}"
      return 0
    else
      # Stale PID file, remove it
      rm -f "$pid_file"
    fi
  fi

  # Install dependencies if node_modules doesn't exist
  if [ ! -d "${dir}/node_modules" ]; then
    echo ""
    echo "  Installing dependencies for ${name}..."
    cd "$dir"
    npm install --silent > /dev/null 2>&1
    cd "$SCRIPT_DIR"
  fi

  # Start the server in background
  cd "$dir"
  nohup npm start > "$log_file" 2>&1 &
  local pid=$!
  echo $pid > "$pid_file"
  cd "$SCRIPT_DIR"

  # Wait a moment and check if process is still running
  sleep 2
  if ps -p "$pid" > /dev/null 2>&1; then
    echo -e "${GREEN}Started (PID: ${pid})${NC}"
    echo "  Log: ${log_file}"
  else
    echo -e "${RED}Failed to start${NC}"
    echo "  Check log: ${log_file}"
    return 1
  fi
}

# Function to stop all MCP servers
stop_all() {
  echo "Stopping all MCP servers..."
  for pid_file in "${PID_DIR}"/*.pid; do
    if [ -f "$pid_file" ]; then
      local pid=$(cat "$pid_file")
      local name=$(basename "$pid_file" .pid)
      if ps -p "$pid" > /dev/null 2>&1; then
        echo "  Stopping ${name} (PID: ${pid})..."
        kill "$pid"
        rm -f "$pid_file"
      else
        rm -f "$pid_file"
      fi
    fi
  done
  echo "All servers stopped."
}

# Handle script arguments
if [ "$1" == "stop" ]; then
  stop_all
  exit 0
elif [ "$1" == "restart" ]; then
  stop_all
  sleep 2
  # Fall through to start servers
elif [ "$1" == "status" ]; then
  echo "MCP Servers Status:"
  for pid_file in "${PID_DIR}"/*.pid; do
    if [ -f "$pid_file" ]; then
      local pid=$(cat "$pid_file")
      local name=$(basename "$pid_file" .pid)
      if ps -p "$pid" > /dev/null 2>&1; then
        echo -e "  ${name}: ${GREEN}Running${NC} (PID: ${pid})"
      else
        echo -e "  ${name}: ${RED}Not running${NC} (stale PID file)"
      fi
    fi
  done
  exit 0
fi

# Start all MCP servers
start_server "google-drive" "${SCRIPT_DIR}/google-drive" 3005
start_server "dropbox" "${SCRIPT_DIR}/dropbox" 3006
start_server "filesystem" "${SCRIPT_DIR}/filesystem" 3007

echo ""
echo "========================================="
echo "All MCP servers started successfully!"
echo "========================================="
echo ""
echo "Health checks:"
echo "  Google Drive: http://localhost:3005/health"
echo "  Dropbox:      http://localhost:3006/health"
echo "  Filesystem:   http://localhost:3007/health"
echo ""
echo "Logs directory: ${LOG_DIR}"
echo "PIDs directory: ${PID_DIR}"
echo ""
echo "Commands:"
echo "  ./start-all.sh stop    - Stop all servers"
echo "  ./start-all.sh restart - Restart all servers"
echo "  ./start-all.sh status  - Check server status"
echo ""

# Trap SIGTERM and SIGINT to gracefully stop servers
trap stop_all SIGTERM SIGINT
