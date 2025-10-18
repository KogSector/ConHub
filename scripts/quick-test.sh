#!/bin/bash

BACKEND_URL="http://localhost:3001"

# Color definitions
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}Testing ConHub Backend...${NC}"
echo ""

echo -e "${YELLOW}1. Checking backend health...${NC}"
HEALTH=$(curl -s -m 5 "$BACKEND_URL/health")
if [ $? -ne 0 ]; then
    echo -e "   ${RED}Backend not responding: Connection failed${NC}"
    exit 1
fi
HEALTH_STATUS=$(echo $HEALTH | jq -r .status)
DB_STATUS=$(echo $HEALTH | jq -r .database)
echo -e "   ${GREEN}Backend Status: $HEALTH_STATUS${NC}"
echo -e "   ${GREEN}Database Status: $DB_STATUS${NC}"

echo -e "${YELLOW}2. Testing database operations...${NC}"
DB_TEST=$(curl -s -m 10 "$BACKEND_URL/api/auth/test-db")
if [ $? -eq 0 ]; then
    CONNECTIVITY=$(echo $DB_TEST | jq -r .database_tests.connectivity.status)
    USER_COUNT=$(echo $DB_TEST | jq -r .database_tests.user_count.user_count)
    echo -e "   ${GREEN}Database Connectivity: $CONNECTIVITY${NC}"
    echo -e "   ${GREEN}Current User Count: $USER_COUNT${NC}"
else
    echo -e "   ${RED}Database test failed: Request failed${NC}"
fi

echo -e "${YELLOW}3. Listing current users...${NC}"
USERS=$(curl -s -m 5 "$BACKEND_URL/api/auth/users")
if [ $? -eq 0 ]; then
    USER_COUNT=$(echo $USERS | jq -r .count)
    echo -e "   ${GREEN}Users found: $USER_COUNT${NC}"
    if [ $USER_COUNT -gt 0 ]; then
        echo $USERS | jq -r '.users[] | "   - \(.name) (\(.email))"' | while read -r line; do
            echo -e "   ${CYAN}$line${NC}"
        done
    fi
else
    echo -e "   ${RED}Failed to list users: Request failed${NC}"
fi

echo -e "${YELLOW}4. Testing user registration...${NC}"
TEST_USER='{
    "name": "Quick Test User",
    "email": "quicktest@conhub.dev",
    "password": "QuickTest123!",
    "organization": "Test Org"
}'
REGISTER=$(curl -s -m 10 -X POST -H "Content-Type: application/json" -d "$TEST_USER" "$BACKEND_URL/api/auth/register")
if [ $? -eq 0 ] && echo "$REGISTER" | jq -e '.user.id' > /dev/null; then
    USER_ID=$(echo $REGISTER | jq -r .user.id)
    EMAIL=$(echo $REGISTER | jq -r .user.email)
    echo -e "   ${GREEN}Registration successful!${NC}"
    echo -e "   ${CYAN}User ID: $USER_ID${NC}"
    echo -e "   ${CYAN}Email: $EMAIL${NC}"
else
    ERROR_BODY=$(echo $REGISTER | jq -r .)
    echo -e "   ${RED}Registration failed: $ERROR_BODY${NC}"
fi

echo ""
echo -e "${BLUE}Test completed!${NC}"
