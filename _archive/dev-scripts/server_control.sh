#!/bin/bash

case "$1" in
    start)
        echo "Starting Sultan Chain server..."
        export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
        RUST_LOG=info cargo run -p sultan-coordinator --bin rpc_server > /tmp/sultan.log 2>&1 &
        echo "Server started with PID: $!"
        ;;
    stop)
        PID=$(pgrep -f 'cargo.*rpc_server' | head -1)
        if [ -n "$PID" ]; then
            echo "Stopping server (PID: $PID)..."
            kill $PID
            echo "Server stopped."
        else
            echo "Server not running."
        fi
        ;;
    status)
        PID=$(pgrep -f 'cargo.*rpc_server' | head -1)
        if [ -n "$PID" ]; then
            echo "✅ Server running (PID: $PID)"
            echo "   RPC: http://127.0.0.1:3030"
            echo "   Metrics: http://127.0.0.1:9100/metrics"
        else
            echo "❌ Server not running"
        fi
        ;;
    logs)
        tail -f /tmp/sultan.log
        ;;
    test)
        echo "Running quick test..."
        export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
        TOKEN=$(cargo run -q -p sultan-coordinator --bin jwt_gen prod 3600 2>/dev/null)
        echo "Testing wallet creation..."
        curl -sS -X POST http://127.0.0.1:3030 \
            -H "Authorization: Bearer $TOKEN" \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"wallet_create","params":["quick_test"],"id":1}' | jq .
        ;;
    *)
        echo "Usage: $0 {start|stop|status|logs|test}"
        exit 1
        ;;
esac
