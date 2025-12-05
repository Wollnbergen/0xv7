#!/usr/bin/env bash
# ./scripts/schema_inspect.sh
set -euo pipefail
Q="SELECT table_name, column_name, type FROM system_schema.columns WHERE keyspace_name='sultan' ALLOW FILTERING;"
SCYLLA=$(docker ps --filter 'publish=9042' --format '{{.Names}}' | head -n1 || true)
if [ -n "${SCYLLA:-}" ]; then
  echo "Using container: $SCYLLA"
  docker exec -it "$SCYLLA" cqlsh -e "$Q"
elif command -v cqlsh >/dev/null 2>&1; then
  cqlsh -e "$Q"
else
  echo "cqlsh not found. Use docker exec with your Scylla container."
  exit 1
fi
