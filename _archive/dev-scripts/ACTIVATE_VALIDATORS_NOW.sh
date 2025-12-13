#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      SULTAN CHAIN - ACTIVATING VALIDATORS RIGHT NOW           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 1. Start the main API if not running
echo "1️⃣ Checking API..."
if ! ps aux | grep -q "[n]ode.*3030"; then
    cd /workspaces/0xv7/api
    if [ -f "server.js" ]; then
        node server.js > /tmp/sultan_api.log 2>&1 &
        echo "✅ Started API on port 3030"
    fi
else
    echo "✅ API already running"
fi

# 2. Start consensus nodes
echo ""
echo "2️⃣ Starting Validator Nodes..."
cd /workspaces/0xv7

# Check if consensus exists
if [ -f "consensus/consensus.js" ]; then
    for i in 1 2 3; do
        NODE_ID=$i PORT=$((4000 + i)) node consensus/consensus.js > /tmp/node$i.log 2>&1 &
        echo "✅ Started validator node $i on port $((4000 + i))"
    done
fi

# 3. Open validator portal
echo ""
echo "3️⃣ Opening Validator Portal..."
if [ -f "validators/recruitment_portal.html" ]; then
    "$BROWSER" "file:///workspaces/0xv7/validators/recruitment_portal.html"
    echo "✅ Validator portal opened"
fi

echo ""
echo "📊 VALIDATOR STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Test the consensus API
sleep 2
for port in 4001 4002 4003; do
    if curl -s http://localhost:$port/validators 2>/dev/null | grep -q "validators"; then
        echo "✅ Node on port $port: ACTIVE"
    fi
done

echo ""
echo "🎯 NEXT STEPS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. Register validators via portal"
echo "2. Or use Telegram bot (if configured)"
echo "3. Test consensus: curl http://localhost:4001/consensus_state"
