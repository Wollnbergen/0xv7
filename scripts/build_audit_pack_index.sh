#!/usr/bin/env bash
set -euo pipefail

ROOT=${1:-/workspaces/0xv7}
OUT_DIR="$ROOT/audit_pack"
mkdir -p "$OUT_DIR"

latest_file() { ls -1t $1 2>/dev/null | head -n1 || true; }

BACKUP=$(latest_file $ROOT/backups/cosmos_snapshot_*.tar.gz)
SLASH_REPORT=$(latest_file $ROOT/security/reports/slashing_probe_*.json)
VAL_STATUS=$(latest_file $ROOT/security/reports/validator_status_*.json)
PANIC_REPORT="$ROOT/security/reports/panic_scan.json"
SECRET_REPORT="$ROOT/security/reports/secret_scan.json"

jq -n \
  --arg backup "$BACKUP" \
  --arg slashing "$SLASH_REPORT" \
  --arg panic "$PANIC_REPORT" \
  --arg secrets "$SECRET_REPORT" \
  --arg zero_gas_script "$ROOT/scripts/zero_gas_full_test.sh" \
  --arg validator_lifecycle "$ROOT/scripts/validator_lifecycle_test.sh" \
  --arg val_status "$VAL_STATUS" \
  '{
    artifacts: {
      backup_archive: $backup,
      slashing_probe_report: $slashing,
      validator_status_report: $val_status,
      panic_scan_report: $panic,
      secret_scan_report: $secrets
    },
    scripts: {
      zero_gas_full_test: $zero_gas_script,
      validator_lifecycle_test: $validator_lifecycle
    }
  }' > "$OUT_DIR/index.json"

echo "✅ Wrote $OUT_DIR/index.json"
