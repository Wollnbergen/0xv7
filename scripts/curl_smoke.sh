#!/usr/bin/env bash
# ./scripts/curl_smoke.sh
set -euo pipefail
: "${TOKEN:?Set TOKEN (source scripts/dev_env.sh)}"
BASE="${BASE:-http://127.0.0.1:3030}"
HDR=(-H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json")

ADDR=$(curl -sS "${HDR[@]}" -d '{"jsonrpc":"2.0","id":1,"method":"wallet_create","params":{"telegram_id":"user123"}}' "$BASE" | jq -r '.result.address')
echo "ADDR=$ADDR"

curl -sS "${HDR[@]}" -d "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"token_mint\",\"params\":{\"to\":\"$ADDR\",\"amount\":1000}}" "$BASE" | jq .
curl -sS "${HDR[@]}" -d "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"wallet_balance\",\"params\":{\"address\":\"$ADDR\"}}" "$BASE" | jq .

# Governance (will be -32601 until wired)
curl -sS "${HDR[@]}" -d '{"jsonrpc":"2.0","id":10,"method":"proposal_create","params":{"proposal_id":"p1","title":"T1","description":"d"}}' "$BASE" | jq .
curl -sS "${HDR[@]}" -d '{"jsonrpc":"2.0","id":11,"method":"vote_on_proposal","params":{"proposal_id":"p1","vote":true,"validator_id":"validator_1"}}' "$BASE" | jq .
curl -sS "${HDR[@]}" -d '{"jsonrpc":"2.0","id":12,"method":"votes_tally","params":{"proposal_id":"p1"}}' "$BASE" | jq .
