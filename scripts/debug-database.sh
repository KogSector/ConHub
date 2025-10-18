#!/bin/bash

# Default backend URL
BACKEND_URL=${1:-"http://localhost:3001"}

# Exit on error
set -e

# Color definitions
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function for colored output
write_color_output() {
    MESSAGE=$1
    COLOR=$2
    echo -e "${COLOR}${MESSAGE}${NC}"
}

write_color_output "🔍 ConHub Database Debug Script" $BLUE
write_color_output "==============================" $BLUE
echo ""

# Step 1: Checking backend health
write_color_output "Step 1: Checking backend health..." $YELLOW
HEALTH_RESPONSE=$(curl -s -m 10 "$BACKEND_URL/health")
if [ $? -ne 0 ]; then
    write_color_output "❌ Backend health check failed: Could not connect to backend." $RED
    exit 1
fi

BACKEND_STATUS=$(echo $HEALTH_RESPONSE | jq -r .status)
DATABASE_STATUS=$(echo $HEALTH_RESPONSE | jq -r .database)
TIMESTAMP=$(echo $HEALTH_RESPONSE | jq -r .timestamp)

write_color_output "✅ Backend Status: $BACKEND_STATUS" $GREEN
write_color_output "✅ Database Status: $DATABASE_STATUS" $GREEN
echo "Timestamp: $TIMESTAMP"

if [ "$DATABASE_STATUS" != "connected" ]; then
    write_color_output "❌ Database is not connected!" $RED
    exit 1
fi
echo ""

# Step 2: Checking database structure
write_color_output "Step 2: Checking database structure..." $YELLOW
DB_TEST_RESPONSE=$(curl -s -m 10 "$BACKEND_URL/api/auth/test-db")
if [ $? -ne 0 ]; then
    write_color_output "❌ Database test failed: Could not connect to backend." $RED
    exit 1
fi

write_color_output "Database Connectivity:" $CYAN
CONNECTIVITY_STATUS=$(echo $DB_TEST_RESPONSE | jq -r .database_tests.connectivity.status)
echo "  Status: $CONNECTIVITY_STATUS"
if [ "$CONNECTIVITY_STATUS" == "success" ]; then
    TEST_VALUE=$(echo $DB_TEST_RESPONSE | jq -r .database_tests.connectivity.test_value)
    DB_TIME=$(echo $DB_TEST_RESPONSE | jq -r .database_tests.connectivity.db_time)
    echo "  Test Value: $TEST_VALUE"
    echo "  DB Time: $DB_TIME"
else
    ERROR=$(echo $DB_TEST_RESPONSE | jq -r .database_tests.connectivity.error)
    echo "  Error: $ERROR"
fi
echo ""

write_color_output "Users Table Structure:" $CYAN
TABLE_TEST_STATUS=$(echo $DB_TEST_RESPONSE | jq -r .database_tests.users_table.status)
echo "  Status: $TABLE_TEST_STATUS"
if [ "$TABLE_TEST_STATUS" == "success" ]; then
    echo "  Columns:"
    echo $DB_TEST_RESPONSE | jq -r '.database_tests.users_table.columns[] | "    - \(.column): \(.type)"'
else
    ERROR=$(echo $DB_TEST_RESPONSE | jq -r .database_tests.users_table.error)
    echo "  Error: $ERROR"
fi
echo ""

write_color_output "User Count:" $CYAN
COUNT_TEST_STATUS=$(echo $DB_TEST_RESPONSE | jq -r .database_tests.user_count.status)
echo "  Status: $COUNT_TEST_STATUS"
if [ "$COUNT_TEST_STATUS" == "success" ]; then
    USER_COUNT=$(echo $DB_TEST_RESPONSE | jq -r .database_tests.user_count.user_count)
    echo "  Current Users: $USER_COUNT"
else
    ERROR=$(echo $DB_TEST_RESPONSE | jq -r .database_tests.user_count.error)
    echo "  Error: $ERROR"
fi
echo ""

# Step 3: Listing current users
write_color_output "Step 3: Listing current users..." $YELLOW
USERS_RESPONSE=$(curl -s -m 10 "$BACKEND_URL/api/auth/users")
USER_COUNT=$(echo $USERS_RESPONSE | jq -r .count)
write_color_output "✅ Current user count: $USER_COUNT" $GREEN

if [ $USER_COUNT -gt 0 ]; then
    write_color_output "Existing users:" $CYAN
    echo $USERS_RESPONSE | jq -c '.users[]' | while read user; do
        ID=$(echo $user | jq -r .id)
        NAME=$(echo $user | jq -r .name)
        EMAIL=$(echo $user | jq -r .email)
        ROLE=$(echo $user | jq -r .role)
        CREATED_AT=$(echo $user | jq -r .created_at)
        echo "  - ID: $ID"
        echo "    Name: $NAME"
        echo "    Email: $EMAIL"
        echo "    Role: $ROLE"
        echo "    Created: $CREATED_AT"
        echo ""
    done
else
    write_color_output "No users found in database." $YELLOW
fi
echo ""

# Step 4: Testing user registration
write_color_output "Step 4: Testing user registration..." $YELLOW
TEST_USER_JSON='{
    "name": "Debug Test User",
    "email": "debug-test@conhub.dev",
    "password": "DebugTest123!",
    "organization": "Debug Test Org"
}'

echo "Sending registration request..."
echo "Body: $TEST_USER_JSON"

REGISTER_RESPONSE=$(curl -s -m 30 -X POST -H "Content-Type: application/json" -d "$TEST_USER_JSON" "$BACKEND_URL/api/auth/register")

if echo "$REGISTER_RESPONSE" | jq -e '.user.id' > /dev/null; then
    write_color_output "✅ User registered successfully!" $GREEN
    echo $REGISTER_RESPONSE | jq -r '
        "User ID: \(.user.id)\n" +
        "Email: \(.user.email)\n" +
        "Name: \(.user.name)\n" +
        "Role: \(.user.role)\n" +
        "Subscription: \(.user.subscription_tier)\n" +
        "Verified: \(.user.is_verified)\n" +
        "Created At: \(.user.created_at)"
    '
else
    write_color_output "❌ Registration failed!" $RED
    ERROR=$(echo $REGISTER_RESPONSE | jq -r .error)
    DETAILS=$(echo $REGISTER_RESPONSE | jq -r .details)
    echo "Error: $ERROR"
    if [ "$DETAILS" != "null" ]; then
        echo "Additional Details: $DETAILS"
    fi
fi
echo ""

# Step 5: Checking users after registration attempt
write_color_output "Step 5: Checking users after registration attempt..." $YELLOW
USERS_AFTER_RESPONSE=$(curl -s -m 10 "$BACKEND_URL/api/auth/users")
USER_COUNT_AFTER=$(echo $USERS_AFTER_RESPONSE | jq -r .count)
write_color_output "✅ User count after registration: $USER_COUNT_AFTER" $GREEN

if [ $USER_COUNT_AFTER -gt 0 ]; then
    write_color_output "Users after registration:" $CYAN
    echo $USERS_AFTER_RESPONSE | jq -c '.users[]' | while read user; do
        ID=$(echo $user | jq -r .id)
        NAME=$(echo $user | jq -r .name)
        EMAIL=$(echo $user | jq -r .email)
        ROLE=$(echo $user | jq -r .role)
        CREATED_AT=$(echo $user | jq -r .created_at)
        echo "  - ID: $ID"
        echo "    Name: $NAME"
        echo "    Email: $EMAIL"
        echo "    Role: $ROLE"
        echo "    Created: $CREATED_AT"
        echo ""
    done
else
    write_color_output "⚠️  Still no users found in database after registration attempt!" $YELLOW
fi
echo ""

write_color_output "🏁 Debug script completed!" $BLUE
