#!/bin/bash
# Test multi-validator consensus

echo "Testing consensus across validators..."

for i in {1..5}; do
    PORT=$((26657 + i - 1))
    echo -n "Validator #$i (port $PORT): "
    
    HEIGHT=$(curl -s "http://localhost:$PORT/status" | jq -r '.height // "ERROR"')
    echo "Block $HEIGHT"
    
    sleep 0.5
done

echo ""
echo "If all validators show the same (or very close) block height, consensus is working!"
