#!/bin/bash
# Stop all genesis validators

echo "Stopping all validators..."

for i in $(seq 1 5); do
    if [ -f "/workspaces/0xv7/genesis-validators/validator-$i.pid" ]; then
        PID=$(cat "/workspaces/0xv7/genesis-validators/validator-$i.pid")
        echo "Stopping validator #$i (PID: $PID)..."
        kill -TERM $PID 2>/dev/null || true
        rm -f "/workspaces/0xv7/genesis-validators/validator-$i.pid"
    fi
done

echo "All validators stopped"
