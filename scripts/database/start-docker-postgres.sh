#!/bin/bash

# Default values
PASSWORD="postgres"
DATABASE="conhub"
PORT=5432

# Parse command-line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        -Password) PASSWORD="$2"; shift ;;
        -Database) DATABASE="$2"; shift ;;
        -Port) PORT="$2"; shift ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done

echo -e "\033[0;36m[DOCKER] Starting PostgreSQL with Docker...\033[0m"

if ! command -v docker &> /dev/null; then
    echo -e "\033[0;31m[ERROR] Docker not found. Please install Docker Desktop.\033[0m"
    exit 1
fi

echo -e "\033[1;33m[CLEANUP] Stopping existing PostgreSQL container...\033[0m"
docker stop conhub-postgres &> /dev/null
docker rm conhub-postgres &> /dev/null

echo -e "\033[0;32m[STARTING] Starting PostgreSQL container...\033[0m"
docker run -d \
    --name conhub-postgres \
    -e POSTGRES_PASSWORD="$PASSWORD" \
    -e POSTGRES_DB="$DATABASE" \
    -p "$PORT":5432 \
    postgres:15

if [ $? -eq 0 ]; then
    echo -e "\033[0;32m[SUCCESS] PostgreSQL started successfully!\033[0m"
    echo -e "\033[0;36mConnection details:\033[0m"
    echo "  Host: localhost"
    echo "  Port: $PORT"
    echo "  Database: $DATABASE"
    echo "  Username: postgres"
    echo "  Password: $PASSWORD"
    echo ""
    echo -e "\033[0;32mNow you can run:\033[0m"
    echo "./clear-database.sh -Password '$PASSWORD'"
    echo ""
    echo -e "\033[1;33mTo stop the container later:\033[0m"
    echo "docker stop conhub-postgres"
else
    echo -e "\033[0;31m[ERROR] Failed to start PostgreSQL container\033[0m"
fi
