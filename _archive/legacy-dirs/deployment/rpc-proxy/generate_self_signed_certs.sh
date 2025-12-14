#!/usr/bin/env bash
set -euo pipefail

DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
OUT="$DIR/secrets"
mkdir -p "$OUT"

CRT="$OUT/fullchain.pem"
KEY="$OUT/privkey.pem"

if [[ -f "$CRT" || -f "$KEY" ]]; then
  echo "Certificates already exist in $OUT; skipping."
  exit 0
fi

echo "Generating self-signed TLS certs (for testing only)..."
openssl req -x509 -newkey rsa:4096 -sha256 -days 365 \
  -nodes -keyout "$KEY" -out "$CRT" \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"

chmod 600 "$KEY"
echo "Self-signed certs written to $OUT"
