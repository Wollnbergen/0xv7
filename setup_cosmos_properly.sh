#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          PROPER COSMOS SDK SETUP FOR SULTAN                   ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd /workspaces/0xv7

# 1. Install Ignite CLI properly
echo "🔧 Installing Ignite CLI..."
if [ -f "./ignite" ]; then
    echo "Moving ignite to PATH..."
    sudo mv ./ignite /usr/local/bin/
    sudo chmod +x /usr/local/bin/ignite
    echo "✅ Ignite installed"
else
    curl https://get.ignite.com/cli | bash
    sudo mv ignite /usr/local/bin/
fi

# Verify installation
ignite version 2>/dev/null && echo "✅ Ignite CLI ready" || echo "❌ Ignite installation failed"

# 2. Check existing Sultan Cosmos structure
echo ""
echo "📂 Checking existing Sultan structure..."
if [ -d "sultan" ]; then
    echo "Found existing sultan directory:"
    ls -la sultan/ | head -5
    
    # Try to build existing structure
    cd sultan
    if [ -f "go.mod" ]; then
        echo "🔨 Building existing Sultan chain..."
        go build -o sultand ./cmd/sultand 2>/dev/null && echo "✅ Build successful" || echo "⚠️ Build failed"
    fi
else
    echo "Creating new Sultan chain with Cosmos SDK..."
    
    # Scaffold new chain
    ignite scaffold chain sultan \
        --address-prefix sultan \
        --no-module \
        --skip-git
    
    cd sultan
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "  1. cd sultan"
echo "  2. ignite scaffold module zerofees"
echo "  3. ignite scaffold module mobile"
echo "  4. ignite chain serve"
