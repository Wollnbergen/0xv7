#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              TESTING SOVEREIGN DASHBOARD                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if web server is running
if lsof -i:3000 > /dev/null 2>&1; then
    echo "✅ Web server is running on port 3000"
else
    echo "🔄 Starting web server..."
    cd /workspaces/0xv7/public
    python3 -m http.server 3000 > /tmp/web.log 2>&1 &
    sleep 2
fi

echo ""
echo "📊 Available Dashboards:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Original Sultan Dashboard:"
echo "   $BROWSER http://localhost:3000"
echo ""
echo "2. Sovereign Chain Dashboard:"
echo "   $BROWSER http://localhost:3000/sovereign-dashboard.html"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Open both dashboards
"$BROWSER" http://localhost:3000 &
sleep 1
"$BROWSER" http://localhost:3000/sovereign-dashboard.html &

echo ""
echo "✅ Dashboards opened in browser!"

