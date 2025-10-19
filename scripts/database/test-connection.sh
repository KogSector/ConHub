#!/bin/bash

# Default values
USERNAME="postgres"
PASSWORD=""
DB_HOST="localhost"
PORT="5432"
DATABASE="postgres"

# Parse command-line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        -Username) USERNAME="$2"; shift ;;
        -Password) PASSWORD="$2"; shift ;;
        -DbHost) DB_HOST="$2"; shift ;;
        -Port) PORT="$2"; shift ;;
        -Database) DATABASE="$2"; shift ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done

echo -e "\033[0;36m[TEST] Testing PostgreSQL Connection\033[0m"
echo "Host: $DB_HOST"
echo "Port: $PORT"
echo "Username: $USERNAME"
echo "Database: $DATABASE"
echo ""

export PGPASSWORD=$PASSWORD

echo -e "\033[1;33m[TESTING] Attempting connection...\033[0m"

RESULT=$(psql -h "$DB_HOST" -p "$PORT" -U "$USERNAME" -d "$DATABASE" -c "SELECT version();" 2>&1)

if [ $? -eq 0 ]; then
    echo -e "\033[0;32m[SUCCESS] Connection successful!\033[0m"
    echo "$RESULT"
    echo ""
    echo -e "\033[0;36mYou can now use these parameters:\033[0m"
    echo "./clear-database.sh -Username '$USERNAME' -Password '$PASSWORD'"
else
    echo -e "\033[0;31m[FAILED] Connection failed\033[0m"
    echo -e "\033[0;31m$RESULT\033[0m"
    echo ""
    echo -e "\033[1;33mTry these solutions:\033[0m"
    echo "1. Check if PostgreSQL service is running"
    echo "2. Try different username (postgres, admin, your_username)"
    echo "3. Try empty password: ./clear-database.sh -Password ''"
    echo "4. Use Docker: ./start-docker-postgres.sh"
fi

unset PGPASSWORD
