#!/bin/bash
# ConHub Database Setup Script
# This script creates the ConHub database and applies the schema

echo "ConHub Database Setup"
echo "====================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Check if PostgreSQL is running
if command -v systemctl &> /dev/null; then
    if ! systemctl is-active --quiet postgresql; then
        echo -e "${RED}❌ PostgreSQL service is not running. Please start PostgreSQL first.${NC}"
        echo "Try: sudo systemctl start postgresql"
        exit 1
    fi
elif command -v brew &> /dev/null; then
    if ! brew services list | grep postgresql | grep started &> /dev/null; then
        echo -e "${RED}❌ PostgreSQL service is not running. Please start PostgreSQL first.${NC}"
        echo "Try: brew services start postgresql"
        exit 1
    fi
fi

echo -e "${GREEN}✅ PostgreSQL service is running${NC}"

# Create database if it doesn't exist
echo -e "\n${YELLOW}Creating ConHub database...${NC}"
if psql -U postgres -c "CREATE DATABASE conhub;" 2>/dev/null || psql -U postgres -c "SELECT 1;" -d conhub &>/dev/null; then
    echo -e "${GREEN}✅ ConHub database ready${NC}"
else
    echo -e "${RED}❌ Failed to create database${NC}"
    exit 1
fi

# Apply schema
if [ -f "database/schema.sql" ]; then
    echo -e "\n${YELLOW}Applying database schema...${NC}"
    if psql -U postgres -d conhub -f "database/schema.sql" &>/dev/null; then
        echo -e "${GREEN}✅ Database schema applied successfully${NC}"
    else
        echo -e "${RED}❌ Failed to apply schema${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Schema file not found: database/schema.sql${NC}"
    exit 1
fi

# Verify setup
echo -e "\n${YELLOW}Verifying database setup...${NC}"
table_count=$(psql -U postgres -d conhub -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" 2>/dev/null | xargs)
if [ "$table_count" -gt 0 ] 2>/dev/null; then
    echo -e "${GREEN}✅ Database setup complete! Found $table_count tables${NC}"
    
    # Show tables
    echo -e "\n${CYAN}Created tables:${NC}"
    psql -U postgres -d conhub -c "\dt"
else
    echo -e "${RED}❌ Database setup verification failed${NC}"
    exit 1
fi

echo -e "\n${GREEN}🎉 ConHub database is ready!${NC}"
echo -e "${CYAN}Next steps:${NC}"
echo -e "${WHITE}1. Update your .env file with the correct PostgreSQL password${NC}"
echo -e "${WHITE}2. Run: ./scripts/test-database.sh to test the connection${NC}"
echo -e "${WHITE}3. Start the backend: cargo run --bin conhub-backend${NC}"