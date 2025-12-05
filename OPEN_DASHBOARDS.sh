#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - OPENING NETWORK DASHBOARDS             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "🌐 Opening dashboards in browser..."

# Open validator portal
"$BROWSER" "file:///workspaces/0xv7/validators/recruitment_portal.html" &

# Open live dashboard
"$BROWSER" "file:///workspaces/0xv7/live_network_dashboard.html" &

echo "✅ Dashboards opened!"
echo ""
echo "If they don't appear, manually open:"
echo "  • file:///workspaces/0xv7/validators/recruitment_portal.html"
echo "  • file:///workspaces/0xv7/live_network_dashboard.html"
