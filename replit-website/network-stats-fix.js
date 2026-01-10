// Sultan L1 Network Stats - Production Validators (January 2026)
// This JavaScript properly maps the RPC response to the HTML stat boxes

(function() {
    // Production RPC endpoint (nginx proxies to port 26657)
    const RPC_ENDPOINTS = [
        'https://rpc.sltn.io',          // Primary endpoint (NYC)
        'http://206.189.224.142:26657', // NYC direct
        'http://134.122.96.36:26657',   // London
        'http://143.198.205.21:26657',  // Singapore  
        'http://142.93.238.33:26657',   // Amsterdam
        'http://46.101.122.13:26657',   // Frankfurt
        'http://24.144.94.23:26657'     // San Francisco
    ];
    const UPDATE_INTERVAL = 5000; // 5 seconds
    let currentEndpointIndex = 0;

    async function fetchNetworkStats() {
        // Try each endpoint until one succeeds
        for (let i = 0; i < RPC_ENDPOINTS.length; i++) {
            const endpoint = RPC_ENDPOINTS[(currentEndpointIndex + i) % RPC_ENDPOINTS.length];
            try {
                const response = await fetch(`${endpoint}/status`, {
                    method: 'GET',
                    mode: 'cors',
                    cache: 'no-cache',
                    signal: AbortSignal.timeout(5000)
                });
                if (!response.ok) continue;
                const data = await response.json();
                
                console.log('✅ RPC Response from', endpoint, ':', data);
                currentEndpointIndex = (currentEndpointIndex + i) % RPC_ENDPOINTS.length;
                updateUI(data);
                return;
            } catch (error) {
                console.warn('⚠️ Endpoint failed:', endpoint, error.message);
                continue;
            }
        }
        console.error('❌ All endpoints failed');
        setOfflineState();
    }

    function updateUI(data) {
        // Status - derive from successful API response
        const statusEl = document.getElementById('network-status');
        if (statusEl) {
            statusEl.textContent = 'Online';
            statusEl.className = 'number status-online';
        }

        // Block Height - API returns "height" not "block_height"
        const blockHeightEl = document.getElementById('block-height');
        if (blockHeightEl && data.height) {
            const height = data.height.toString();
            // Split into groups of 3 from right to left
            const groups = [];
            for (let i = height.length; i > 0; i -= 3) {
                groups.unshift(height.slice(Math.max(0, i - 3), i));
            }
            blockHeightEl.innerHTML = groups.join('<br>');
        }

        // Active Validators - API returns "validator_count" 
        const validatorsEl = document.getElementById('active-validators');
        if (validatorsEl && data.validator_count !== undefined) {
            // 6 total validators in production
            validatorsEl.textContent = data.validator_count + ' / 6';
        }

        // TPS Capacity - calculate from shard_count (16 shards × 4000 TPS each = 64K)
        const tpsEl = document.getElementById('tps-capacity');
        if (tpsEl && data.shard_count) {
            const tps = data.shard_count * 4000; // 4K TPS per shard
            tpsEl.textContent = tps >= 1000 ? (tps / 1000) + 'K' : tps;
        }

        // Active Shards - API returns "shard_count"
        const shardsEl = document.getElementById('active-shards');
        if (shardsEl && data.shard_count) {
            shardsEl.textContent = data.shard_count;
        }

        // Last Update Time
        const lastUpdateEl = document.getElementById('last-update');
        if (lastUpdateEl) {
            const now = new Date();
            lastUpdateEl.textContent = now.toLocaleTimeString();
        }

        console.log('✅ UI updated successfully');
    }

    function setOfflineState() {
        const statusEl = document.getElementById('network-status');
        if (statusEl) {
            statusEl.textContent = 'Offline';
            statusEl.className = 'number status-offline';
        }

        // Set all other values to "N/A"
        ['block-height', 'active-validators', 'tps-capacity'].forEach(id => {
            const el = document.getElementById(id);
            if (el) el.textContent = 'N/A';
        });
        
        console.log('⚠️ Network marked as offline');
    }

    // Initialize on page load
    console.log('🚀 Initializing network stats...');
    fetchNetworkStats();

    // Update every 10 seconds
    setInterval(fetchNetworkStats, UPDATE_INTERVAL);
})();
