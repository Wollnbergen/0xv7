#!/usr/bin/env bash
set -euo pipefail

# Create a timestamped snapshot of the Cosmos data directory and genesis.
# Usage: ./scripts/backup_snapshot.sh [output-dir]

OUT_DIR=${1:-/workspaces/0xv7/backups}
mkdir -p "$OUT_DIR"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
ARCHIVE="$OUT_DIR/cosmos_snapshot_$STAMP.tar.gz"
CONTAINER=${CONTAINER:-cosmos-sultan}

echo "🗄️  Creating snapshot $ARCHIVE"
docker ps --format '{{.Names}}' | grep -qw "$CONTAINER" || { echo "❌ Container $CONTAINER not running"; exit 1; }

TMP_DIR=$(mktemp -d)
docker cp "$CONTAINER:/root/.wasmd/config/genesis.json" "$TMP_DIR/genesis.json" 2>/dev/null || echo "⚠️ Could not copy genesis.json"
docker cp "$CONTAINER:/root/.wasmd/data" "$TMP_DIR/data" 2>/dev/null || echo "⚠️ Could not copy data dir"

tar -czf "$ARCHIVE" -C "$TMP_DIR" .
rm -rf "$TMP_DIR"
echo "✅ Snapshot created: $ARCHIVE"
