#!/bin/bash
echo "SULTAN CHAIN STATUS"
echo "==================="
PID=$(pgrep -f 'cargo.*rpc_server' | head -1)
if [ -n "$PID" ]; then
    echo "✅ Server: Running (PID: $PID)"
    echo "   RPC: http://127.0.0.1:3030"
    echo "   Metrics: http://127.0.0.1:9100/metrics"
else
    echo "❌ Server: Not running"
fi
echo ""
echo "Day 3-4: ✅ COMPLETE"
echo "Next: Day 5-6 - Token Economics"
