#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    FREEING DISK SPACE                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "Current disk usage:"
df -h /

echo ""
echo "🧹 Cleaning up..."

# Clean cargo build artifacts
cd /workspaces/0xv7
cargo clean 2>/dev/null
echo "✅ Cleaned cargo target directory"

# Clean old Docker images/containers
docker system prune -af --volumes 2>/dev/null
echo "✅ Cleaned Docker system"

# Remove conda cache
conda clean --all -y 2>/dev/null
echo "✅ Cleaned conda cache"

# Clean apt cache
sudo apt-get clean 2>/dev/null
sudo apt-get autoremove -y 2>/dev/null
echo "✅ Cleaned apt cache"

# Remove unnecessary files in target directories
find . -type d -name "target" -exec rm -rf {} + 2>/dev/null
find . -type d -name "node_modules" -exec rm -rf {} + 2>/dev/null
echo "✅ Removed build artifacts"

echo ""
echo "Disk usage after cleanup:"
df -h /

echo ""
echo "✅ Cleanup complete!"
