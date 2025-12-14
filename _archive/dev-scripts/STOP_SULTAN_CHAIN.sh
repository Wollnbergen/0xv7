#!/bin/bash
# Stop all Sultan Chain instances

echo "🛑 Stopping Sultan Chain..."

# Kill all sultand processes
pkill -f sultand || true

# Also kill any processes on common ports
for port in 8080 8081 8082; do
    if lsof -iTCP:$port -sTCP:LISTEN -P -n >/dev/null 2>&1; then
        echo "Killing process on port $port..."
        fuser -k ${port}/tcp 2>/dev/null || true
    fi
done

echo "✅ Sultan Chain stopped"
