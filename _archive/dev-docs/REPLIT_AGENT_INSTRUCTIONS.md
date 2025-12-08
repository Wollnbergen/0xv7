# Instructions for Replit Agent: Add Live Network Stats to Sultan Website

## Task
Replace the static "Network Stats" section in index.html with live data from the Sultan L1 production node API.

---

## API Endpoint
```
https://rpc.sltn.io/api/stats
```

Updates every 10 seconds with real-time blockchain data.

---

## Step 1: Find and Replace HTML Section

**FIND THIS in index.html** (the static stats section):
```html
<div class="network-stats">
    <h3>Network Stats</h3>
    <div class="stats-grid">
        <div class="stat-item">
            <span class="stat-label">Status</span>
            <span class="stat-value">Offline</span>
        </div>
        <div class="stat-item">
            <span class="stat-label">Block Height</span>
            <span class="stat-value">N/A</span>
        </div>
        <div class="stat-item">
            <span class="stat-label">Active Validators</span>
            <span class="stat-value">200K+</span>
        </div>
        <div class="stat-item">
            <span class="stat-label">TPS Capacity</span>
            <span class="stat-value">???</span>
        </div>
        <div class="stat-item">
            <span class="stat-label">Block Time</span>
            <span class="stat-value">2s</span>
        </div>
    </div>
</div>
```

**REPLACE WITH THIS:**
```html
<div class="network-stats">
    <h3>Network Stats 
        <span id="stats-status-indicator" class="status-dot" title="Live Updates"></span>
    </h3>
    <div class="stats-grid">
        <div class="stat-item">
            <span class="stat-label">Status</span>
            <span id="network-status" class="stat-value status-badge">Loading...</span>
        </div>
        <div class="stat-item">
            <span class="stat-label">Block Height</span>
            <span id="block-height" class="stat-value">Loading...</span>
        </div>
        <div class="stat-item">
            <span class="stat-label">Active Validators</span>
            <span id="active-validators" class="stat-value">Loading...</span>
        </div>
        <div class="stat-item">
            <span class="stat-label">TPS Capacity</span>
            <span id="tps-capacity" class="stat-value">Loading...</span>
        </div>
        <div class="stat-item">
            <span class="stat-label">Block Time</span>
            <span id="block-time" class="stat-value">2s</span>
        </div>
    </div>
    <div class="stats-footer">
        <small>Last updated: <span id="last-update">Never</span></small>
    </div>
</div>
```

---

## Step 2: Add CSS Styling

**ADD THIS CSS** to the `<style>` section in the `<head>` (or to your external CSS file):

```css
/* Live Stats Styling */
.status-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: #4ade80;
    margin-left: 8px;
    animation: pulse 2s infinite;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

.status-badge {
    padding: 4px 12px;
    border-radius: 12px;
    font-weight: bold;
}

.status-online {
    background-color: #4ade80;
    color: #064e3b;
}

.status-offline {
    background-color: #f87171;
    color: #7f1d1d;
}

.stat-value {
    font-family: 'Courier New', monospace;
    font-weight: bold;
    font-size: 1.1em;
}

.stats-footer {
    text-align: center;
    margin-top: 16px;
    opacity: 0.7;
    font-size: 0.85em;
}

/* Smooth number transitions */
.stat-value {
    transition: all 0.3s ease;
}
```

---

## Step 3: Add JavaScript

**ADD THIS JAVASCRIPT** right before the closing `</body>` tag:

```html
<script>
// Sultan L1 Network Stats - Live Updates
(function() {
    const API_URL = 'https://rpc.sltn.io/api/stats';
    const UPDATE_INTERVAL = 10000; // 10 seconds
    
    async function fetchNetworkStats() {
        try {
            const response = await fetch(API_URL);
            if (!response.ok) throw new Error('API request failed');
            const data = await response.json();
            updateUI(data);
            updateStatusIndicator(true);
        } catch (error) {
            console.error('Failed to fetch network stats:', error);
            setOfflineState();
            updateStatusIndicator(false);
        }
    }
    
    function updateUI(data) {
        // Status
        const statusEl = document.getElementById('network-status');
        if (statusEl) {
            statusEl.textContent = data.status === 'online' ? 'Online' : 'Offline';
            statusEl.className = 'stat-value status-badge ' + 
                (data.status === 'online' ? 'status-online' : 'status-offline');
        }
        
        // Block Height (formatted with commas)
        const blockHeightEl = document.getElementById('block-height');
        if (blockHeightEl) {
            blockHeightEl.textContent = data.block_height.toLocaleString();
        }
        
        // Active Validators
        const validatorsEl = document.getElementById('active-validators');
        if (validatorsEl) {
            validatorsEl.textContent = data.validators.active + ' / ' + data.validators.total;
        }
        
        // TPS Capacity (formatted)
        const tpsEl = document.getElementById('tps-capacity');
        if (tpsEl) {
            const tps = data.tps.capacity;
            tpsEl.textContent = tps >= 1000 ? (tps / 1000) + 'K' : tps;
        }
        
        // Block Time
        const blockTimeEl = document.getElementById('block-time');
        if (blockTimeEl) {
            blockTimeEl.textContent = data.block_time + 's';
        }
        
        // Last Update Time
        const lastUpdateEl = document.getElementById('last-update');
        if (lastUpdateEl) {
            const now = new Date();
            lastUpdateEl.textContent = now.toLocaleTimeString();
        }
    }
    
    function setOfflineState() {
        const statusEl = document.getElementById('network-status');
        if (statusEl) {
            statusEl.textContent = 'Offline';
            statusEl.className = 'stat-value status-badge status-offline';
        }
        
        // Set all other values to "N/A"
        ['block-height', 'active-validators', 'tps-capacity'].forEach(id => {
            const el = document.getElementById(id);
            if (el) el.textContent = 'N/A';
        });
    }
    
    function updateStatusIndicator(isOnline) {
        const indicator = document.getElementById('stats-status-indicator');
        if (indicator) {
            indicator.style.backgroundColor = isOnline ? '#4ade80' : '#f87171';
        }
    }
    
    // Initialize on page load
    fetchNetworkStats();
    
    // Update every 10 seconds
    setInterval(fetchNetworkStats, UPDATE_INTERVAL);
})();
</script>
```

---

## Summary for Replit Agent

**Instructions:**
1. Find the existing `<div class="network-stats">` section in index.html
2. Replace it with the new HTML from Step 1 (includes 5 optimized stat items with IDs)
3. Add the CSS from Step 2 to your existing `<style>` section
4. Add the JavaScript from Step 3 right before `</body>`
5. Test by refreshing the page - stats should update every 10 seconds

**Expected Result:**
- ✅ **Status** - "Online" in green badge
- ✅ **Block Height** - Live count (e.g., "15,883,824")
- ✅ **Active Validators** - "11 / 11"
- ✅ **TPS Capacity** - "64K"
- ✅ **Block Time** - "2s"
- Green pulsing dot next to "Network Stats" indicates live updates
- "Last updated" timestamp at bottom

**Note:** Advanced stats (shards, supply, inflation) can be added to a separate "Tokenomics" or "Technical Details" section later if needed.

**API Response Example:**
```json
{
  "status": "online",
  "block_height": 15883824,
  "validators": {"active": 11, "total": 11},
  "tps": {"capacity": 64000, "max_capacity": 64000000},
  "sharding": {"active_shards": 8, "max_shards": 8000},
  "supply": {"genesis": 500000000, "current": 540000000, "inflation_rate": 8.0},
  "block_time": 2
}
```

---

## Testing the API (Optional)

To verify the API is working, run this in browser console:
```javascript
fetch('https://rpc.sltn.io/api/stats')
  .then(res => res.json())
  .then(data => console.table(data));
```

---

**Note:** The API updates every 10 seconds automatically, so the stats will refresh without page reload.
