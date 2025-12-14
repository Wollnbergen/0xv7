#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
PROXY_DIR="$ROOT_DIR/deployment/rpc-proxy"

if ! command -v docker >/dev/null 2>&1; then
  echo "❌ Docker not available. Install or start Docker and retry."
  exit 1
fi

if ! command -v docker compose >/dev/null 2>&1 && ! command -v docker-compose >/dev/null 2>&1; then
  echo "❌ docker compose not available. Install docker-compose-plugin or docker-compose."
  exit 1
fi

mkdir -p "$PROXY_DIR/secrets" "$PROXY_DIR/logs"

CRT="$PROXY_DIR/secrets/fullchain.pem"
KEY="$PROXY_DIR/secrets/privkey.pem"

if [[ ! -f "$CRT" || ! -f "$KEY" ]]; then
  echo "⚠️  No TLS certs found. Generating self-signed certs (for testing)."
  bash "$PROXY_DIR/generate_self_signed_certs.sh"
fi

echo "🚀 Starting Sultan RPC TLS proxy on :443 (host network)"
if command -v docker compose >/dev/null 2>&1; then
  (cd "$PROXY_DIR" && docker compose up -d)
else
  (cd "$PROXY_DIR" && docker-compose up -d)
fi

echo "✅ Proxy started. Test with:"
echo "  curl -k https://localhost/status | head -c 400"
