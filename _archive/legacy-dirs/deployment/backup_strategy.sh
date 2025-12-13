#!/bin/bash
# Sultan Chain Backup Strategy

BACKUP_DIR="/backups/sultan"
DATA_DIR="/workspaces/0xv7/data"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create backup
echo "Creating backup at $TIMESTAMP..."
mkdir -p $BACKUP_DIR
tar -czf "$BACKUP_DIR/sultan_backup_$TIMESTAMP.tar.gz" \
    --exclude='node_modules' \
    --exclude='.git' \
    $DATA_DIR

# Keep only last 7 days of backups
find $BACKUP_DIR -name "sultan_backup_*.tar.gz" -mtime +7 -delete

echo "Backup complete: sultan_backup_$TIMESTAMP.tar.gz"
