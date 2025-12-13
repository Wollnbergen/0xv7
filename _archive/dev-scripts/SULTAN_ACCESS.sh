#!/bin/bash

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║                    SULTAN CHAIN QUICK ACCESS                        ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""
echo "🌐 OPENING WEB INTERFACE..."
echo ""
echo "Local URL:    http://localhost:3000"
echo "External URL: https://orange-telegram-pj6qgwgv59jjfrj9j-3000.app.github.dev"
echo ""

# Ensure web server is running
if ! pgrep -f "python3 -m http.server 3000" > /dev/null; then
    echo "Starting web server..."
    cd /workspaces/0xv7/public && python3 -m http.server 3000 > /tmp/web.log 2>&1 &
    sleep 2
fi

# Open in browser
"$BROWSER" http://localhost:3000

echo "✅ Web interface opened in browser!"
echo ""
echo "Other commands:"
echo "  ./SULTAN_LIVE_DASHBOARD.sh - Interactive dashboard"
echo "  ./VIEW_SULTAN_STATUS.sh    - Status report"
echo "  tail -f /tmp/web.log       - View logs"
