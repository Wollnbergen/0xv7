#!/usr/bin/env bash
set -euo pipefail

CONTAINER=${1:-cosmos-sultan}

echo "🔎 Zero-fee ante test against container: $CONTAINER"

echo "⛓ Height before"
curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height' || true

VAL=$(docker exec "$CONTAINER" wasmd keys show validator -a --keyring-backend test)
RECIP_JSON=$(docker exec "$CONTAINER" sh -lc 'wasmd keys add anteuser --keyring-backend test --output json 2>/dev/null || wasmd keys show anteuser --keyring-backend test --output json')
RECIP=$(echo "$RECIP_JSON" | jq -r .address)
echo "validator=$VAL"
echo "anteuser=$RECIP"

echo "💸 Sending 1234usltn with --fees 0usltn"
TX=$(docker exec "$CONTAINER" sh -lc 'wasmd tx bank send validator '"$RECIP"' 1234usltn --fees 0usltn --yes --chain-id sultan-1 --broadcast-mode sync --keyring-backend test -o json' || true)
echo "$TX" | jq -r '"code="+(.code|tostring)+" log="+(.raw_log//"")'

echo "👛 Recipient balance"
docker exec "$CONTAINER" wasmd q bank balances "$RECIP" -o json | jq -r '.balances[]? | .denom+":"+.amount'

echo "⛓ Height after"
curl -sf http://127.0.0.1:26657/status | jq -r '.result.sync_info.latest_block_height' || true

echo "✅ Zero-fee ante test completed"
