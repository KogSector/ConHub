#!/bin/bash

echo -e "\033[0;36m[STATUS] Checking ConHub services...\033[0m"

SERVICES=(
    "Frontend:3000"
    "Backend:3001"
    "Lexor:3002"
    "Doc Search:8001"
    "Langchain Service:8003"
)

for service in "${SERVICES[@]}"; do
    IFS=':' read -r -a arr <<< "$service"
    NAME=${arr[0]}
    PORT=${arr[1]}
    
    if curl -s --head --fail --max-time 2 "http://localhost:$PORT" > /dev/null; then
        echo -e "\033[0;32m✓ $NAME - Running on port $PORT\033[0m"
    else
        echo -e "\033[0;31m✗ $NAME - Not responding on port $PORT\033[0m"
    fi
done
