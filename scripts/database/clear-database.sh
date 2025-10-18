#!/bin/bash

# Default values
DB_URL="postgresql://localhost:5432/conhub"
DB_NAME="conhub"
DB_USER="postgres"
DB_PASS=""
DB_HOST="localhost"
DB_PORT="5432"
CONFIRM=false

# Parse command-line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        -DatabaseUrl) DB_URL="$2"; shift ;;
        -DatabaseName) DB_NAME="$2"; shift ;;
        -Username) DB_USER="$2"; shift ;;
        -Password) DB_PASS="$2"; shift ;;
        -DatabaseHost) DB_HOST="$2"; shift ;;
        -Port) DB_PORT="$2"; shift ;;
        -Confirm) CONFIRM=true ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done

echo -e "\033[1;33m[DATABASE] Clearing ConHub database...\033[0m"

if [ "$CONFIRM" = false ]; then
    read -p "Are you sure you want to delete ALL data? This cannot be undone! (type 'yes' to confirm) " response
    if [ "$response" != "yes" ]; then
        echo -e "\033[0;31m[CANCELLED] Database clearing cancelled by user\033[0m"
        exit 0
    fi
fi

if ! command -v psql &> /dev/null; then
    echo -e "\033[0;31m[ERROR] PostgreSQL client (psql) not found. Please install PostgreSQL or add it to PATH.\033[0m"
    exit 1
fi

export PGPASSWORD=$DB_PASS

echo -e "\033[0;36m[EXECUTING] Running database cleanup script...\033[0m"
echo -e "\033[1;33m[CONNECTION] Connecting to: ${DB_HOST}:${DB_PORT} as user: $DB_USER\033[0m"

SCRIPT_DIR=$(dirname "$0")
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$SCRIPT_DIR/clear-database.sql"

if [ $? -eq 0 ]; then
    echo -e "\033[0;32m[SUCCESS] Database cleared successfully!\033[0m"
else
    echo -e "\033[0;31m[ERROR] Failed to clear database\033[0m"
    unset PGPASSWORD
    exit 1
fi

unset PGPASSWORD

echo -e "\033[0;32m[COMPLETE] Database cleanup completed\033[0m"
