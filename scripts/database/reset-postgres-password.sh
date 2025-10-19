#!/bin/bash

USERNAME="postgres"

echo -e "\033[0;36m[POSTGRES] PostgreSQL Password Reset Helper\033[0m"
echo ""

if ! command -v psql &> /dev/null; then
    echo -e "\033[0;31m[ERROR] PostgreSQL not found in PATH. Please ensure PostgreSQL is installed.\033[0m"
    echo -e "\033[1;33mCommon installation paths:\033[0m"
    echo "  - /usr/lib/postgresql/15/bin"
    echo "  - /usr/lib/postgresql/14/bin"
    echo "  - /usr/lib/postgresql/13/bin"
    exit 1
fi

PG_PATH=$(command -v psql)
echo -e "\033[0;32m[INFO] PostgreSQL found at: $PG_PATH\033[0m"
echo ""

echo -e "\033[0;36mChoose an option:\033[0m"
echo "1. Try common default passwords"
echo "2. Reset password using pg_hba.conf (requires service restart)"
echo "3. Connect without password (trust method)"
echo "4. Use different connection method"
echo ""

read -p "Enter your choice (1-4): " choice

case $choice in
    1)
        echo -e "\033[0;36m[TESTING] Trying common default passwords...\033[0m"
        
        COMMON_PASSWORDS=("" "postgres" "admin" "password" "root" "123456")
        
        for password in "${COMMON_PASSWORDS[@]}"; do
            echo -e "\033[1;33mTrying password: '$password'\033[0m"
            
            export PGPASSWORD=$password
            psql -h localhost -U "$USERNAME" -d postgres -c "SELECT version();" &> /dev/null
            
            if [ $? -eq 0 ]; then
                echo -e "\033[0;32m[SUCCESS] Password found: '$password'\033[0m"
                echo -e "\033[0;32mYou can now use: ./clear-database.sh -Password '$password'\033[0m"
                unset PGPASSWORD
                exit 0
            fi
        done
        
        echo -e "\033[0;31m[FAILED] None of the common passwords worked\033[0m"
        unset PGPASSWORD
        ;;
    
    2)
        echo -e "\033[0;36m[INFO] To reset password using pg_hba.conf:\033[0m"
        echo "1. Stop PostgreSQL service (e.g., sudo systemctl stop postgresql)"
        echo "2. Edit pg_hba.conf and change 'md5' or 'scram-sha-256' to 'trust' for local connections"
        echo "3. Start PostgreSQL service (e.g., sudo systemctl start postgresql)"
        echo "4. Connect and change password: ALTER USER postgres PASSWORD 'newpassword';"
        echo "5. Change pg_hba.conf back to its original setting"
        echo "6. Restart service"
        echo ""
        echo -e "\033[1;33mpg_hba.conf location: /etc/postgresql/[version]/main/pg_hba.conf\033[0m"
        ;;
    
    3)
        echo -e "\033[0;36m[TESTING] Trying to connect without password...\033[0m"
        
        psql -h localhost -U "$USERNAME" -d postgres -c "SELECT version();" &> /dev/null
        
        if [ $? -eq 0 ]; then
            echo -e "\033[0;32m[SUCCESS] Connected without password!\033[0m"
            echo -e "\033[0;32mYou can now run: ./clear-database.sh -Password ''\033[0m"
        else
            echo -e "\033[0;31m[FAILED] Cannot connect without password\033[0m"
        fi
        ;;
    
    4)
        echo -e "\033[0;36m[INFO] Alternative connection methods:\033[0m"
        echo "1. Use pgAdmin (GUI tool)"
        echo "2. Use different username (try: postgres, admin, root)"
        echo "3. Check if PostgreSQL is running on different port (5433, 5434, etc.)"
        echo "4. Use Docker PostgreSQL: docker run -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres"
        ;;
    
    *)
        echo -e "\033[0;31m[ERROR] Invalid choice\033[0m"
        ;;
esac

echo ""
echo -e "\033[0;36m[TIP] You can also try:\033[0m"
echo "./clear-database.sh -Username 'your_username' -Password 'your_password'"
