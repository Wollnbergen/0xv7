#!/bin/bash
# Start all genesis validators

echo "Starting 5 validators..."

for i in $(seq 1 5); do
    echo "Starting validator #$i..."
    bash "/workspaces/0xv7/genesis-validators/start-validator-$i.sh" > "/workspaces/0xv7/genesis-validators/validator-$i.log" 2>&1 &
    PID=$!
    echo "$PID" > "/workspaces/0xv7/genesis-validators/validator-$i.pid"
    echo "  PID: $PID"
    sleep 1
done

echo ""
echo "All validators started!"
echo "Test consensus: bash /workspaces/0xv7/genesis-validators/test-consensus.sh"
