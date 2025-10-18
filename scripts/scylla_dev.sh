# filepath: /workspaces/0xv7/scripts/scylla_dev.sh
#!/usr/bin/env bash
set -euo pipefail

SCYLLA_CONTAINER="${SCYLLA_CONTAINER:-scylla}"
SCYLLA_IMAGE="${SCYLLA_IMAGE:-scylladb/scylla:5.4}"
SCYLLA_VOLUME="${SCYLLA_VOLUME:-scylla-data}"
KEYSPACE="${KEYSPACE:-sultan}"

run_container() {
  echo "Creating container '$SCYLLA_CONTAINER'..."
  docker run -d --name "$SCYLLA_CONTAINER" \
    -p 9042:9042 -p 9100:9100 -p 9180:9180 \
    -v "$SCYLLA_VOLUME:/var/lib/scylla" \
    "$SCYLLA_IMAGE" --smp 1 --memory 1G --overprovisioned 1 --developer-mode 1 >/dev/null
}

start_container() {
  if ! docker inspect "$SCYLLA_CONTAINER" >/dev/null 2>&1; then
    run_container
  else
    docker start "$SCYLLA_CONTAINER" >/dev/null || true
  fi
}

wait_for_cql() {
  echo "Waiting for cqlsh on $SCYLLA_CONTAINER..."
  for i in {1..120}; do
    if docker exec "$SCYLLA_CONTAINER" cqlsh -e "SELECT release_version FROM system.local;" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  echo "Timed out waiting for cqlsh." >&2
  exit 1
}

cql() {
  docker exec -i "$SCYLLA_CONTAINER" cqlsh "$@"
}

ensure_schema() {
  cql <<CQL
CREATE KEYSPACE IF NOT EXISTS $KEYSPACE
WITH replication = {'class':'SimpleStrategy','replication_factor':'1'}
AND durable_writes = true;

CREATE TABLE IF NOT EXISTS $KEYSPACE.wallets (
  address text PRIMARY KEY,
  balance bigint,
  created_at bigint,
  last_update_tx text,
  updated_at bigint
);

CREATE TABLE IF NOT EXISTS $KEYSPACE.transfers (
  tx_id text PRIMARY KEY,
  amount bigint,
  created_at bigint,
  from_address text,
  last_error text,
  status text,
  to_address text,
  updated_at bigint
);

CREATE TABLE IF NOT EXISTS $KEYSPACE.proposals (
  proposal_id text PRIMARY KEY,
  created_at timestamp,
  description text,
  status text,
  title text
);

CREATE TABLE IF NOT EXISTS $KEYSPACE.validators (
  validator_id text PRIMARY KEY,
  address text,
  metadata text,
  stake bigint
);

CREATE TABLE IF NOT EXISTS $KEYSPACE.votes (
  proposal_id text,
  validator_id text,
  ts timestamp,
  vote boolean,
  PRIMARY KEY (proposal_id, validator_id)
);

CREATE TABLE IF NOT EXISTS $KEYSPACE.token_supply (
  name text PRIMARY KEY,
  total bigint
);

CREATE INDEX IF NOT EXISTS transfers_from_idx   ON $KEYSPACE.transfers (from_address);
CREATE INDEX IF NOT EXISTS transfers_status_idx ON $KEYSPACE.transfers (status);
CREATE INDEX IF NOT EXISTS transfers_to_idx     ON $KEYSPACE.transfers (to_address);
CQL
}

seed() {
  ensure_schema
  local FROM="0x8b4f0f5e26e61d80000000008b4f0f5e26e61d80"
  local TO="0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  local NOW_MS; NOW_MS=$(date +%s%3N)
  local TX="tx_$(cat /proc/sys/kernel/random/uuid | tr -d -)"

  cql <<CQL
INSERT INTO $KEYSPACE.wallets (address, balance, created_at, last_update_tx, updated_at)
VALUES ('$FROM', 0, $NOW_MS, 'create:seed', $NOW_MS);

INSERT INTO $KEYSPACE.transfers (tx_id, amount, created_at, from_address, status, to_address, updated_at)
VALUES ('$TX', 1000, $NOW_MS, '$FROM', 'pending', '$TO', $NOW_MS);
CQL
  echo "Seed complete. Sample TX: $TX"
}

reset() {
  ensure_schema
  cql <<CQL
TRUNCATE $KEYSPACE.transfers;
TRUNCATE $KEYSPACE.wallets;
TRUNCATE $KEYSPACE.votes;
TRUNCATE $KEYSPACE.validators;
TRUNCATE $KEYSPACE.proposals;
TRUNCATE $KEYSPACE.token_supply;
CQL
  echo "All tables in keyspace '$KEYSPACE' truncated."
}

status_cmd() {
  local state
  state="$(docker inspect -f '{{.State.Status}}' "$SCYLLA_CONTAINER" 2>/dev/null || echo 'absent')"
  echo "Container: $SCYLLA_CONTAINER ($state)"
  if [ "$state" = "running" ]; then
    ensure_schema
    cql -e "SELECT keyspace_name FROM system_schema.keyspaces WHERE keyspace_name='$KEYSPACE';" || true
    cql -e "SELECT count(*) AS wallets FROM $KEYSPACE.wallets;" || true
    cql -e "SELECT count(*) AS transfers FROM $KEYSPACE.transfers;" || true
  fi
}

recreate() {
  echo "Removing container '$SCYLLA_CONTAINER' (volume '$SCYLLA_VOLUME' preserved)..."
  docker rm -f "$SCYLLA_CONTAINER" >/dev/null 2>&1 || true
  run_container
  wait_for_cql
  status_cmd
}

logs() {
  docker logs --tail=200 -f "$SCYLLA_CONTAINER"
}

main() {
  case "${1:-status}" in
    start)    start_container; wait_for_cql; status_cmd ;;
    status)   status_cmd ;;
    seed)     start_container; wait_for_cql; seed; status_cmd ;;
    reset)    start_container; wait_for_cql; reset; status_cmd ;;
    recreate) recreate ;;
    logs)     logs ;;
    *) echo "Usage: $0 [start|status|seed|reset|recreate|logs]"; exit 1 ;;
  esac
}
main "$@"
