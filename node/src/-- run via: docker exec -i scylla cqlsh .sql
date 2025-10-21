-- run via: docker exec -i scylla cqlsh <<'CQL' ... CQL
CREATE KEYSPACE IF NOT EXISTS sultan WITH replication = {'class':'SimpleStrategy','replication_factor':1};
USE sultan;

CREATE TABLE IF NOT EXISTS wallets (
  address text PRIMARY KEY,
  balance bigint,
  created_at bigint,
  updated_at bigint,
  last_update_tx text
);

CREATE TABLE IF NOT EXISTS transfers (
  tx_id text PRIMARY KEY,
  from_address text,
  to_address text,
  amount bigint,
  status text,
  created_at bigint,
  updated_at bigint,
  last_error text
);

CREATE TABLE IF NOT EXISTS validators (
  validator_id text PRIMARY KEY,
  address text,
  stake bigint,
  metadata text
);

CREATE TABLE IF NOT EXISTS proposals (
  proposal_id text PRIMARY KEY,
  title text,
  description text,
  created_at timestamp,
  status text
);

CREATE TABLE IF NOT EXISTS votes (
  proposal_id text,
  validator_id text,
  vote boolean,
  ts timestamp,
  PRIMARY KEY (proposal_id, validator_id)
);

CREATE TABLE IF NOT EXISTS token_supply (
  name text PRIMARY KEY,
  total bigint
);

CREATE INDEX IF NOT EXISTS transfers_status_idx ON transfers (status);
CREATE INDEX IF NOT EXISTS transfers_from_idx   ON transfers (from_address);
CREATE INDEX IF NOT EXISTS transfers_to_idx     ON transfers (to_address);