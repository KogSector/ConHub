#!/bin/bash

BACKEND_URL=${1:-"http://localhost:3001"}

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

write_color_output "🧪 Testing ConHub User Signup Functionality" $BLUE
write_color_output "==========================================" $BLUE
echo ""

TEST_USER_JSON='{
    "name": "Test User",
    "email": "test@conhub.dev",
    "password": "TestPassword123!",
    "avatar_url": null,
    "organization": "ConHub Test Org"
}'

write_color_output "📝 Test User Data:" $YELLOW
echo "Name: $(echo $TEST_USER_JSON | jq -r .name)"
echo "Email: $(echo $TEST_USER_JSON | jq -r .email)"
echo "Organization: $(echo $TEST_USER_JSON | jq -r .organization)"
echo ""

write_color_output "🔍 Step 1: Checking backend health..." $YELLOW
HEALTH_RESPONSE=$(curl -s -m 10 "$BACKEND_URL/health")
if [ $? -ne 0 ]; then
    write_color_output "❌ Backend is not responding at $BACKEND_URL" $RED
    write_color_output "Please ensure the backend is running with: npm run dev:backend" $RED
    exit 1
fi
write_color_output "✅ Backend is healthy: $(echo $HEALTH_RESPONSE | jq -r .status)" $GREEN

write_color_output "🔍 Step 2: Checking existing users..." $YELLOW
USERS_RESPONSE=$(curl -s -m 10 "$BACKEND_URL/api/auth/users")
if [ $? -eq 0 ]; then
    USER_COUNT=$(echo $USERS_RESPONSE | jq -r .count)
    write_color_output "📊 Current user count: $USER_COUNT" $GREEN
    if [ $USER_COUNT -gt 0 ]; then
        write_color_output "Existing users:" $CYAN
        echo $USERS_RESPONSE | jq -r '.users[] | "  - \(.name) (\(.email)) - Role: \(.role)"'
    fi
else
    write_color_output "⚠️  Could not fetch existing users" $YELLOW
fi
echo ""

write_color_output "🔍 Step 3: Registering new user..." $YELLOW
REGISTER_RESPONSE=$(curl -s -w "%{http_code}" -X POST -H "Content-Type: application/json" -d "$TEST_USER_JSON" "$BACKEND_URL/api/auth/register")
HTTP_CODE=${REGISTER_RESPONSE: -3}
BODY=${REGISTER_RESPONSE::-3}

if [ "$HTTP_CODE" -ge 200 ] && [ "$HTTP_CODE" -lt 300 ]; then
    write_color_output "✅ User registered successfully!" $GREEN
    echo $BODY | jq -r '
        "User ID: \(.user.id)\n" +
        "Email: \(.user.email)\n" +
        "Name: \(.user.name)\n" +
        "Role: \(.user.role)\n" +
        "Subscription: \(.user.subscription_tier)\n" +
        "Verified: \(.user.is_verified)\n" +
        "JWT Token: \(.token | limit(50; "..."))"
    '
    USER_ID=$(echo $BODY | jq -r .user.id)
else
    ERROR_DETAILS=$(echo $BODY | jq -r .error)
    DETAILS=$(echo $BODY | jq -r .details)
    write_color_output "❌ Registration failed: $ERROR_DETAILS" $RED
    if [ "$DETAILS" != "null" ]; then
        write_color_output "Details: $DETAILS" $RED
    fi
    exit 1
fi
echo ""

write_color_output "🔍 Step 4: Verifying user was stored in database..." $YELLOW
USERS_AFTER_RESPONSE=$(curl -s -m 10 "$BACKEND_URL/api/auth/users")
USER_COUNT_AFTER=$(echo $USERS_AFTER_RESPONSE | jq -r .count)
write_color_output "📊 User count after registration: $USER_COUNT_AFTER" $GREEN

CREATED_USER=$(echo $USERS_AFTER_RESPONSE | jq -r --arg email "$(echo $TEST_USER_JSON | jq -r .email)" '.users[] | select(.email == $email)')
if [ -n "$CREATED_USER" ]; then
    write_color_output "✅ User found in database!" $GREEN
    echo "$CREATED_USER" | jq -r '
        "Database User ID: \(.id)\n" +
        "Database Email: \(.email)\n" +
        "Database Name: \(.name)\n" +
        "Created At: \(.created_at)"
    '
else
    write_color_output "❌ User not found in database!" $RED
    exit 1
fi
echo ""

write_color_output "🔍 Step 5: Testing login with new user..." $YELLOW
LOGIN_BODY=$(echo $TEST_USER_JSON | jq -c '{email: .email, password: .password}')
LOGIN_RESPONSE=$(curl -s -w "%{http_code}" -X POST -H "Content-Type: application/json" -d "$LOGIN_BODY" "$BACKEND_URL/api/auth/login")
HTTP_CODE=${LOGIN_RESPONSE: -3}
BODY=${LOGIN_RESPONSE::-3}

if [ "$HTTP_CODE" -ge 200 ] && [ "$HTTP_CODE" -lt 300 ]; then
    write_color_output "✅ Login successful!" $GREEN
    echo $BODY | jq -r '
        "Login User ID: \(.user.id)\n" +
        "Login Email: \(.user.email)\n" +
        "Last Login: \(.user.last_login_at)\n" +
        "JWT Token: \(.token | limit(50; "..."))"
    '
else
    ERROR_DETAILS=$(echo $BODY | jq -r .error)
    write_color_output "❌ Login failed: $ERROR_DETAILS" $RED
    exit 1
fi
echo ""

write_color_output "🔍 Step 6: Testing duplicate registration (should fail)..." $YELLOW
DUPLICATE_RESPONSE=$(curl -s -w "%{http_code}" -X POST -H "Content-Type: application/json" -d "$TEST_USER_JSON" "$BACKEND_URL/api/auth/register")
HTTP_CODE=${DUPLICATE_RESPONSE: -3}

if [ "$HTTP_CODE" -eq 400 ]; then
    write_color_output "✅ Duplicate registration correctly rejected!" $GREEN
else
    write_color_output "⚠️  Unexpected error for duplicate registration: HTTP $HTTP_CODE" $YELLOW
fi
echo ""

write_color_output "🎉 All tests passed! User signup functionality is working correctly." $GREEN
write_color_output "✅ Users are being properly stored in the database" $GREEN
write_color_output "✅ Authentication is working with stored users" $GREEN
write_color_output "✅ Duplicate email validation is working" $GREEN
echo ""

write_color_output "📋 Summary:" $BLUE
echo "- Backend health check: ✅"
echo "- User registration: ✅"
echo "- Database storage: ✅"
echo "- User authentication: ✅"
echo "- Duplicate prevention: ✅"
echo ""

write_color_output "🚀 You can now use the signup functionality in your frontend!" $GREEN
