# Sultan L1 Website Dashboard Integration

## Live Network Stats API

**Endpoint**: `https://rpc.sltn.io/api/stats`  
**Alternative**: `http://5.161.225.96/api/stats`  
**Update Frequency**: Every 10 seconds  
**CORS**: Enabled for all origins

---

## API Response Format

```json
{
  "status": "online",
  "network": "mainnet",
  "chain_id": "sultan-1",
  "block_height": 15883824,
  "block_time": 2,
  "validators": {
    "active": 11,
    "total": 11
  },
  "tps": {
    "current": 64000,
    "capacity": 64000,
    "max_capacity": 64000000
  },
  "sharding": {
    "active_shards": 8,
    "max_shards": 8000,
    "tps_per_shard": 8000
  },
  "supply": {
    "genesis": 500000000,
    "current": 540000000,
    "inflation_rate": 8.0
  },
  "staking": {
    "min_stake": 10000,
    "total_staked": 110000
  },
  "node": {
    "running": true,
    "uptime_seconds": 270,
    "memory_mb": 10.75
  },
  "timestamp": 1733492448
}
```

---

## JavaScript Integration (Vanilla JS)

### Option 1: Simple Fetch API

```javascript
async function updateNetworkStats() {
    try {
        const response = await fetch('https://rpc.sltn.io/api/stats');
        const data = await response.json();
        
        // Update your HTML elements
        document.getElementById('network-status').textContent = data.status;
        document.getElementById('block-height').textContent = data.block_height.toLocaleString();
        document.getElementById('active-validators').textContent = data.validators.active;
        document.getElementById('tps-capacity').textContent = data.tps.capacity.toLocaleString();
        document.getElementById('block-time').textContent = data.block_time + 's';
        document.getElementById('active-shards').textContent = data.sharding.active_shards;
        document.getElementById('total-supply').textContent = (data.supply.current / 1e6).toFixed(1) + 'M SLTN';
        
    } catch (error) {
        console.error('Failed to fetch network stats:', error);
        document.getElementById('network-status').textContent = 'Offline';
    }
}

// Update every 10 seconds
updateNetworkStats();
setInterval(updateNetworkStats, 10000);
```

### Option 2: With Error Handling & Animations

```javascript
class SultanNetworkMonitor {
    constructor(apiUrl = 'https://rpc.sltn.io/api/stats') {
        this.apiUrl = apiUrl;
        this.updateInterval = 10000; // 10 seconds
        this.intervalId = null;
    }
    
    async fetchStats() {
        try {
            const response = await fetch(this.apiUrl);
            if (!response.ok) throw new Error('API request failed');
            return await response.json();
        } catch (error) {
            console.error('Stats fetch error:', error);
            return null;
        }
    }
    
    updateUI(data) {
        if (!data) {
            this.setOfflineState();
            return;
        }
        
        // Status indicator
        const statusEl = document.getElementById('network-status');
        statusEl.textContent = data.status === 'online' ? 'Online' : 'Offline';
        statusEl.className = data.status === 'online' ? 'status-online' : 'status-offline';
        
        // Block height with animation
        this.animateValue('block-height', data.block_height);
        
        // Validators
        document.getElementById('active-validators').textContent = 
            `${data.validators.active}/${data.validators.total}`;
        
        // TPS with formatting
        const tps = data.tps.capacity;
        document.getElementById('tps-capacity').textContent = 
            tps >= 1000000 ? (tps / 1000000).toFixed(1) + 'M' : 
            tps >= 1000 ? (tps / 1000).toFixed(0) + 'K' : tps;
        
        // Block time
        document.getElementById('block-time').textContent = data.block_time + 's';
        
        // Shards
        document.getElementById('active-shards').textContent = 
            `${data.sharding.active_shards} / ${data.sharding.max_shards}`;
        
        // Supply
        document.getElementById('total-supply').textContent = 
            (data.supply.current / 1e6).toFixed(1) + 'M SLTN';
        
        // Inflation rate
        document.getElementById('inflation-rate').textContent = 
            data.supply.inflation_rate + '%';
        
        // Last update time
        const lastUpdate = new Date(data.timestamp * 1000);
        document.getElementById('last-update').textContent = 
            lastUpdate.toLocaleTimeString();
    }
    
    animateValue(elementId, targetValue) {
        const element = document.getElementById(elementId);
        const currentValue = parseInt(element.textContent.replace(/,/g, '')) || 0;
        
        if (currentValue === targetValue) return;
        
        const duration = 500; // milliseconds
        const steps = 20;
        const stepValue = (targetValue - currentValue) / steps;
        const stepDuration = duration / steps;
        
        let step = 0;
        const interval = setInterval(() => {
            step++;
            const newValue = Math.round(currentValue + (stepValue * step));
            element.textContent = newValue.toLocaleString();
            
            if (step >= steps) {
                clearInterval(interval);
                element.textContent = targetValue.toLocaleString();
            }
        }, stepDuration);
    }
    
    setOfflineState() {
        document.getElementById('network-status').textContent = 'Offline';
        document.getElementById('network-status').className = 'status-offline';
        document.getElementById('block-height').textContent = 'N/A';
    }
    
    start() {
        this.fetchStats().then(data => this.updateUI(data));
        this.intervalId = setInterval(async () => {
            const data = await this.fetchStats();
            this.updateUI(data);
        }, this.updateInterval);
    }
    
    stop() {
        if (this.intervalId) {
            clearInterval(this.intervalId);
            this.intervalId = null;
        }
    }
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', () => {
    const monitor = new SultanNetworkMonitor();
    monitor.start();
});
```

---

## HTML Example

```html
<div class="network-stats">
    <div class="stat-card">
        <div class="stat-label">Status</div>
        <div id="network-status" class="stat-value">Loading...</div>
    </div>
    
    <div class="stat-card">
        <div class="stat-label">Block Height</div>
        <div id="block-height" class="stat-value">Loading...</div>
    </div>
    
    <div class="stat-card">
        <div class="stat-label">Active Validators</div>
        <div id="active-validators" class="stat-value">Loading...</div>
    </div>
    
    <div class="stat-card">
        <div class="stat-label">TPS Capacity</div>
        <div id="tps-capacity" class="stat-value">Loading...</div>
    </div>
    
    <div class="stat-card">
        <div class="stat-label">Block Time</div>
        <div id="block-time" class="stat-value">Loading...</div>
    </div>
    
    <div class="stat-card">
        <div class="stat-label">Active Shards</div>
        <div id="active-shards" class="stat-value">Loading...</div>
    </div>
    
    <div class="stat-card">
        <div class="stat-label">Total Supply</div>
        <div id="total-supply" class="stat-value">Loading...</div>
    </div>
    
    <div class="stat-card">
        <div class="stat-label">Inflation Rate</div>
        <div id="inflation-rate" class="stat-value">Loading...</div>
    </div>
    
    <div class="last-update">
        Last updated: <span id="last-update">Never</span>
    </div>
</div>
```

---

## CSS Styling Example

```css
.network-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    padding: 20px;
}

.stat-card {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    padding: 20px;
    border-radius: 10px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    color: white;
    transition: transform 0.2s;
}

.stat-card:hover {
    transform: translateY(-5px);
}

.stat-label {
    font-size: 14px;
    opacity: 0.9;
    margin-bottom: 8px;
    text-transform: uppercase;
    letter-spacing: 1px;
}

.stat-value {
    font-size: 28px;
    font-weight: bold;
    font-family: 'Courier New', monospace;
}

.status-online {
    color: #4ade80;
}

.status-offline {
    color: #f87171;
}

.last-update {
    grid-column: 1 / -1;
    text-align: center;
    color: #666;
    font-size: 12px;
    margin-top: 10px;
}
```

---

## React Integration

```jsx
import React, { useState, useEffect } from 'react';

function NetworkStats() {
    const [stats, setStats] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    useEffect(() => {
        const fetchStats = async () => {
            try {
                const response = await fetch('https://rpc.sltn.io/api/stats');
                const data = await response.json();
                setStats(data);
                setLoading(false);
                setError(null);
            } catch (err) {
                setError('Failed to load network stats');
                setLoading(false);
            }
        };

        fetchStats();
        const interval = setInterval(fetchStats, 10000);
        
        return () => clearInterval(interval);
    }, []);

    if (loading) return <div>Loading...</div>;
    if (error) return <div className="error">{error}</div>;

    return (
        <div className="network-stats">
            <StatCard label="Status" value={stats.status} />
            <StatCard label="Block Height" value={stats.block_height.toLocaleString()} />
            <StatCard label="Active Validators" value={stats.validators.active} />
            <StatCard label="TPS Capacity" value={`${(stats.tps.capacity / 1000).toFixed(0)}K`} />
            <StatCard label="Block Time" value={`${stats.block_time}s`} />
            <StatCard label="Active Shards" value={`${stats.sharding.active_shards} / ${stats.sharding.max_shards}`} />
            <StatCard label="Total Supply" value={`${(stats.supply.current / 1e6).toFixed(1)}M SLTN`} />
            <StatCard label="Inflation Rate" value={`${stats.supply.inflation_rate}%`} />
        </div>
    );
}

function StatCard({ label, value }) {
    return (
        <div className="stat-card">
            <div className="stat-label">{label}</div>
            <div className="stat-value">{value}</div>
        </div>
    );
}

export default NetworkStats;
```

---

## Vue.js Integration

```vue
<template>
    <div class="network-stats">
        <div v-if="loading">Loading...</div>
        <div v-else-if="error" class="error">{{ error }}</div>
        <div v-else class="stats-grid">
            <div class="stat-card" v-for="stat in displayStats" :key="stat.label">
                <div class="stat-label">{{ stat.label }}</div>
                <div class="stat-value">{{ stat.value }}</div>
            </div>
        </div>
    </div>
</template>

<script>
export default {
    name: 'NetworkStats',
    data() {
        return {
            stats: null,
            loading: true,
            error: null,
            intervalId: null
        };
    },
    computed: {
        displayStats() {
            if (!this.stats) return [];
            return [
                { label: 'Status', value: this.stats.status },
                { label: 'Block Height', value: this.stats.block_height.toLocaleString() },
                { label: 'Active Validators', value: this.stats.validators.active },
                { label: 'TPS Capacity', value: `${(this.stats.tps.capacity / 1000).toFixed(0)}K` },
                { label: 'Block Time', value: `${this.stats.block_time}s` },
                { label: 'Active Shards', value: `${this.stats.sharding.active_shards} / ${this.stats.sharding.max_shards}` },
                { label: 'Total Supply', value: `${(this.stats.supply.current / 1e6).toFixed(1)}M SLTN` },
                { label: 'Inflation Rate', value: `${this.stats.supply.inflation_rate}%` }
            ];
        }
    },
    methods: {
        async fetchStats() {
            try {
                const response = await fetch('https://rpc.sltn.io/api/stats');
                this.stats = await response.json();
                this.loading = false;
                this.error = null;
            } catch (err) {
                this.error = 'Failed to load network stats';
                this.loading = false;
            }
        }
    },
    mounted() {
        this.fetchStats();
        this.intervalId = setInterval(this.fetchStats, 10000);
    },
    beforeUnmount() {
        if (this.intervalId) {
            clearInterval(this.intervalId);
        }
    }
};
</script>
```

---

## Testing the API

### cURL Test
```bash
curl https://rpc.sltn.io/api/stats | jq '.'
```

### Browser Console Test
```javascript
fetch('https://rpc.sltn.io/api/stats')
    .then(res => res.json())
    .then(data => console.table(data));
```

---

## Current Live Values (as of Dec 6, 2025)

- **Status**: Online ✅
- **Block Height**: ~15.9 million (2-second blocks since Dec 3)
- **Active Validators**: 11
- **TPS Capacity**: 64,000 (base), expandable to 64M
- **Block Time**: 2 seconds
- **Active Shards**: 8 (max: 8,000)
- **Total Supply**: 540M SLTN (500M genesis + 40M inflation)
- **Inflation Rate**: 8%

---

## Support

For API issues or feature requests, contact: **dev@sltn.io**

**API Documentation**: This document  
**API Status**: https://rpc.sltn.io/api/stats  
**Grafana Dashboard**: http://5.161.225.96:3000
