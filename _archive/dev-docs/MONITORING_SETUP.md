# Sultan L1 - Basic Monitoring Setup

## 🎯 Quick Monitoring (Immediate)

### Simple Block Height Monitor
```bash
# Create monitoring script on production server
cat > /usr/local/bin/sultan-monitor.sh << 'EOF'
#!/bin/bash

while true; do
    HEIGHT=$(curl -s http://127.0.0.1:8080/status | jq -r '.height')
    VALIDATORS=$(curl -s http://127.0.0.1:8080/status | jq -r '.validator_count')
    SHARDS=$(curl -s http://127.0.0.1:8080/status | jq -r '.shard_count')
    
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$TIMESTAMP] Height: $HEIGHT | Validators: $VALIDATORS | Shards: $SHARDS"
    
    # Alert if height hasn't increased in 10 seconds
    if [ -f /tmp/last_height ]; then
        LAST_HEIGHT=$(cat /tmp/last_height)
        if [ "$HEIGHT" == "$LAST_HEIGHT" ]; then
            echo "⚠️  WARNING: Block height stuck at $HEIGHT!"
        fi
    fi
    echo $HEIGHT > /tmp/last_height
    
    sleep 5
done
EOF

chmod +x /usr/local/bin/sultan-monitor.sh

# Run in background
nohup /usr/local/bin/sultan-monitor.sh >> /var/log/sultan-monitor.log 2>&1 &
```

### Quick Dashboard (HTML)
```bash
# Create simple web dashboard
cat > /var/www/html/monitor.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Sultan L1 Monitor</title>
    <meta http-equiv="refresh" content="2">
    <style>
        body { font-family: monospace; background: #1a1a1a; color: #0f0; padding: 20px; }
        .metric { font-size: 24px; margin: 10px 0; }
        .label { color: #888; }
        .value { color: #0f0; font-weight: bold; }
        .error { color: #f00; }
    </style>
</head>
<body>
    <h1>🚀 Sultan L1 Live Monitor</h1>
    <div id="stats"></div>
    
    <script>
        async function updateStats() {
            try {
                const response = await fetch('https://rpc.sltn.io/status');
                const data = await response.json();
                
                const html = `
                    <div class="metric">
                        <span class="label">Block Height:</span> 
                        <span class="value">${data.height}</span>
                    </div>
                    <div class="metric">
                        <span class="label">Validators:</span> 
                        <span class="value">${data.validator_count}</span>
                    </div>
                    <div class="metric">
                        <span class="label">Shards:</span> 
                        <span class="value">${data.shard_count} / ${data.max_shards || 8000}</span>
                    </div>
                    <div class="metric">
                        <span class="label">TPS:</span> 
                        <span class="value">${data.tps || 0}</span>
                    </div>
                    <div class="metric">
                        <span class="label">Supply:</span> 
                        <span class="value">${data.total_supply?.toLocaleString() || 0} SLTN</span>
                    </div>
                `;
                
                document.getElementById('stats').innerHTML = html;
            } catch (error) {
                document.getElementById('stats').innerHTML = '<div class="error">Error fetching stats</div>';
            }
        }
        
        updateStats();
        setInterval(updateStats, 2000);
    </script>
</body>
</html>
EOF

# Access at: http://5.161.225.96/monitor.html
```

---

## 📊 Prometheus Setup (Production-Grade)

### Install Prometheus
```bash
# On production server or separate monitoring server

# Download Prometheus
cd /tmp
wget https://github.com/prometheus/prometheus/releases/download/v2.48.0/prometheus-2.48.0.linux-amd64.tar.gz
tar xvf prometheus-2.48.0.linux-amd64.tar.gz
sudo mv prometheus-2.48.0.linux-amd64 /opt/prometheus

# Create configuration
cat > /opt/prometheus/prometheus.yml << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'sultan-node'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
    
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['localhost:9100']
EOF

# Create systemd service
cat > /etc/systemd/system/prometheus.service << 'EOF'
[Unit]
Description=Prometheus
After=network.target

[Service]
Type=simple
User=root
ExecStart=/opt/prometheus/prometheus \
  --config.file=/opt/prometheus/prometheus.yml \
  --storage.tsdb.path=/opt/prometheus/data
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable prometheus
systemctl start prometheus

# Verify
curl http://localhost:9090/-/healthy
```

### Install Node Exporter (System Metrics)
```bash
# Download Node Exporter
cd /tmp
wget https://github.com/prometheus/node_exporter/releases/download/v1.7.0/node_exporter-1.7.0.linux-amd64.tar.gz
tar xvf node_exporter-1.7.0.linux-amd64.tar.gz
sudo mv node_exporter-1.7.0.linux-amd64/node_exporter /usr/local/bin/

# Create systemd service
cat > /etc/systemd/system/node-exporter.service << 'EOF'
[Unit]
Description=Node Exporter
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/node_exporter
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable node-exporter
systemctl start node-exporter

# Verify
curl http://localhost:9100/metrics | head
```

---

## 📈 Grafana Setup (Dashboards)

### Install Grafana
```bash
# Add Grafana repository
sudo apt-get install -y software-properties-common
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -
echo "deb https://packages.grafana.com/oss/deb stable main" | sudo tee -a /etc/apt/sources.list.d/grafana.list

# Install
sudo apt-get update
sudo apt-get install -y grafana

# Start service
sudo systemctl daemon-reload
sudo systemctl enable grafana-server
sudo systemctl start grafana-server

# Access Grafana at http://5.161.225.96:3000
# Default login: admin / admin
```

### Configure Prometheus Data Source
```bash
# Add Prometheus as data source via API
curl -X POST http://admin:admin@localhost:3000/api/datasources \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Prometheus",
    "type": "prometheus",
    "url": "http://localhost:9090",
    "access": "proxy",
    "isDefault": true
  }'
```

### Create Sultan Dashboard
Save this as `/tmp/sultan-dashboard.json`:

```json
{
  "dashboard": {
    "title": "Sultan L1 Blockchain",
    "panels": [
      {
        "title": "Block Height",
        "targets": [{
          "expr": "sultan_block_height",
          "refId": "A"
        }],
        "type": "graph"
      },
      {
        "title": "Validators Online",
        "targets": [{
          "expr": "sultan_validator_count",
          "refId": "A"
        }],
        "type": "stat"
      },
      {
        "title": "Active Shards",
        "targets": [{
          "expr": "sultan_shard_count",
          "refId": "A"
        }],
        "type": "stat"
      },
      {
        "title": "TPS (Transactions Per Second)",
        "targets": [{
          "expr": "rate(sultan_transactions_total[1m])",
          "refId": "A"
        }],
        "type": "graph"
      }
    ]
  }
}
```

Import dashboard:
```bash
curl -X POST http://admin:admin@localhost:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @/tmp/sultan-dashboard.json
```

---

## 🚨 AlertManager Setup (Alerts)

### Install AlertManager
```bash
cd /tmp
wget https://github.com/prometheus/alertmanager/releases/download/v0.26.0/alertmanager-0.26.0.linux-amd64.tar.gz
tar xvf alertmanager-0.26.0.linux-amd64.tar.gz
sudo mv alertmanager-0.26.0.linux-amd64 /opt/alertmanager

# Create configuration
cat > /opt/alertmanager/alertmanager.yml << 'EOF'
global:
  resolve_timeout: 5m

route:
  receiver: 'email-notifications'
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h

receivers:
  - name: 'email-notifications'
    email_configs:
      - to: 'your-email@example.com'
        from: 'sultan-alerts@sltn.io'
        smarthost: 'smtp.gmail.com:587'
        auth_username: 'your-email@gmail.com'
        auth_password: 'your-app-password'
EOF

# Create systemd service
cat > /etc/systemd/system/alertmanager.service << 'EOF'
[Unit]
Description=AlertManager
After=network.target

[Service]
Type=simple
ExecStart=/opt/alertmanager/alertmanager \
  --config.file=/opt/alertmanager/alertmanager.yml
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable alertmanager
systemctl start alertmanager
```

### Create Alert Rules
```bash
# Create alert rules file
cat > /opt/prometheus/alert_rules.yml << 'EOF'
groups:
  - name: sultan_alerts
    rules:
      - alert: BlockProductionStopped
        expr: increase(sultan_block_height[2m]) == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Block production has stopped"
          description: "No new blocks in last 2 minutes"
      
      - alert: ValidatorOffline
        expr: sultan_validator_count < 11
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Validator offline"
          description: "Only {{ $value }} validators online (expected 11)"
      
      - alert: HighMemoryUsage
        expr: node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes < 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Less than 10% memory available"
      
      - alert: DiskSpaceLow
        expr: node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes < 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low disk space"
          description: "Less than 10% disk space remaining"
EOF

# Update Prometheus config to include alert rules
cat >> /opt/prometheus/prometheus.yml << 'EOF'

rule_files:
  - 'alert_rules.yml'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['localhost:9093']
EOF

# Reload Prometheus
systemctl reload prometheus
```

---

## 📝 Logging Setup

### Centralized Logging with Loki
```bash
# Install Loki
cd /tmp
wget https://github.com/grafana/loki/releases/download/v2.9.3/loki-linux-amd64.zip
unzip loki-linux-amd64.zip
sudo mv loki-linux-amd64 /usr/local/bin/loki

# Create config
cat > /etc/loki-config.yml << 'EOF'
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /tmp/loki/boltdb-shipper-active
    cache_location: /tmp/loki/boltdb-shipper-cache
    shared_store: filesystem
  filesystem:
    directory: /tmp/loki/chunks
EOF

# Create systemd service
cat > /etc/systemd/system/loki.service << 'EOF'
[Unit]
Description=Loki
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/loki -config.file=/etc/loki-config.yml
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable loki
systemctl start loki
```

### Install Promtail (Log Shipper)
```bash
cd /tmp
wget https://github.com/grafana/loki/releases/download/v2.9.3/promtail-linux-amd64.zip
unzip promtail-linux-amd64.zip
sudo mv promtail-linux-amd64 /usr/local/bin/promtail

# Create config
cat > /etc/promtail-config.yml << 'EOF'
server:
  http_listen_port: 9080

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://localhost:3100/loki/api/v1/push

scrape_configs:
  - job_name: sultan-node
    static_configs:
      - targets:
          - localhost
        labels:
          job: sultan-node
          __path__: /var/log/syslog
    pipeline_stages:
      - match:
          selector: '{job="sultan-node"}'
          stages:
            - regex:
                expression: '.*sultan-node.*'
EOF

# Create systemd service
cat > /etc/systemd/system/promtail.service << 'EOF'
[Unit]
Description=Promtail
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/promtail -config.file=/etc/promtail-config.yml
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable promtail
systemctl start promtail
```

---

## ✅ Monitoring Checklist

After setup, verify:

### Prometheus
- [ ] http://5.161.225.96:9090 accessible
- [ ] Targets showing "UP" status
- [ ] Metrics being collected

### Grafana
- [ ] http://5.161.225.96:3000 accessible
- [ ] Dashboard showing live data
- [ ] Panels updating every 15s

### AlertManager
- [ ] http://5.161.225.96:9093 accessible
- [ ] Email alerts configured
- [ ] Test alert fires correctly

### Node Exporter
- [ ] http://5.161.225.96:9100/metrics showing data
- [ ] CPU, memory, disk metrics available

---

## 🎯 Quick Start Commands

```bash
# Check all monitoring services
systemctl status prometheus grafana-server alertmanager node-exporter loki promtail

# View Sultan metrics
curl http://localhost:9090/api/v1/query?query=sultan_block_height

# Test alert
curl -X POST http://localhost:9093/api/v1/alerts \
  -d '[{"labels":{"alertname":"test"}}]'

# View Grafana dashboards
xdg-open http://localhost:3000

# View logs in Grafana
# Go to Explore → Select Loki → Query: {job="sultan-node"}
```

---

## 📊 Metrics to Track

### Blockchain Metrics
- `sultan_block_height` - Current block number
- `sultan_block_time` - Time to produce last block
- `sultan_validator_count` - Active validators
- `sultan_shard_count` - Active shards
- `sultan_transactions_total` - Total transactions processed
- `sultan_tps` - Transactions per second

### System Metrics
- `node_cpu_seconds_total` - CPU usage
- `node_memory_MemAvailable_bytes` - Available memory
- `node_filesystem_avail_bytes` - Available disk space
- `node_network_receive_bytes_total` - Network traffic

### Consensus Metrics
- `sultan_consensus_rounds` - Consensus rounds completed
- `sultan_missed_blocks` - Blocks missed by validators
- `sultan_consensus_failures` - Failed consensus attempts

---

This monitoring setup will give you real-time visibility into your blockchain's health! 🚀
