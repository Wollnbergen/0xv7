#!/bin/bash
set -euo pipefail
if ! command -v docker >/dev/null 2>&1 || ! command -v jq >/dev/null 2>&1 || ! command -v bc >/dev/null 2>&1; then
  echo "N/A"; exit 0
fi
INFLATION_JSON=$(docker exec cosmos-node wasmd query mint inflation -o json 2>/dev/null || true)
POOL_JSON=$(docker exec cosmos-node wasmd query staking pool -o json 2>/dev/null || true)
DIST_JSON=$(docker exec cosmos-node wasmd query distribution params -o json 2>/dev/null || true)
if [ -z "$INFLATION_JSON" ] || [ -z "$POOL_JSON" ]; then echo "N/A"; exit 0; fi
INFLATION=$(echo "$INFLATION_JSON" | jq -r '.inflation // empty')
COMM_TAX=$(echo "$DIST_JSON" | jq -r '.params.community_tax // 0' 2>/dev/null || echo "0")
BONDED=$(echo "$POOL_JSON" | jq -r '.bonded_tokens // empty')
NOTBONDED=$(echo "$POOL_JSON" | jq -r '.not_bonded_tokens // empty')
if [ -z "$INFLATION" ] || [ -z "$BONDED" ] || [ -z "$NOTBONDED" ]; then echo "N/A"; exit 0; fi
TOTAL=$(echo "$BONDED + $NOTBONDED" | bc -l)
if [ "$(echo "$TOTAL == 0" | bc -l)" -eq 1 ]; then echo "N/A"; exit 0; fi
BONDED_RATIO=$(echo "scale=10; $BONDED / $TOTAL" | bc -l)
if [ "$(echo "$BONDED_RATIO == 0" | bc -l)" -eq 1 ]; then echo "N/A"; exit 0; fi
APY=$(echo "scale=6; ($INFLATION / $BONDED_RATIO) * (1 - $COMM_TAX) * 100" | bc -l)
printf "%.2f\n" "$APY"
