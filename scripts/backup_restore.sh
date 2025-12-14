#!/usr/bin/env bash
set -euo pipefail

# Restore a snapshot created by backup_snapshot.sh
# Usage: ./scripts/backup_restore.sh /path/to/archive.tar.gz

ARCHIVE=${1:?"Archive file required"}
CONTAINER=${CONTAINER:-cosmos-sultan}

echo "♻️  Restoring snapshot from $ARCHIVE"
[ -f "$ARCHIVE" ] || { echo "❌ Archive not found"; exit 1; }
docker ps --format '{{.Names}}' | grep -qw "$CONTAINER" || { echo "❌ Container $CONTAINER not running"; exit 1; }

TMP_DIR=$(mktemp -d)
tar -xzf "$ARCHIVE" -C "$TMP_DIR"

if [ -f "$TMP_DIR/genesis.json" ]; then
  docker cp "$TMP_DIR/genesis.json" "$CONTAINER:/root/.wasmd/config/genesis.json"
fi
if [ -d "$TMP_DIR/data" ]; then
  docker exec "$CONTAINER" sh -lc 'rm -rf $HOME/.wasmd/data/*'
  docker cp "$TMP_DIR/data" "$CONTAINER:/root/.wasmd/"
fi

rm -rf "$TMP_DIR"
echo "🔄 Restarting container"
docker restart "$CONTAINER" >/dev/null
echo "✅ Restore completed"
