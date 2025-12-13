#!/bin/bash
# Deploy All Bridge Services to Production
# Usage: ./deploy-bridges.sh [environment]

set -e

ENVIRONMENT=${1:-production}
COMPOSE_FILE="docker-compose.yml"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║        🌉 SULTAN L1 - BRIDGE DEPLOYMENT SCRIPT 🌉              ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Environment: $ENVIRONMENT"
echo "Compose File: $COMPOSE_FILE"
echo ""

# Check prerequisites
echo "1️⃣  Checking prerequisites..."
command -v docker >/dev/null 2>&1 || { echo "❌ Docker not found. Please install Docker."; exit 1; }
command -v docker-compose >/dev/null 2>&1 || { echo "❌ Docker Compose not found. Please install Docker Compose."; exit 1; }
echo "✅ Prerequisites satisfied"

# Stop existing services
echo ""
echo "2️⃣  Stopping existing services..."
docker-compose -f $COMPOSE_FILE down || true
echo "✅ Existing services stopped"

# Build images
echo ""
echo "3️⃣  Building Docker images..."
docker-compose -f $COMPOSE_FILE build --parallel
echo "✅ Images built successfully"

# Start services
echo ""
echo "4️⃣  Starting bridge services..."
docker-compose -f $COMPOSE_FILE up -d
echo "✅ Services started"

# Wait for services to be healthy
echo ""
echo "5️⃣  Waiting for services to be healthy..."
sleep 10

# Check service status
echo ""
echo "6️⃣  Checking service health..."
echo ""

check_service() {
    SERVICE=$1
    PORT=$2
    if docker ps | grep -q "$SERVICE"; then
        echo "  ✅ $SERVICE - Running"
    else
        echo "  ❌ $SERVICE - NOT RUNNING"
        return 1
    fi
}

check_service "sultan-bitcoin-bridge" "9001"
check_service "sultan-ethereum-bridge" "50051"
check_service "sultan-solana-bridge" "50052"
check_service "sultan-ton-bridge" "9004"
check_service "sultan-ibc-relayer" "3000"
check_service "sultan-node" "26657"

# Setup IBC relayer
echo ""
echo "7️⃣  Setting up IBC relayer..."
if [ -f "./setup-ibc-relayer.sh" ]; then
    chmod +x ./setup-ibc-relayer.sh
    # ./setup-ibc-relayer.sh  # Uncomment when mnemonics are configured
    echo "⚠️  IBC setup script ready (configure mnemonics first)"
else
    echo "⚠️  IBC setup script not found"
fi

# Display service endpoints
echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║            ✅ DEPLOYMENT COMPLETE - ALL SERVICES RUNNING       ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "🔌 Service Endpoints:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Bitcoin Bridge:    http://localhost:9001"
echo "  Ethereum Bridge:   grpc://localhost:50051"
echo "  Solana Bridge:     grpc://localhost:50052"
echo "  TON Bridge:        http://localhost:9004"
echo "  IBC Relayer API:   http://localhost:3000"
echo "  Sultan Node RPC:   http://localhost:26657"
echo "  Prometheus:        http://localhost:9090"
echo "  Grafana:           http://localhost:3002 (admin/admin)"
echo ""
echo "📊 Management Commands:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  View logs:         docker-compose logs -f [service-name]"
echo "  Stop services:     docker-compose down"
echo "  Restart service:   docker-compose restart [service-name]"
echo "  Service status:    docker-compose ps"
echo "  Bridge stats:      curl http://localhost:26657/bridges | jq"
echo ""
echo "🔍 Health Checks:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Bitcoin:           curl http://localhost:9001/health"
echo "  Ethereum:          grpc_health_probe -addr=localhost:50051"
echo "  Solana:            grpc_health_probe -addr=localhost:50052"
echo "  TON:               curl http://localhost:9004/health"
echo "  Sultan Node:       curl http://localhost:26657/status"
echo ""
echo "📖 Documentation:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Bridge Status:     INTEROPERABILITY_STATUS.md"
echo "  Deployment Guide:  BRIDGE_DEPLOYMENT_GUIDE.md"
echo "  Monitoring:        deploy/prometheus/README.md"
echo ""
echo "✨ Next Steps:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  1. Configure IBC relayer mnemonics"
echo "  2. Setup monitoring alerts"
echo "  3. Test cross-chain transactions"
echo "  4. Configure load balancers for production"
echo ""
