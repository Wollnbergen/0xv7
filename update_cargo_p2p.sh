#!/bin/bash

# Check current libp2p version and add missing features
if grep -q 'libp2p = { version = "0.53"' node/Cargo.toml; then
    echo "Updating libp2p features for v0.53..."
    
    # Create a backup
    cp node/Cargo.toml node/Cargo.toml.backup_day78
    
    # Update libp2p line with all needed features
    sed -i 's/libp2p = .*/libp2p = { version = "0.53", features = ["tcp", "relay", "ping", "dns", "async-std", "noise", "yamux", "kad", "gossipsub", "identify", "tokio"] }/' node/Cargo.toml
fi

echo "✅ Updated Cargo.toml with P2P features"
