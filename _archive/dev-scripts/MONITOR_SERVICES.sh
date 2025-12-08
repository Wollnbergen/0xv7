#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          SULTAN CHAIN - SERVICE MONITOR                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

while true; do
    # Check and restart web server if needed
    if ! lsof -i:3000 > /dev/null 2>&1; then
        echo "⚠️  Web server down, restarting..."
        cd /workspaces/0xv7/public && python3 -m http.server 3000 > /tmp/web.log 2>&1 &
    fi
    
    # Check and restart API server if needed  
    if ! lsof -i:1317 > /dev/null 2>&1; then
        echo "⚠️  API server down, restarting..."
        cd /workspaces/0xv7 && node server/api.js > /tmp/api.log 2>&1 &
    fi
    
    # Show status
    echo -n "📊 Status [$(date '+%H:%M:%S')]: "
    
    if lsof -i:3000 > /dev/null 2>&1; then
        echo -n "Web ✅ "
    else
        echo -n "Web ❌ "
    fi
    
    if lsof -i:1317 > /dev/null 2>&1; then
        echo -n "API ✅"
    else
        echo -n "API ❌"
    fi
    
    echo ""
    
    # Wait 10 seconds before next check
    sleep 10
done

