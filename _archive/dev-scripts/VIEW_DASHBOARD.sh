#!/bin/bash

echo "🌐 Opening Sultan Chain Dashboard..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if services are running
RUNNING=0
for port in 3000 3030 4001 5001 5002 5003; do
    nc -z localhost $port 2>/dev/null && ((RUNNING++))
done

if [ $RUNNING -eq 0 ]; then
    echo "⚠️  Services are not running. Starting them first..."
    /workspaces/0xv7/PYTHON_SERVICES.sh
    echo ""
fi

echo "✅ Opening dashboard in browser..."
"$BROWSER" /workspaces/0xv7/production_dashboard.html &

echo ""
echo "Dashboard Features:"
echo "  • Real-time block updates"
echo "  • Service status monitoring"
echo "  • Network metrics"
echo "  • Activity log"
echo ""
echo "Alternative dashboards:"
echo "  $BROWSER /workspaces/0xv7/sultan_dashboard.html"
echo "  $BROWSER /workspaces/0xv7/dashboard.html"

