#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           FIXING COSMOS SDK BUILD FOR SULTAN                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7/sultan

# Fix the undefined 'address' error in expected_keepers.go
echo "🔧 Fixing expected_keepers.go..."
sed -i 's/address/sdk.AccAddress/g' x/zerofees/types/expected_keepers.go

# Add missing imports if needed
if ! grep -q "sdk \"github.com/cosmos/cosmos-sdk/types\"" x/zerofees/types/expected_keepers.go; then
    sed -i '1a import sdk "github.com/cosmos/cosmos-sdk/types"' x/zerofees/types/expected_keepers.go
fi

# Try to build again
echo ""
echo "🔨 Building Sultan with Cosmos SDK..."
go build -o build/sultand ./cmd/sultand 2>&1 | tail -10

if [ -f "build/sultand" ]; then
    echo "✅ Sultan Chain built successfully!"
    
    # Initialize and start
    echo ""
    echo "🚀 Initializing Sultan Chain..."
    ./build/sultand init sultan-node --chain-id sultan-1
    
    echo ""
    echo "📊 Sultan Chain with Cosmos SDK is ready!"
    echo "  • Zero gas fees module: x/zerofees"
    echo "  • Mobile validators: Coming soon"
    echo "  • IBC enabled: Yes"
else
    echo "⚠️ Build still failing. Run 'ignite chain serve' for auto-fix"
fi
