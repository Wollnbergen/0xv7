#!/bin/bash
echo "🚀 Starting Sultan Chain..."

# Start web interface
if ! pgrep -f "python3 -m http.server 3000" > /dev/null; then
    cd /workspaces/0xv7/public && python3 -m http.server 3000 > /tmp/web.log 2>&1 &
    echo "✅ Web interface: http://localhost:3000"
fi

# Run node
if [ -f /workspaces/0xv7/target/release/sultan_node ]; then
    /workspaces/0xv7/target/release/sultan_node
elif [ -f /workspaces/0xv7/target/debug/sultan_node ]; then
    /workspaces/0xv7/target/debug/sultan_node
else
    echo "Building node first..."
    cd /workspaces/0xv7 && cargo build --package sultan-coordinator --bin sultan_node
fi
