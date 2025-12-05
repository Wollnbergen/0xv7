#!/bin/bash

echo "📝 Applying Day 5-6 updates to RPC server..."

# Backup current rpc_server.rs
cp node/src/rpc_server.rs node/src/rpc_server.rs.backup_day56

# Add imports if not already present
if ! grep -q "use crate::token_transfer" node/src/rpc_server.rs; then
    sed -i '1a\use crate::token_transfer::{Transfer, TransferManager, TransferStatus};' node/src/rpc_server.rs
    sed -i '2a\use crate::rewards::{RewardManager, RewardCalculation};' node/src/rpc_server.rs
fi

# Add the new RPC methods to the server builder
# This needs to be done manually as the structure is complex

echo "✅ Updates applied"
echo ""
echo "⚠️  IMPORTANT: You need to manually add these RPC methods to the server builder:"
echo "   .with_method(\"token_transfer\", token_transfer)"
echo "   .with_method(\"calculate_rewards\", calculate_rewards)"
echo "   .with_method(\"claim_rewards\", claim_rewards)"
echo "   .with_method(\"get_transfer_history\", get_transfer_history)"
echo ""
echo "Add them in the main() function where other methods are registered."
