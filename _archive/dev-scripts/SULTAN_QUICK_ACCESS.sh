#!/bin/bash
echo "🚀 Sultan Chain Quick Access"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━"
for port in 3000 3001 3030 5000 8080; do
    if lsof -i:$port > /dev/null 2>&1; then
        URL="http://localhost:$port"
        echo "✅ Opening: $URL"
        "$BROWSER" "$URL"
        break
    fi
done
