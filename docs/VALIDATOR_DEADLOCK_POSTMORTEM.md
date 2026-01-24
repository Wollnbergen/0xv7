# Sultan Chain Validator Deadlock - Postmortem & Fix Guide

**Date:** January 14, 2026  
**Affected Systems:** Sultan Chain Validators (NYC, SFO, FRA, AMS, SGP, LON)  
**Status:** ✅ RESOLVED

---

## Executive Summary

The Sultan Chain validator network experienced a block production stall where blocks stopped incrementing despite validators running. This document details all issues encountered and their resolutions for future reference.

---

## Issue 1: Block Production Loop Deadlock

### Symptoms
- Block height stuck (not incrementing)
- Logs showed `ThreadId(01)` going silent after a few iterations
- Other threads (ThreadId(02), etc.) continued running
- No error messages in logs

### Root Cause
The `produce_block()` function was using `.await` on `RwLock` read/write operations which could deadlock when multiple async tasks competed for the same locks.

### Fix Applied
Changed blocking `.read().await` and `.write().await` calls to non-blocking `try_read()` and `try_write()` with retry loops:

**File:** `sultan-core/src/main.rs`

```rust
// BEFORE (could deadlock):
let guard = self.consensus.read().await;

// AFTER (non-blocking with retry):
let guard = {
    let mut retries = 0;
    loop {
        match self.consensus.try_read() {
            Ok(g) => break g,
            Err(_) => {
                retries += 1;
                if retries > 50 {
                    anyhow::bail!("Failed to acquire lock after 50 retries");
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
};
```

### Debug Logging Added
Added trace points throughout `produce_block()` to identify where stalls occur:

```rust
info!("⏰ Block production: tick received, starting produce_block()");
info!("🔨 produce_block: entering function");
info!("🔨 produce_block: got height {}", current_height);
info!("🔨 produce_block: proposer selected: {}", proposer);
info!("🔨 produce_block: passed sync check");
info!("🔨 produce_block: passed peer gate");
info!("🎯 We are proposer for height {}", next_height);
```

---

## Issue 2: Peer Gate Blocking Bootstrap Validator

### Symptoms
- Logs showed "IT IS OUR TURN for height N" then immediately "waiting for next tick"
- "passed peer gate" log never appeared
- Bootstrap validator couldn't produce first blocks

### Root Cause
The peer gate logic required peers before producing blocks:

```rust
// PROBLEMATIC CODE:
if peer_count < 1 && next_height <= 5 {
    return Ok(()); // Early return - no blocks produced!
}
```

This created a chicken-and-egg problem: the bootstrap validator needed peers to produce blocks, but peers couldn't sync without blocks.

### Fix Applied
Disabled the peer gate to only log peer status without blocking:

**File:** `sultan-core/src/main.rs` (lines ~1050-1077)

```rust
// === PEER-GATE: Disabled for now - just log peer count ===
// The peer gate was causing the chain to stall because the bootstrap
// validator couldn't produce without peers, but peers can't connect
// without blocks to sync. Chicken-and-egg problem.
if self.p2p_enabled {
    if let Some(p2p) = &self.p2p_network {
        match tokio::time::timeout(Duration::from_millis(100), p2p.read()).await {
            Ok(p2p_guard) => {
                if let Ok(peer_count) = tokio::time::timeout(Duration::from_millis(50), p2p_guard.peer_count()).await {
                    if peer_count < 1 {
                        debug!("📡 No peers connected, but producing block {} anyway", next_height);
                    } else {
                        debug!("📡 {} peer(s) connected, producing block {}", peer_count, next_height);
                    }
                }
                drop(p2p_guard);
            }
            Err(_) => {
                warn!("⚠️ P2P lock busy, skipping peer check");
            }
        }
    }
}
```

---

## Issue 3: P2P Validators Auto-Adding to Consensus (CRITICAL)

### Symptoms
```
NYC selected proposer: 289d7e71... (height 17)
SFO selected proposer: a1f2b3c4... (height 17, different!)
```

- Multiple validators appeared in consensus (validator_count: 5)
- Same height, different proposer selections across nodes
- Chain stalled at height 16-17 because no consensus on proposer

### Root Cause: Divergent Validator Sets
When validators connected via P2P, they sent `ValidatorAnnounce` messages and were **automatically added to the consensus validator set**. This caused:

1. **Non-deterministic ordering**: Different nodes received announcements in different orders
2. **Divergent validator sets**: Each node had a different view of active validators
3. **Proposer disagreement**: Height-based proposer selection produced different results
4. **Chain deadlock**: No blocks accepted because nodes couldn't agree on proposer

### Fix Applied: Enterprise-Grade On-Chain Registration (v0.1.6+)

**Architectural Change:** Separated P2P discovery from consensus membership.

**P2P `ValidatorAnnounce` - Discovery Only:**
```rust
P2PMessage::ValidatorAnnounce { address, pubkey, .. } => {
    // ✅ Register pubkey for signature verification (discovery)
    if let Ok(pubkey_arr) = pubkey.clone().try_into() {
        network.register_validator_pubkey(address.clone(), pubkey_arr);
    }
    // ⛔ DO NOT add to consensus - validators MUST register on-chain
    info!("📡 Validator discovered via P2P - awaiting on-chain registration: {}", 
          &address[..16]);
}
```

**P2P `ValidatorSetResponse` - Discovery Only:**
```rust
P2PMessage::ValidatorSetResponse { validators } => {
    for v_info in validators {
        // ✅ Register pubkeys for signature verification
        if let Some(pubkey) = v_info.pubkey {
            if let Ok(pk_arr) = pubkey.try_into() {
                network.register_validator_pubkey(v_info.address.clone(), pk_arr);
            }
        }
        // ⛔ DO NOT sync to consensus - on-chain state is authoritative
    }
    info!("📡 Received {} validators for discovery (on-chain registration required)", 
          validators.len());
}
```

**On-Chain Validator Registration (Required):**
```bash
# Validators MUST register through the staking RPC endpoint
curl -X POST http://rpc.sltn.io/staking/create_validator \
  -H "Content-Type: application/json" \
  -d '{
    "address": "validator_address",
    "stake_amount": "10000000000000",
    "commission_rate": "0.1"
  }'
```

**Genesis Mode (Single Validator at Start):**
```rust
// Single genesis validator at network start is normal - not an error
let staked_count = consensus.validators().iter()
    .filter(|v| v.stake >= MIN_STAKE).count();

if staked_count == 1 && is_genesis_validator {
    info!("🌱 [GENESIS] Single genesis validator - normal bootstrap operation");
}
```

### Why This Is Enterprise-Grade

| Aspect | Before (Auto-Add) | After (On-Chain Required) |
|--------|-------------------|---------------------------|
| **Validator Set Source** | P2P messages (unreliable) | Blockchain state (canonical) |
| **Consistency** | Divergent across nodes | Identical everywhere |
| **Authorization** | Anyone with P2P access | Must stake tokens on-chain |
| **Audit Trail** | None | Full on-chain history |
| **Attack Surface** | P2P message injection | Requires private key + stake |

### Key Principle
> **P2P is for discovery. Blockchain is for consensus.**
> 
> All validators derive their validator set from on-chain state, ensuring 100% consistency across the network. P2P announcements only register public keys for signature verification.

---

## Issue 4: RocksDB LOCK File Stuck

### Symptoms
- Node crash loop with error: `Failed to start P2P network`
- Or: `IO error: While lock file: /root/sultan-data/blocks/LOCK: Resource temporarily unavailable`

### Root Cause
When the node process was killed ungracefully (e.g., `kill -9`), the RocksDB LOCK file remained, preventing the new process from starting.

### Fix Applied
Manual cleanup before restart:

```bash
ssh root@<validator-ip> "pkill -9 sultan-node; rm -f /root/sultan-data/blocks/LOCK; systemctl start sultan-node"
```

### Prevention
Always use graceful shutdown when possible:
```bash
systemctl stop sultan-node  # Graceful
# NOT: kill -9 <pid>        # Leaves LOCK file
```

---

## Issue 5: Port Already in Use (EADDRINUSE)

### Symptoms
```
Error: Failed to start P2P network
Caused by: Address already in use (os error 98)
```

### Root Cause
Previous node process still holding port 30303 (P2P) or 8545 (RPC).

### Fix Applied
Kill stale processes and wait for port release:

```bash
ssh root@<validator-ip> "pkill -9 sultan-node; sleep 2; lsof -i :30303"
# Should show "Port 30303 free" or no output
```

---

## Issue 6: Corrupted Block Data

### Symptoms
```
WARN Failed to apply stored block 1: Block validation failed
```

### Root Cause
Block data in RocksDB became corrupted (possibly from ungraceful shutdown during write).

### Fix Applied
Clear block data and restart fresh:

```bash
ssh root@<validator-ip> "rm -rf /root/sultan-data/blocks/* /var/lib/sultan/commit-log/* && systemctl start sultan-node"
```

**⚠️ WARNING:** This resets the chain to genesis. Only do this if the chain needs to be restarted.

---

## Deployment Commands Reference

### Build
```bash
# Fast debug build (LTO disabled)
cargo build --profile release-dev -p sultan-core

# Binary location
/tmp/cargo-target/release-dev/sultan-node
```

### Deploy to Single Validator
```bash
HOST=206.189.224.142  # NYC
ssh -i ~/.ssh/sultan_deploy root@$HOST "systemctl stop sultan-node && rm -f /usr/local/bin/sultan-node"
scp -i ~/.ssh/sultan_deploy /tmp/cargo-target/release-dev/sultan-node root@$HOST:/usr/local/bin/sultan-node
ssh -i ~/.ssh/sultan_deploy root@$HOST "chmod +x /usr/local/bin/sultan-node && systemctl start sultan-node"
```

### Deploy to All Validators
```bash
for host in 206.189.224.142 24.144.94.23 46.101.122.13 142.93.238.33 143.198.205.21 134.122.96.36; do
  echo "Deploying to $host..."
  ssh -i ~/.ssh/sultan_deploy root@$host "systemctl stop sultan-node; rm -f /usr/local/bin/sultan-node"
  scp -i ~/.ssh/sultan_deploy /tmp/cargo-target/release-dev/sultan-node root@$host:/usr/local/bin/sultan-node
  ssh -i ~/.ssh/sultan_deploy root@$host "chmod +x /usr/local/bin/sultan-node && systemctl start sultan-node"
done
```

### Fresh Start (Reset Chain)
```bash
ssh -i ~/.ssh/sultan_deploy root@$HOST "pkill -9 sultan-node; rm -rf /root/sultan-data/blocks/* /var/lib/sultan/commit-log/* /root/sultan-data/blocks/LOCK; systemctl start sultan-node"
```

---

## Validator Infrastructure

| Location | IP Address | Role |
|----------|------------|------|
| NYC | 206.189.224.142 | Bootstrap (Primary) |
| SFO | 24.144.94.23 | Validator |
| FRA | 46.101.122.13 | Validator |
| AMS | 142.93.238.33 | Validator |
| SGP | 143.198.205.21 | Validator |
| LON | 134.122.96.36 | Validator |

### NYC Validator Address
```
289d7e71b1bf1ed6fb115a14723a87b9313e81e20ff3cc38a00cbc3ebc1f1799
```

---

## Monitoring Commands

### Check Block Height
```bash
curl -s http://206.189.224.142:8545/status | jq '.height'
```

### Check Full Status
```bash
curl -s http://206.189.224.142:8545/status | jq
```

### Watch Block Production
```bash
ssh -i ~/.ssh/sultan_deploy root@206.189.224.142 "journalctl -u sultan-node -f" | grep -E 'Block.*created|height'
```

### Check for Errors
```bash
ssh -i ~/.ssh/sultan_deploy root@206.189.224.142 "journalctl -u sultan-node -n 100 --no-pager | grep -i error"
```

---

## Key Files Modified

1. **`sultan-core/src/main.rs`**
   - Lines 916-943: Block production loop with debug logging
   - Lines 980-1030: Proposer selection with bootstrap mode
   - Lines 1050-1077: Peer gate (disabled)

2. **`.cargo/config.toml`**
   - Sets `CARGO_TARGET_DIR=/tmp/cargo-target/` for faster builds

---

## Lessons Learned

1. **Never use blocking `.await` on locks in hot paths** - Use `try_read()`/`try_write()` with timeouts
2. **P2P is for discovery, blockchain is for consensus** - Never auto-add validators from P2P messages
3. **On-chain state must be the single source of truth** - All nodes derive validator set from blockchain
4. **Divergent state = network failure** - Any non-deterministic state causes consensus breakdown
5. **Always add debug logging** - Silent failures are impossible to diagnose
6. **Graceful shutdown prevents data corruption** - Use `systemctl stop`, not `kill -9`

---

## Issue 7: Timestamp Collision Bug (CRITICAL)

**Date Discovered:** January 24, 2026  
**Severity:** Critical - Network Stall

### Symptoms
- Network stalled at block 7 across all 6 validators
- Error in logs: `Block timestamp must be greater than previous block`
- Block 6 and Block 7 had identical timestamps: `ts=1769281157, prev_ts=1769281157`

### Root Cause
The `create_block()` function in `sharded_blockchain_production.rs` used `SystemTime::now().as_secs()` directly for block timestamps without ensuring they exceeded the previous block's timestamp.

When validators started simultaneously (within the same second), multiple blocks could be created with identical second-level timestamps, causing validation to fail.

**Problematic code:**
```rust
// BEFORE - could create blocks with same timestamp
let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();
```

### Fix Applied

**File:** `sultan-core/src/sharded_blockchain_production.rs` (lines 200-232)

```rust
// AFTER - ensures strictly increasing timestamps
// Save previous timestamp BEFORE dropping the lock
let prev_timestamp = prev_block.timestamp;

// ... later in create_block() ...

let current_time = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();

// CRITICAL FIX: Ensure timestamp is strictly greater than previous block
// This prevents timestamp collision when blocks are created rapidly
let timestamp = std::cmp::max(current_time, prev_timestamp + 1);
```

### Verification
After deploying the fix, logs show:
```
✓ Timestamp check passed: 1769284767 > 1769284766
```

### Key Principle
> **Block timestamps MUST be strictly increasing.**
> 
> Using `max(current_time, prev_timestamp + 1)` guarantees monotonically increasing timestamps, preventing validation failures even when blocks are produced faster than 1 per second.

---

## Current Status (as of 2026-01-24)

✅ **Sultan Chain is LIVE (Enterprise-Grade)**
- Height: 300+ and counting
- Block time: ~2 seconds
- Validators: 6 (NYC, SGP, AMS, FRA, SFO, LON)
- Shards: 16 active
- Public RPC: https://rpc.sltn.io

**Key Changes in v0.1.7+:**
- P2P `ValidatorAnnounce` for discovery only (pubkey registration)
- P2P `ValidatorSetResponse` for discovery only (pubkey registration)  
- Validators MUST register on-chain via `/staking/create_validator`
- Removed bootstrap mode workaround
- Genesis mode: single validator at start is normal operation
- **CRITICAL FIX:** Timestamp collision bug resolved - blocks always have strictly increasing timestamps
