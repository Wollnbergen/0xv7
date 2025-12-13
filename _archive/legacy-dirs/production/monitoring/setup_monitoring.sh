#!/bin/bash

# Start Prometheus
docker run -d \
    --name sultan-prometheus \
    -p 9090:9090 \
    -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml \
    prom/prometheus:latest

# Start Grafana
docker run -d \
    --name sultan-grafana \
    -p 3001:3000 \
    -e "GF_SECURITY_ADMIN_PASSWORD=sultan" \
    -e "GF_INSTALL_PLUGINS=redis-datasource" \
    grafana/grafana:latest

echo "✅ Monitoring stack deployed"
echo "   Prometheus: http://localhost:9090"
echo "   Grafana: http://localhost:3001 (admin/sultan)"
