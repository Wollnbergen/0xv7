#!/usr/bin/env bash
#
# Deploy Sultan Wallet to Production (Replit via PWA repo)
# 
# The Sultan Wallet is hosted on Replit, which pulls from the PWA repo:
#   - Source: wallet-extension/ (in 0xv7 repo)
#   - Deploy: Wollnbergen/PWA repo → Replit auto-syncs → wallet.sltn.io
#   - Backup: rpc.sltn.io/wallet/ (NYC server)
#
# Workflow:
#   1. Make changes in wallet-extension/
#   2. Run this script to sync to PWA repo
#   3. On Replit: git pull && npm install && npm run build
#
# Usage: 
#   ./scripts/deploy_wallet.sh          # Sync wallet-extension → PWA repo
#   ./scripts/deploy_wallet.sh --push   # Sync and push PWA repo to GitHub
#   ./scripts/deploy_wallet.sh --backup # Also deploy to NYC backup server
#
set -euo pipefail

# Configuration
WALLET_DIR="/workspaces/0xv7/wallet-extension"
PWA_REPO_DIR="/workspaces/PWA"

# Backup server (NYC)
SSH_KEY="$HOME/.ssh/sultan_deploy"
NYC_HOST="root@206.189.224.142"
BACKUP_DIR="/var/www/wallet.sltn.io"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${YELLOW}[INFO]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step()  { echo -e "${BLUE}[STEP]${NC} $1"; }

# Parse arguments
PUSH_TO_GIT=false
DEPLOY_BACKUP=false
for arg in "$@"; do
    case $arg in
        --push) PUSH_TO_GIT=true ;;
        --backup) DEPLOY_BACKUP=true ;;
    esac
done

echo "╔════════════════════════════════════════════════════════════╗"
echo "║      Sultan Wallet Deployment Script (PWA → Replit)        ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check PWA repo exists
if [ ! -d "$PWA_REPO_DIR" ]; then
    log_error "PWA repo not found at $PWA_REPO_DIR"
    log_info "Clone it: git clone https://github.com/Wollnbergen/PWA.git /workspaces/PWA"
    exit 1
fi

# Step 1: Sync source files to PWA repo
log_step "1/4 Syncing wallet-extension → PWA repo..."
rsync -av --delete \
    --exclude='node_modules' \
    --exclude='dist' \
    --exclude='.git' \
    --exclude='*.zip' \
    "$WALLET_DIR/" "$PWA_REPO_DIR/"
log_ok "Source files synced to PWA repo"

# Step 2: Show what changed
log_step "2/4 Checking changes..."
cd "$PWA_REPO_DIR"
git status --short

# Step 3: Push to GitHub (optional)
if [ "$PUSH_TO_GIT" = true ]; then
    log_step "3/4 Committing and pushing PWA repo..."
    cd "$PWA_REPO_DIR"
    
    git add -A
    
    if git diff --cached --quiet; then
        log_info "No changes to commit"
    else
        git commit -m "sync: wallet-extension updates $(date +%Y-%m-%d)"
        git push origin main
        log_ok "Pushed to Wollnbergen/PWA - Now run on Replit:"
        log_info "  git pull && npm install && npm run build"
    fi
else
    log_info "3/4 Skipping git push (use --push flag)"
    log_info "    Changes staged in PWA repo - commit manually if needed"
fi

# Step 4: Deploy to backup server (optional)
if [ "$DEPLOY_BACKUP" = true ]; then
    log_step "4/4 Deploying to NYC backup server..."
    cd "$WALLET_DIR"
    npm run build 2>&1 | tail -3
    rsync -avz --delete -e "ssh -i $SSH_KEY" \
        "$WALLET_DIR/dist/" \
        "$NYC_HOST:$BACKUP_DIR/" 2>&1 | tail -3
    log_ok "Backup deployed to rpc.sltn.io/wallet/"
else
    log_info "4/4 Skipping backup deploy (use --backup flag)"
fi

# Summary
echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                   Deployment Summary                        ║"
echo "╠════════════════════════════════════════════════════════════╣"
echo "║                                                              ║"
echo "║  📦 Source: wallet-extension/ (0xv7 repo)                   ║"
echo "║  📦 Deploy: Wollnbergen/PWA repo                            ║"
echo "║  🌐 Production: https://wallet.sltn.io (Replit)             ║"
echo "║  🔄 Backup: https://rpc.sltn.io/wallet/ (NYC)               ║"
echo "║                                                              ║"
echo "║  🚀 NEXT STEPS ON REPLIT:                                   ║"
echo "║     1. git pull origin main                                 ║"
echo "║     2. npm install                                          ║"
echo "║     3. npm run build                                        ║"
echo "║                                                              ║"
echo "╚════════════════════════════════════════════════════════════╝"

