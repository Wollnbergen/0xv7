// Sultan L1 Network Stats - CORRECTED VERSION
// This JavaScript properly maps the RPC response to the HTML stat boxes

(function() {
    const API_URL = 'https://rpc.sltn.io/status';
    const UPDATE_INTERVAL = 10000; // 10 seconds

    async function fetchNetworkStats() {
        try {
            const response = await fetch(API_URL);
            if (!response.ok) throw new Error('API request failed');
            const data = await response.json();
            
            console.log('✅ RPC Response:', data); // Debug log
            updateUI(data);
        } catch (error) {
            console.error('❌ Failed to fetch network stats:', error);
            setOfflineState();
        }
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

        // Active Validators - API returns "validator_count" not "validators.active"
        const validatorsEl = document.getElementById('active-validators');
        if (validatorsEl && data.validator_count !== undefined) {
            // Assuming total validators = 11 (from genesis config)
            validatorsEl.textContent = data.validator_count + ' / 11';
        }

        // TPS Capacity - calculate from shard_count (16 shards × 4000 TPS each = 64K)
        const tpsEl = document.getElementById('tps-capacity');
        if (tpsEl && data.shard_count) {
            const tps = data.shard_count * 4000; // 4K TPS per shard
            tpsEl.textContent = tps >= 1000 ? (tps / 1000) + 'K' : tps;
        }

        // Block Time - hardcoded to 2s (not in API response)
        const blockTimeEl = document.getElementById('block-time');
        if (blockTimeEl) {
            blockTimeEl.textContent = '2s';
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
