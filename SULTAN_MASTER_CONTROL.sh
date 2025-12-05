#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - MASTER CONTROL PANEL                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📅 $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Function to check service status
check_service() {
    local name=$1
    local check_cmd=$2
    local status="❌ Not Running"
    
    if eval $check_cmd > /dev/null 2>&1; then
        status="✅ Running"
    fi
    
    echo "  $name: $status"
}

# 1. Check all services
echo "📊 SERVICE STATUS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
check_service "Testnet API" "curl -s http://localhost:3030"
check_service "ScyllaDB" "docker ps | grep -q scylla"
check_service "Redis" "docker ps | grep -q redis"
check_service "Mainnet Node" "pgrep -f sultan-mainnet"

echo ""
echo "🚀 QUICK ACTIONS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "1) Start ALL Services"
echo "2) Start Testnet API Only"
echo "3) Start Mainnet Node Only"
echo "4) Open Testnet UI in Browser"
echo "5) Test Economics API"
echo "6) View Logs"
echo "7) Stop All Services"
echo "8) Exit"
echo ""
read -p "Select an option (1-8): " choice

case $choice in
    1)
        echo "🚀 Starting all services..."
        
        # Start Docker services
        docker start scylla redis 2>/dev/null || {
            docker run --name scylla -d -p 9042:9042 scylladb/scylla
            docker run --name redis -d -p 6379:6379 redis:alpine
        }
        
        # Start API if not running
        if ! curl -s http://localhost:3030 > /dev/null 2>&1; then
            cd /workspaces/0xv7/api && node simple_server.js > /tmp/api.log 2>&1 &
            sleep 2
            echo "✅ Started Testnet API"
        else
            echo "✅ API already running"
        fi
        
        echo "✅ All services started!"
        ;;
        
    2)
        echo "🌐 Starting Testnet API..."
        if ! curl -s http://localhost:3030 > /dev/null 2>&1; then
            cd /workspaces/0xv7/api && node simple_server.js > /tmp/api.log 2>&1 &
            sleep 2
            echo "✅ API started on port 3030"
        else
            echo "✅ API already running"
        fi
        ;;
        
    3)
        echo "⛓️ Starting Mainnet Node..."
        cd /workspaces/0xv7/sultan_mainnet
        ./target/release/sultan-mainnet
        ;;
        
    4)
        echo "🌐 Opening Testnet UI..."
        "$BROWSER" https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/
        ;;
        
    5)
        echo "📊 Testing Economics API..."
        if curl -s http://localhost:3030 > /dev/null 2>&1; then
            curl -s -X POST http://localhost:3030 \
                -H 'Content-Type: application/json' \
                -d '{"jsonrpc":"2.0","method":"get_economics","id":1}' | jq '.result'
        else
            echo "❌ API not running. Start it first with option 2"
        fi
        ;;
        
    6)
        echo "📋 Recent logs:"
        if [ -f /tmp/api.log ]; then
            tail -20 /tmp/api.log
        elif [ -f /tmp/sultan-api-fixed.log ]; then
            tail -20 /tmp/sultan-api-fixed.log
        else
            echo "No logs found"
        fi
        ;;
        
    7)
        echo "🛑 Stopping all services..."
        pkill -f "node.*simple_server" 2>/dev/null
        pkill -f "sultan-mainnet" 2>/dev/null
        docker stop scylla redis 2>/dev/null
        echo "✅ All services stopped"
        ;;
        
    8)
        echo "👋 Goodbye!"
        exit 0
        ;;
        
    *)
        echo "⚠️ Invalid option"
        ;;
esac

echo ""
echo "Press Enter to continue..."
read

# Re-run the script
exec "$0"

