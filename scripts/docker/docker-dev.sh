#!/bin/bash

ACTION="up"
SERVICE=""
BUILD=false

# Parse command-line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        up|down|restart|logs|build|clean|help|h) ACTION="$1" ;;
        -Service) SERVICE="$2"; shift ;;
        -Build) BUILD=true ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done

# Color definitions
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function for colored output
write_color_output() {
    MESSAGE=$1
    COLOR=$2
    echo -e "${COLOR}${MESSAGE}${NC}"
}

show_header() {
    write_color_output "🐳 ConHub Docker Development Environment" $BLUE
    write_color_output "=======================================" $BLUE
    echo ""
}

check_prerequisites() {
    write_color_output "🔍 Checking prerequisites..." $YELLOW
    
    if ! command -v docker &> /dev/null; then
        write_color_output "❌ Docker is not installed or not running" $RED
        write_color_output "Please install Docker Desktop and ensure it's running" $RED
        exit 1
    fi
    DOCKER_VERSION=$(docker --version)
    write_color_output "✅ Docker found: $DOCKER_VERSION" $GREEN
    
    if ! command -v docker-compose &> /dev/null; then
        write_color_output "❌ Docker Compose is not available" $RED
        exit 1
    fi
    COMPOSE_VERSION=$(docker-compose --version)
    write_color_output "✅ Docker Compose found: $COMPOSE_VERSION" $GREEN
    
    if [ -f ".env" ]; then
        write_color_output "✅ Environment file found" $GREEN
    else
        write_color_output "⚠️  .env file not found, using .env.example" $YELLOW
        if [ -f ".env.example" ]; then
            cp ".env.example" ".env"
            write_color_output "✅ Created .env from .env.example" $GREEN
        else
            write_color_output "❌ .env.example not found" $RED
            exit 1
        fi
    fi
    
    echo ""
}

start_services() {
    write_color_output "🚀 Starting ConHub services..." $YELLOW
    
    COMPOSE_ARGS=("up" "-d")
    
    if [ "$BUILD" = true ]; then
        COMPOSE_ARGS+=("--build")
    fi
    
    if [ -n "$SERVICE" ]; then
        COMPOSE_ARGS+=("$SERVICE")
        write_color_output "Starting service: $SERVICE" $BLUE
    else
        write_color_output "Starting all services..." $BLUE
    fi
    
    docker-compose "${COMPOSE_ARGS[@]}"
    if [ $? -eq 0 ]; then
        write_color_output "✅ Services started successfully!" $GREEN
        show_service_status
    else
        write_color_output "❌ Failed to start services" $RED
        exit 1
    fi
}

stop_services() {
    write_color_output "🛑 Stopping ConHub services..." $YELLOW
    
    if [ -n "$SERVICE" ]; then
        docker-compose stop "$SERVICE"
        write_color_output "✅ Service $SERVICE stopped" $GREEN
    else
        docker-compose down
        write_color_output "✅ All services stopped" $GREEN
    fi
}

restart_services() {
    write_color_output "🔄 Restarting ConHub services..." $YELLOW
    stop_services
    sleep 2
    start_services
}

show_logs() {
    write_color_output "📋 Showing service logs..." $YELLOW
    
    if [ -n "$SERVICE" ]; then
        docker-compose logs -f "$SERVICE"
    else
        docker-compose logs -f
    fi
}

build_services() {
    write_color_output "🔨 Building ConHub services..." $YELLOW
    
    if [ -n "$SERVICE" ]; then
        docker-compose build "$SERVICE"
        write_color_output "✅ Service $SERVICE built successfully" $GREEN
    else
        docker-compose build
        write_color_output "✅ All services built successfully" $GREEN
    fi
}

clean_environment() {
    write_color_output "🧹 Cleaning Docker environment..." $YELLOW
    
    docker-compose down -v --remove-orphans
    docker image prune -f
    docker volume prune -f
    docker network prune -f
    
    write_color_output "✅ Environment cleaned successfully" $GREEN
}

show_service_status() {
    write_color_output "📊 Service Status:" $BLUE
    echo ""
    
    SERVICES=(
        "Frontend:3000:/"
        "Backend:3001:/health"
        "Lexor:3002:/health"
        "MCP Service:3004:/api/health"
        "AI Service:8001:/health"
        "LangChain:8003:/health"
    )
    
    for service in "${SERVICES[@]}"; do
        IFS=':' read -r -a arr <<< "$service"
        NAME=${arr[0]}
        PORT=${arr[1]}
        PATH=${arr[2]}
        
        STATUS_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:$PORT$PATH" -m 5)
        if [ "$STATUS_CODE" -eq 200 ]; then
            write_color_output "✅ $NAME (http://localhost:$PORT)" $GREEN
        else
            write_color_output "❌ $NAME (http://localhost:$PORT) - Not responding (Status: $STATUS_CODE)" $RED
        fi
    done
    
    echo ""
    write_color_output "🌐 Access ConHub at: http://localhost:3000" $BLUE
    echo ""
}

show_help() {
    write_color_output "ConHub Docker Development Commands:" $BLUE
    echo ""
    echo "  up       - Start all services (default)"
    echo "  down     - Stop all services"
    echo "  restart  - Restart all services"
    echo "  logs     - Show service logs"
    echo "  build    - Build services"
    echo "  clean    - Clean Docker environment"
    echo ""
    echo "Options:"
    echo "  -Service <name>  - Target specific service"
    echo "  -Build           - Build images before starting"
    echo ""
    echo "Examples:"
    echo "  ./docker-dev.sh up -Build"
    echo "  ./docker-dev.sh logs -Service backend"
    echo "  ./docker-dev.sh restart -Service frontend"
    echo ""
}

show_header

if [ "$ACTION" = "help" ] || [ "$ACTION" = "h" ]; then
    show_help
    exit 0
fi

check_prerequisites

case $ACTION in
    up) start_services ;;
    down) stop_services ;;
    restart) restart_services ;;
    logs) show_logs ;;
    build) build_services ;;
    clean) clean_environment ;;
    *)
        write_color_output "❌ Unknown action: $ACTION" $RED
        show_help
        exit 1
        ;;
esac

write_color_output "🎉 Operation completed!" $GREEN
