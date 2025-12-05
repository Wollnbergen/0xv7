#!/bin/bash
# Deploy Sultan Telegram Bot to Production

set -e

echo "🤖 Deploying Sultan Telegram Bot..."

# Configuration
SERVER="${SULTAN_SERVER:-root@5.161.225.96}"
BOT_TOKEN="${TELEGRAM_BOT_TOKEN}"
API_URL="https://api.sultan.finance"

if [ -z "$BOT_TOKEN" ]; then
    echo "❌ TELEGRAM_BOT_TOKEN not set"
    exit 1
fi

# Build bot
echo "🔨 Building Telegram bot..."
cd telegram-bot
cargo build --release

# Copy to server
echo "📤 Uploading to server..."
scp -i ~/.ssh/sultan-node-2024 \
    ../target/release/sultan-telegram-bot \
    "$SERVER:/usr/local/bin/"

# Create systemd service
echo "⚙️  Creating systemd service..."
ssh -i ~/.ssh/sultan-node-2024 "$SERVER" << 'EOF'
cat > /etc/systemd/system/sultan-telegram-bot.service << 'SERVICE'
[Unit]
Description=Sultan Telegram Bot
After=network.target

[Service]
Type=simple
User=sultan
WorkingDirectory=/var/lib/sultan
Environment="TELEGRAM_BOT_TOKEN=${BOT_TOKEN}"
Environment="API_ENDPOINT=https://api.sultan.finance"
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/sultan-telegram-bot
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
SERVICE

# Enable and start
systemctl daemon-reload
systemctl enable sultan-telegram-bot
systemctl restart sultan-telegram-bot
systemctl status sultan-telegram-bot --no-pager
EOF

echo "✅ Telegram bot deployed!"
echo "🔗 Test at: https://t.me/SultanFinanceBot"
echo "📊 Logs: ssh $SERVER 'journalctl -u sultan-telegram-bot -f'"
