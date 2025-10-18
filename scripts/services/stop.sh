#!/bin/bash

echo -e "\033[1;33m[STOP] Stopping ConHub services...\033[0m"

SCRIPT_DIR=$(dirname "$0")
"$SCRIPT_DIR/../maintenance/force-stop.sh"

echo -e "\033[0;32m[OK] All services stopped\033[0m"
