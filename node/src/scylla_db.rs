use anyhow::Result;
use bincode;
use rocksdb::DB;
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use scylla::statement::batch::{Batch, BatchType};
use scylla::statement::prepared::PreparedStatement;
use scylla::statement::Consistency;
use serde_json::to_string;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::types::{Block, Transaction};

#[derive(Debug)]
pub struct ScyllaCluster {
    pub session: Arc<Session>,
    pub keyspace: String,
    pub prepared_queries: Arc<RwLock<PreparedQueries>>,
}

#[derive(Debug)]
pub struct PreparedQueries {
    // Blocks (legacy support)
    pub insert_block: PreparedStatement,
    pub get_block: PreparedStatement,
    pub get_latest_block: PreparedStatement,

    // Transfers (current schema)
    pub insert_transfer: PreparedStatement,
    pub get_transfer: PreparedStatement,
    pub update_transfer_status: PreparedStatement,
    pub list_transfers_from_100: PreparedStatement,
    pub list_transfers_to_100: PreparedStatement,
    pub list_transfers_status_100: PreparedStatement,

    // Validators
    pub upsert_validator: PreparedStatement,
    pub get_validator: PreparedStatement,
    pub update_validator_stake: PreparedStatement,

    // Wallets
    pub insert_wallet_new: PreparedStatement,
    pub get_wallet: PreparedStatement,
    pub update_wallet_balance_new: PreparedStatement,

    // Governance
    pub insert_proposal_new: PreparedStatement,
    pub get_proposal: PreparedStatement,
    pub insert_vote_new: PreparedStatement,
    pub list_votes_for_proposal: PreparedStatement,

    // Token supply
    pub set_token_supply: PreparedStatement,
    pub get_token_supply: PreparedStatement,
}

#[derive(Debug, Clone)]
pub struct Wallet {
    pub address: String,
    pub balance: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_update_tx: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub validator_id: String,
    pub address: String,
    pub stake: i64,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub proposal_id: String,
    pub title: String,
    pub description: String,
    pub created_at: i64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct VoteTally {
    pub for_count: i64,
    pub against_count: i64,
}

impl ScyllaCluster {
    pub async fn from_env() -> Result<Self> {
        let contact_points =
            env::var("SCYLLA_CONTACT_POINTS").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
        let keyspace = env::var("SCYLLA_KEYSPACE").unwrap_or_else(|_| "sultan".to_string());
        let create_keyspace = env::var("SCYLLA_CREATE_KEYSPACE")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let replication_json = env::var("SCYLLA_REPLICATION_JSON")
            .unwrap_or_else(|_| r#"{"class":"SimpleStrategy","replication_factor":1}"#.to_string());

        let nodes: Vec<String> = contact_points
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let session: Arc<Session> =
            Arc::new(SessionBuilder::new().known_nodes(nodes).build().await?);

        if create_keyspace {
            session
                .query_iter(
                    format!(
                        "CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {}",
                        keyspace, replication_json
                    ),
                    &[],
                )
                .await
                .ok();
        }
        session.use_keyspace(&keyspace, false).await?;

        Self::create_tables(&session).await?;
        let prepared_queries = Self::prepare_queries(&session).await?;

        Ok(Self {
            session,
            keyspace,
            prepared_queries: Arc::new(RwLock::new(prepared_queries)),
        })
    }

    pub async fn new(nodes: Vec<&str>, keyspace: &str) -> Result<Self> {
        let session: Arc<Session> =
            Arc::new(SessionBuilder::new().known_nodes(nodes).build().await?);
        session.use_keyspace(keyspace, false).await?;

        Self::create_tables(&session).await?;
        let prepared_queries = Self::prepare_queries(&session).await?;

        Ok(Self {
            session,
            keyspace: keyspace.to_string(),
            prepared_queries: Arc::new(RwLock::new(prepared_queries)),
        })
    }

    async fn create_tables(session: &Session) -> Result<()> {
        session
            .query_iter(
                "CREATE TABLE IF NOT EXISTS blocks (
                    shard_id INT,
                    height BIGINT,
                    hash TEXT,
                    previous_hash TEXT,
                    timestamp BIGINT,
                    validator TEXT,
                    signature TEXT,
                    state_root BLOB,
                    transactions LIST<TEXT>,
                    PRIMARY KEY (shard_id, height)
                ) WITH CLUSTERING ORDER BY (height DESC)",
                &[],
            )
            .await?;

        session
            .query_iter(
                "CREATE TABLE IF NOT EXISTS wallets (
                    address text PRIMARY KEY,
                    balance bigint,
                    created_at bigint,
                    updated_at bigint,
                    last_update_tx text
                )",
                &[],
            )
            .await?;

        session
            .query_iter(
                "CREATE TABLE IF NOT EXISTS transfers (
                    tx_id text PRIMARY KEY,
                    from_address text,
                    to_address text,
                    amount bigint,
                    status text,
                    created_at bigint,
                    updated_at bigint,
                    last_error text
                )",
                &[],
            )
            .await?;
        session
            .query_iter(
                "CREATE INDEX IF NOT EXISTS transfers_status_idx ON transfers (status)",
                &[],
            )
            .await
            .ok();
        session
            .query_iter(
                "CREATE INDEX IF NOT EXISTS transfers_from_idx ON transfers (from_address)",
                &[],
            )
            .await
            .ok();
        session
            .query_iter(
                "CREATE INDEX IF NOT EXISTS transfers_to_idx ON transfers (to_address)",
                &[],
            )
            .await
            .ok();

        session
            .query_iter(
                "CREATE TABLE IF NOT EXISTS validators (
                    validator_id text PRIMARY KEY,
                    address text,
                    stake bigint,
                    metadata text
                )",
                &[],
            )
            .await?;

        session
            .query_iter(
                "CREATE TABLE IF NOT EXISTS proposals (
                    proposal_id text PRIMARY KEY,
                    title text,
                    description text,
                    created_at bigint,
                    status text
                )",
                &[],
            )
            .await?;

        session
            .query_iter(
                "CREATE TABLE IF NOT EXISTS votes (
                    proposal_id text,
                    validator_id text,
                    vote boolean,
                    ts bigint,
                    PRIMARY KEY (proposal_id, validator_id)
                )",
                &[],
            )
            .await?;

        session
            .query_iter(
                "CREATE TABLE IF NOT EXISTS token_supply (
                    name text PRIMARY KEY,
                    total bigint
                )",
                &[],
            )
            .await?;

        Ok(())
    }

    async fn prepare_queries(session: &Session) -> Result<PreparedQueries> {
        Ok(PreparedQueries {
            insert_block: session
                .prepare(
                    "INSERT INTO blocks (shard_id, height, hash, previous_hash, timestamp, validator, signature, state_root, transactions)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .await?,
            get_block: session
                .prepare("SELECT * FROM blocks WHERE shard_id = ? AND height = ?")
                .await?,
            get_latest_block: session
                .prepare("SELECT * FROM blocks WHERE shard_id = ? ORDER BY height DESC LIMIT 1")
                .await?,

            insert_transfer: session
                .prepare(
                    "INSERT INTO transfers (tx_id, from_address, to_address, amount, status, created_at, updated_at, last_error)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .await?,
            get_transfer: session
                .prepare("SELECT tx_id, from_address, to_address, amount, status, created_at, updated_at, last_error FROM transfers WHERE tx_id = ?")
                .await?,
            update_transfer_status: session
                .prepare("UPDATE transfers SET status = ?, updated_at = ?, last_error = ? WHERE tx_id = ?")
                .await?,
            list_transfers_from_100: session
                .prepare("SELECT tx_id, from_address, to_address, amount, status, created_at, updated_at, last_error FROM transfers WHERE from_address = ? LIMIT 100")
                .await?,
            list_transfers_to_100: session
                .prepare("SELECT tx_id, from_address, to_address, amount, status, created_at, updated_at, last_error FROM transfers WHERE to_address = ? LIMIT 100")
                .await?,
            list_transfers_status_100: session
                .prepare("SELECT tx_id, from_address, to_address, amount, status, created_at, updated_at, last_error FROM transfers WHERE status = ? LIMIT 100")
                .await?,

            upsert_validator: session
                .prepare("INSERT INTO validators (validator_id, address, stake, metadata) VALUES (?, ?, ?, ?)")
                .await?,
            get_validator: session
                .prepare("SELECT validator_id, address, stake, metadata FROM validators WHERE validator_id = ?")
                .await?,
            update_validator_stake: session
                .prepare("UPDATE validators SET stake = ? WHERE validator_id = ?")
                .await?,

            insert_wallet_new: session
                .prepare("INSERT INTO wallets (address, balance, created_at, updated_at, last_update_tx) VALUES (?, ?, ?, ?, ?) IF NOT EXISTS")
                .await?,
            get_wallet: session
                .prepare("SELECT address, balance, created_at, updated_at, last_update_tx FROM wallets WHERE address = ?")
                .await?,
            update_wallet_balance_new: session
                .prepare("UPDATE wallets SET balance = balance + ?, updated_at = ?, last_update_tx = ? WHERE address = ?")
                .await?,

            insert_proposal_new: session
                .prepare("INSERT INTO proposals (proposal_id, title, description, created_at, status) VALUES (?, ?, ?, ?, ?)")
                .await?,
            get_proposal: session
                .prepare("SELECT proposal_id, title, description, created_at, status FROM proposals WHERE proposal_id = ?")
                .await?,
            insert_vote_new: session
                .prepare("INSERT INTO votes (proposal_id, validator_id, vote, ts) VALUES (?, ?, ?, ?)")
                .await?,
            list_votes_for_proposal: session
                .prepare("SELECT vote FROM votes WHERE proposal_id = ?")
                .await?,

            set_token_supply: session
                .prepare("UPDATE token_supply SET total = ? WHERE name = ?")
                .await?,
            get_token_supply: session
                .prepare("SELECT total FROM token_supply WHERE name = ?")
                .await?,
        })
    }

    pub async fn insert_block(&self, shard_id: i32, block: &Block) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        let tx_strings: Vec<String> = block
            .transactions
            .iter()
            .map(|tx| to_string(tx).unwrap_or_default())
            .collect();
        self.session
            .execute_iter(
                queries.insert_block.clone(),
                (
                    shard_id,
                    block.height as i64,
                    &block.hash,
                    &block.previous_hash,
                    block.timestamp as i64,
                    &block.validator,
                    &block.signature,
                    &block.state_root,
                    tx_strings,
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn insert_wallet(
        &self,
        telegram_id: &str,
        address: &str,
        _pk: &[u8],
        _sk: &[u8],
        created_at: i64,
    ) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        let updated_at = created_at;
        let last_update_tx = Some(format!("tg:{telegram_id}"));
        self.session
            .execute_iter(
                queries.insert_wallet_new.clone(),
                (address, 0_i64, created_at, updated_at, last_update_tx),
            )
            .await?;
        Ok(())
    }

    pub async fn create_wallet(&self, address: &str, created_at_ms: i64) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        self.session
            .execute_iter(
                queries.insert_wallet_new.clone(),
                (
                    address,
                    0_i64,
                    created_at_ms,
                    created_at_ms,
                    Option::<String>::None,
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn get_wallet(&self, address: &str) -> Result<Option<Wallet>> {
        let queries = self.prepared_queries.read().await;
        let rs = self
            .session
            .execute_unpaged(&queries.get_wallet, (address,))
            .await?;
        let rows_res = rs.into_rows_result()?;
        let mut it = rows_res.rows::<(String, i64, i64, i64, Option<String>)>()?;
        if let Some(row) = it.next() {
            let (addr, balance, created_at, updated_at, last_update_tx) = row?;
            return Ok(Some(Wallet {
                address: addr,
                balance,
                created_at,
                updated_at,
                last_update_tx,
            }));
        }
        Ok(None)
    }

    pub async fn update_wallet_balance(&self, address: &str, amount: i64) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        let updated_at_ms: i64 = chrono::Utc::now().timestamp_millis();
        self.session
            .execute_iter(
                queries.update_wallet_balance_new.clone(),
                (amount, updated_at_ms, Option::<String>::None, address),
            )
            .await?;
        Ok(())
    }

    pub async fn register_validator(
        &self,
        validator_id: &str,
        address: &str,
        stake: i64,
        metadata: Option<String>,
    ) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        self.session
            .execute_iter(
                queries.upsert_validator.clone(),
                (validator_id, address, stake, metadata),
            )
            .await?;
        Ok(())
    }

    pub async fn get_validator(&self, validator_id: &str) -> Result<Option<Validator>> {
        let queries = self.prepared_queries.read().await;
        let rs = self
            .session
            .execute_unpaged(&queries.get_validator, (validator_id,))
            .await?;
        let rows_res = rs.into_rows_result()?;
        let mut it = rows_res.rows::<(String, String, i64, Option<String>)>()?;
        if let Some(row) = it.next() {
            let (vid, address, stake, metadata) = row?;
            return Ok(Some(Validator {
                validator_id: vid,
                address,
                stake,
                metadata,
            }));
        }
        Ok(None)
    }

    pub async fn update_validator_stake(&self, validator_id: &str, stake: i64) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        self.session
            .execute_iter(
                queries.update_validator_stake.clone(),
                (stake, validator_id),
            )
            .await?;
        Ok(())
    }

    pub async fn insert_proposal(
        &self,
        proposal_id: &str,
        description: &str,
        _chain_name: &str,
        _interop_chain: &str,
        _votes_for: i64,
        _votes_against: i64,
        _quorum: f64,
    ) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        let now_ms: i64 = chrono::Utc::now().timestamp_millis();
        self.session
            .execute_iter(
                queries.insert_proposal_new.clone(),
                (proposal_id, description, description, now_ms, "open"),
            )
            .await?;
        Ok(())
    }

    pub async fn create_proposal(
        &self,
        proposal_id: &str,
        title: &str,
        description: &str,
        created_at_ms: i64,
    ) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        self.session
            .execute_iter(
                queries.insert_proposal_new.clone(),
                (proposal_id, title, description, created_at_ms, "open"),
            )
            .await?;
        Ok(())
    }

    pub async fn get_proposal(&self, proposal_id: &str) -> Result<Option<Proposal>> {
        let queries = self.prepared_queries.read().await;
        let rs = self
            .session
            .execute_unpaged(&queries.get_proposal, (proposal_id,))
            .await?;
        let rows_res = rs.into_rows_result()?;
        let mut it = rows_res.rows::<(String, String, String, i64, String)>()?;
        if let Some(row) = it.next() {
            let (pid, title, description, created_at, status) = row?;
            return Ok(Some(Proposal {
                proposal_id: pid,
                title,
                description,
                created_at,
                status,
            }));
        }
        Ok(None)
    }

    pub async fn insert_vote(
        &self,
        proposal_id: &str,
        validator_id: &str,
        vote: bool,
        _stake_weight: i64,
        _sig: &[u8],
        timestamp_ms: i64,
    ) -> Result<()> {
        self.cast_vote(proposal_id, validator_id, vote, timestamp_ms)
            .await
    }

    pub async fn cast_vote(
        &self,
        proposal_id: &str,
        validator_id: &str,
        vote: bool,
        timestamp_ms: i64,
    ) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        let ts = if timestamp_ms > 0 {
            timestamp_ms
        } else {
            chrono::Utc::now().timestamp_millis()
        };
        self.session
            .execute_iter(
                queries.insert_vote_new.clone(),
                (proposal_id, validator_id, vote, ts),
            )
            .await?;
        Ok(())
    }

    pub async fn tally_votes(&self, proposal_id: &str) -> Result<VoteTally> {
        let queries = self.prepared_queries.read().await;
        let rs = self
            .session
            .execute_unpaged(&queries.list_votes_for_proposal, (proposal_id,))
            .await?;
        let rows_res = rs.into_rows_result()?;
        let mut it = rows_res.rows::<(bool,)>()?;
        let (mut for_count, mut against_count) = (0_i64, 0_i64);
        while let Some(row) = it.next() {
            let (v,) = row?;
            if v {
                for_count += 1
            } else {
                against_count += 1
            }
        }
        Ok(VoteTally {
            for_count,
            against_count,
        })
    }

    pub async fn set_supply(&self, name: &str, total: i64) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        self.session
            .execute_iter(queries.set_token_supply.clone(), (total, name))
            .await?;
        Ok(())
    }

    pub async fn get_supply(&self, name: &str) -> Result<Option<i64>> {
        let queries = self.prepared_queries.read().await;
        let rs = self
            .session
            .execute_unpaged(&queries.get_token_supply, (name,))
            .await?;
        let rows_res = rs.into_rows_result()?;
        let mut it = rows_res.rows::<(i64,)>()?;
        if let Some(row) = it.next() {
            let (total,) = row?;
            return Ok(Some(total));
        }
        Ok(None)
    }

    pub async fn insert_transactions_batch(
        &self,
        _shard_id: i32,
        transactions: Vec<Transaction>,
    ) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        let mut batch = Batch::new(BatchType::Logged);
        batch.set_consistency(Consistency::LocalQuorum);

        let mut values = Vec::new();
        for tx in transactions {
            batch.append_statement(queries.insert_transfer.clone());
            let status = "pending".to_string();
            let last_error: Option<String> = None;
            let ts_ms = chrono::Utc::now().timestamp_millis();
            values.push((
                tx.tx_hash.clone(),
                tx.from_address.clone(),
                tx.to_address.clone(),
                tx.amount as i64,
                status,
                ts_ms,
                ts_ms,
                last_error,
            ));
        }
        self.session.batch(&batch, &values).await?;
        Ok(())
    }
}

pub struct DataMigrator {
    rocks_db: Arc<DB>,
    scylla_db: Arc<ScyllaCluster>,
}

impl DataMigrator {
    pub fn new(rocks_db: Arc<DB>, scylla_db: Arc<ScyllaCluster>) -> Self {
        Self {
            rocks_db,
            scylla_db,
        }
    }

    pub async fn migrate_blocks(
        &self,
        shard_id: i32,
        start_height: u64,
        end_height: u64,
    ) -> Result<()> {
        info!(
            "Migrating blocks {} to {} for shard {}",
            start_height, end_height, shard_id
        );
        for height in start_height..=end_height {
            if let Some(block_bytes) = self.rocks_db.get(height.to_be_bytes())? {
                let block: Block =
                    bincode::decode_from_slice(&block_bytes, bincode::config::standard())
                        .unwrap()
                        .0;
                self.scylla_db.insert_block(shard_id, &block).await?;
                if height % 1000 == 0 {
                    info!("Migrated {} blocks", height);
                }
            }
        }
        info!("Migration complete for shard {}", shard_id);
        Ok(())
    }
}
