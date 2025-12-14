#!/bin/bash
# Health Check Script for All Bridge Services
# Can be used for monitoring and automated restarts

set -e

SERVICES=(
    "bitcoin-service:9001:/health"
    "ethereum-service:50051:grpc"
    "solana-service:50052:grpc"
    "ton-service:9004:/health"
    "ibc-relayer:3000:/status"
    "sultan-node:26657:/status"
)

echo "🏥 Bridge Services Health Check"
echo "================================"
echo ""

ALL_HEALTHY=true

for SERVICE_INFO in "${SERVICES[@]}"; do
    IFS=':' read -r SERVICE PORT ENDPOINT <<< "$SERVICE_INFO"
    
    printf "Checking %-25s " "$SERVICE..."
    
    if [ "$ENDPOINT" = "grpc" ]; then
        # gRPC health check
        if command -v grpc_health_probe >/dev/null 2>&1; then
            if grpc_health_probe -addr=localhost:$PORT >/dev/null 2>&1; then
                echo "✅ Healthy"
            else
                echo "❌ Unhealthy"
                ALL_HEALTHY=false
            fi
        else
            echo "⚠️  grpc_health_probe not installed"
        fi
    else
        # HTTP health check
        if curl -sf "http://localhost:${PORT}${ENDPOINT}" >/dev/null 2>&1; then
            echo "✅ Healthy"
        else
            echo "❌ Unhealthy"
            ALL_HEALTHY=false
        fi
    fi
done

echo ""
if [ "$ALL_HEALTHY" = true ]; then
    echo "✅ All services healthy"
    exit 0
else
    echo "❌ Some services are unhealthy"
    exit 1
fi
