#!/bin/bash
# ConHub Database Connection Test Script
# Run this script to test your PostgreSQL database connection

echo "ConHub Database Connection Test"
echo "==============================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check if PostgreSQL is running
if command -v systemctl &> /dev/null; then
    if systemctl is-active --quiet postgresql; then
        echo -e "${GREEN}✅ PostgreSQL service is running${NC}"
    else
        echo -e "${RED}❌ PostgreSQL service is not running. Please start PostgreSQL first.${NC}"
        echo "Try: sudo systemctl start postgresql"
        exit 1
    fi
elif command -v brew &> /dev/null; then
    if brew services list | grep postgresql | grep started &> /dev/null; then
        echo -e "${GREEN}✅ PostgreSQL service is running${NC}"
    else
        echo -e "${RED}❌ PostgreSQL service is not running. Please start PostgreSQL first.${NC}"
        echo "Try: brew services start postgresql"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  Cannot check PostgreSQL service status. Assuming it's running...${NC}"
fi

# Check if psql is accessible
if command -v psql &> /dev/null; then
    echo -e "${GREEN}✅ psql command is accessible${NC}"
else
    echo -e "${RED}❌ psql command not found. Please install PostgreSQL client tools.${NC}"
    exit 1
fi

# Get password once and set environment variable
echo -e "\n${YELLOW}Enter your PostgreSQL password (you'll only need to enter it once):${NC}"
read -s -p "Password for user 'postgres': " password
echo ""

# Set PGPASSWORD environment variable for this session
export PGPASSWORD="$password"

# Test if ConHub database exists
echo -e "\n${YELLOW}Checking if ConHub database exists...${NC}"
if psql -U postgres -lqt | cut -d \| -f 1 | grep -qw conhub; then
    echo -e "${GREEN}✅ ConHub database exists${NC}"
else
    echo -e "${YELLOW}❌ ConHub database not found. Creating it now...${NC}"
    if psql -U postgres -c "CREATE DATABASE conhub;" 2>/dev/null; then
        echo -e "${GREEN}✅ ConHub database created successfully${NC}"
    else
        echo -e "${RED}❌ Failed to create ConHub database${NC}"
        unset PGPASSWORD
        exit 1
    fi
fi

# Test database schema
echo -e "\n${YELLOW}Checking database schema...${NC}"
table_count=$(psql -U postgres -d conhub -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" 2>/dev/null | xargs)
if [ "$table_count" -gt 0 ] 2>/dev/null; then
    echo -e "${GREEN}✅ Database schema exists ($table_count tables found)${NC}"
    
    # List tables
    echo -e "\n${CYAN}Tables in ConHub database:${NC}"
    psql -U postgres -d conhub -c "\dt"
else
    echo -e "${YELLOW}⚠️  No tables found. Run the schema setup:${NC}"
    echo -e "${CYAN}psql -U postgres -d conhub -f database/schema.sql${NC}"
fi

# Ask to update .env file
echo ""
read -p "Do you want to update the .env file with your PostgreSQL password? (y/N): " update_env
if [[ $update_env =~ ^[Yy]$ ]]; then
    # Update .env file
    if [ -f ".env" ]; then
        sed -i.bak "s|DATABASE_URL=postgresql://postgres:.*@localhost:5432/conhub|DATABASE_URL=postgresql://postgres:$password@localhost:5432/conhub|" .env
        echo -e "${GREEN}✅ Updated .env file with your database password${NC}"
    else
        echo -e "${RED}❌ .env file not found${NC}"
    fi
fi

# Clear password from environment
unset PGPASSWORD

echo -e "\n${GREEN}🎉 Database test complete!${NC}"
echo -e "You can now run the backend with: ${CYAN}cargo run --bin conhub-backend${NC}"