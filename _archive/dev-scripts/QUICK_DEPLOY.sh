#!/bin/bash

echo "🚀 SULTAN CHAIN - QUICK DEPLOYMENT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

case "${1:-help}" in
    start)
        echo "Starting Sultan Chain..."
        # Kill any existing processes
        lsof -ti:8080 | xargs kill -9 2>/dev/null || true
        lsof -ti:3000 | xargs kill -9 2>/dev/null || true
        
        # Start blockchain
        cd /workspaces/0xv7/working-chain
        if [ -f "./sultan-chain" ]; then
            ./sultan-chain > /tmp/sultan-chain.log 2>&1 &
            echo "✅ Blockchain started on port 8080"
        else
            echo "⚠️  Building blockchain first..."
            go build -o sultan-chain main.go
            ./sultan-chain > /tmp/sultan-chain.log 2>&1 &
            echo "✅ Blockchain built and started"
        fi
        
        # Start web server
        cd /workspaces/0xv7/public
        python3 -m http.server 3000 > /tmp/web.log 2>&1 &
        echo "✅ Dashboard started on port 3000"
        
        sleep 2
        echo ""
        echo "🌐 Access points:"
        echo "   • Dashboard: http://localhost:3000/live-blockchain.html"
        echo "   • API: http://localhost:8080/status"
        echo "   • Test Results: http://localhost:3000/test-results.html"
        ;;
        
    stop)
        echo "Stopping Sultan Chain..."
        lsof -ti:8080 | xargs kill -9 2>/dev/null || true
        lsof -ti:3000 | xargs kill -9 2>/dev/null || true
        echo "✅ All services stopped"
        ;;
        
    status)
        /workspaces/0xv7/FINAL_STATUS_REPORT.sh
        ;;
        
    test)
        echo "Running tests..."
        cd /workspaces/0xv7
        npm test
        ;;
        
    logs)
        echo "📜 Recent blockchain logs:"
        tail -n 20 /tmp/sultan-chain.log 2>/dev/null || echo "No logs available"
        ;;
        
    *)
        echo "Usage: ./QUICK_DEPLOY.sh [command]"
        echo ""
        echo "Commands:"
        echo "  start   - Start blockchain and dashboard"
        echo "  stop    - Stop all services"
        echo "  status  - Show system status"
        echo "  test    - Run test suite"
        echo "  logs    - Show recent logs"
        ;;
esac

