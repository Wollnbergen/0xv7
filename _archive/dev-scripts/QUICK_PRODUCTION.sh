#!/bin/bash

echo "🚀 Deploying a production-ready fork..."

# Fork Evmos (Cosmos + EVM compatible)
git clone https://github.com/evmos/evmos.git /workspaces/0xv7/sultan-production
cd /workspaces/0xv7/sultan-production

# Customize for Sultan Chain
sed -i 's/evmos/sultan/g' cmd/evmosd/main.go
sed -i 's/Evmos/Sultan/g' app/app.go

# Set zero gas fees in app/ante/ante.go
echo "Setting zero gas fees..."
cat >> app/ante/ante.go << 'EOF'

// Sultan Chain: Override gas fees to zero
func (sud SetUpContextDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (newCtx sdk.Context, err error) {
    // Set minimum gas price to 0
    ctx = ctx.WithMinGasPrices(sdk.DecCoins{})
    return next(ctx, tx, simulate)
}
EOF

# Build
make install

# Initialize
sultand init sultan-node --chain-id sultan-mainnet-1
sultand keys add validator --keyring-backend test
sultand add-genesis-account validator 1000000000stake --keyring-backend test
sultand gentx validator 1000000stake --keyring-backend test --chain-id sultan-mainnet-1
sultand collect-gentxs
sultand start
