#!/usr/bin/env bash
set -euo pipefail
# Build CW20 (and optionally CW721) artifacts locally using workspace-optimizer.
# Produces optimized wasm under ./cw-artifacts/
# Compatible with newer wasmvm (v3) if CWPLUS_TAG is set to 'main' or an updated tag.

ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT_DIR="$ROOT/cw-artifacts"
mkdir -p "$OUT_DIR"

CWPLUS_TAG=${CWPLUS_TAG:-main}
CWNFTS_TAG=${CWNFTS_TAG:-v0.18.0}
OPT_IMAGE=${OPT_IMAGE:-cosmwasm/rust-optimizer:0.14.0}

# Ensure sources
SRC_DIR="$ROOT/third_party"
mkdir -p "$SRC_DIR"

echo "📥 Fetching cw-plus ($CWPLUS_TAG)"
if [ ! -d "$SRC_DIR/cw-plus/.git" ]; then
  git clone --depth 1 --branch "$CWPLUS_TAG" https://github.com/CosmWasm/cw-plus "$SRC_DIR/cw-plus" 2>/dev/null || {
    echo "⚠️ Initial clone failed, retrying full clone"; git clone https://github.com/CosmWasm/cw-plus "$SRC_DIR/cw-plus"; }
else
  git -C "$SRC_DIR/cw-plus" fetch origin "$CWPLUS_TAG" --depth 1 || true
  git -C "$SRC_DIR/cw-plus" checkout "$CWPLUS_TAG" || true
fi
git -C "$SRC_DIR/cw-plus" rev-parse --short HEAD || true

# Build cw20-base
echo "🛠 Building cw20-base ($CWPLUS_TAG)"
if [ ! -d "$SRC_DIR/cw-plus/contracts/cw20-base" ]; then
  echo "❌ cw20-base directory not found under cw-plus; abort"; exit 1; fi
docker run --rm -v "$SRC_DIR/cw-plus/contracts/cw20-base":/code "$OPT_IMAGE"
cp -f "$SRC_DIR/cw-plus/contracts/cw20-base/artifacts/cw20_base.wasm" "$OUT_DIR/"
ls -lh "$OUT_DIR/cw20_base.wasm"
sha256sum "$OUT_DIR/cw20_base.wasm"

# Try CW721 via cw-nfts (optional)
set +e
if [ ! -d "$SRC_DIR/cw-nfts/.git" ]; then
  git clone --depth 1 --branch "$CWNFTS_TAG" https://github.com/CosmWasm/cw-nfts "$SRC_DIR/cw-nfts"
else
  git -C "$SRC_DIR/cw-nfts" fetch --tags --depth 1
  git -C "$SRC_DIR/cw-nfts" checkout "$CWNFTS_TAG"
fi
set -e

if [ -d "$SRC_DIR/cw-nfts/contracts/cw721-base" ]; then
  echo "🛠 Building cw721-base ($CWNFTS_TAG)"
  docker run --rm -v "$SRC_DIR/cw-nfts/contracts/cw721-base":/code "$OPT_IMAGE"
  if [ -f "$SRC_DIR/cw-nfts/contracts/cw721-base/artifacts/cw721_base.wasm" ]; then
    cp -f "$SRC_DIR/cw-nfts/contracts/cw721-base/artifacts/cw721_base.wasm" "$OUT_DIR/"
    ls -lh "$OUT_DIR/cw721_base.wasm"
    sha256sum "$OUT_DIR/cw721_base.wasm"
  else
    echo "⚠️ cw721_base.wasm not produced; skipping"
  fi
else
  echo "ℹ️ cw-nfts cw721-base not found; skipping"
fi

echo "✅ Artifacts ready in $OUT_DIR"
echo "To test CW20 deployment (zero-fee): CONTAINER=cosmos-sultan ./scripts/test_cw20_zero_fee.sh"
