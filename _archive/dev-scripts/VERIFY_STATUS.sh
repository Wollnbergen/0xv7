#!/bin/bash

echo "🔍 Verifying Sultan Chain Status..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

SERVICES_UP=0
TOTAL_SERVICES=6

# Check each service
for port in 3000 3030 4001 5001 5002 5003; do
    if nc -z localhost $port 2>/dev/null; then
        ((SERVICES_UP++))
    fi
done

echo "✅ Services Running: $SERVICES_UP/$TOTAL_SERVICES"

if [ $SERVICES_UP -eq 6 ]; then
    echo "🎉 All services are operational!"
    echo ""
    echo "Quick Actions:"
    echo "  • Control Panel: /workspaces/0xv7/SULTAN_CONTROL.sh"
    echo "  • View Dashboard: $BROWSER /workspaces/0xv7/production_dashboard.html"
else
    echo "⚠️ Some services are down. Restarting..."
    /workspaces/0xv7/PYTHON_SERVICES.sh
fi

