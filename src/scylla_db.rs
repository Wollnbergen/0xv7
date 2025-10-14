// scylla_db.rs - ScyllaDB integration for sharding/TPS

use anyhow::{Result, Context, anyhow};
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use scylla::statement::Consistency;
use scylla::prepared_statement::PreparedStatement;
use scylla::statement::batch::{Batch, BatchType};
use scylla::macros::FromRow;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use std::collections::HashMap;
use rocksdb::DB;
use crate::quantum::QuantumCrypto;
use crate::lib::{Block, Transaction};

pub struct ScyllaCluster {
    pub session: Arc<Session>,
    pub keyspace: String,
    pub prepared_queries: Arc<RwLock<PreparedQueries>>,
}

#[derive(FromRow)]
pub struct BlockData {
    pub height: i64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub validator: String,
    pub state_root: Vec<u8>,
}

pub struct PreparedQueries {
    pub insert_block: PreparedStatement,
    pub get_block: PreparedStatement,
    pub get_latest_block: PreparedStatement,
    pub insert_transaction: PreparedStatement,
    pub get_validator: PreparedStatement,
    pub update_shard_state: PreparedStatement,
    pub update_balance: PreparedStatement,
    pub get_balance: PreparedStatement,
    // Added for production wallet/token/governance support:
    pub insert_wallet: PreparedStatement,
    pub update_wallet_balance: PreparedStatement,
    pub insert_proposal: PreparedStatement,
    pub insert_vote: PreparedStatement,
}

impl ScyllaCluster {
    pub async fn new(nodes: Vec<&str>, keyspace: &str) -> Result<Self> {
        let session: Arc<Session> = Arc::new(SessionBuilder::new()
            .known_nodes(nodes)
            .build()
            .await
            .context("Failed to connect to ScyllaDB")?);
        session.query_iter(
            format!(
                "CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {{'class': 'NetworkTopologyStrategy', 'replication_factor': 3}}",
                keyspace
            ),
            &[]
        ).await.context("Failed to create keyspace")?;
        session.use_keyspace(keyspace, false).await.context("Failed to use keyspace")?;
        Self::create_tables(&session).await.context("Failed to create tables")?;
        let prepared_queries = Self::prepare_queries(&session).await.context("Failed to prepare queries")?;
        Ok(Self {
            session,
            keyspace: keyspace.to_string(),
            prepared_queries: Arc::new(RwLock::new(prepared_queries)),
        })
    }

    async fn create_tables(session: &Session) -> Result<()> {
        session.query_iter(
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
            &[]
        ).await.context("Failed to create blocks table")?;
        session.query_iter(
            "CREATE TABLE IF NOT EXISTS wallets (
                telegram_id TEXT PRIMARY KEY,
                address TEXT,
                pk BLOB,
                sk BLOB,
                balance BIGINT,
                created_at BIGINT
            )",
            &[]
        ).await.context("Failed to create wallets table")?;
        session.query_iter(
            "CREATE TABLE IF NOT EXISTS proposals (
                proposal_id TEXT PRIMARY KEY,
                description TEXT,
                chain_name TEXT,
                interop_chain TEXT,
                votes_for BIGINT,
                votes_against BIGINT,
                quorum DOUBLE
            )",
            &[]
        ).await.context("Failed to create proposals table")?;
        session.query_iter(
            "CREATE TABLE IF NOT EXISTS votes (
                proposal_id TEXT,
                validator_id TEXT,
                vote BOOL,
                stake_weight BIGINT,
                sig BLOB,
                timestamp BIGINT,
                PRIMARY KEY (proposal_id, validator_id)
            )",
            &[]
        ).await.context("Failed to create votes table")?;
        session.query_iter(
            "CREATE TABLE IF NOT EXISTS accounts (
                address TEXT,
                shard_id INT,
                balance BIGINT,
                PRIMARY KEY (address, shard_id)
            )",
            &[]
        ).await.context("Failed to create accounts table")?;
        Ok(())
    }

    async fn prepare_queries(session: &Session) -> Result<PreparedQueries> {
        Ok(PreparedQueries {
            insert_block: session.prepare(
                "INSERT INTO blocks (shard_id, height, hash, previous_hash, timestamp, validator, signature, state_root, transactions)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            ).await.context("Failed to prepare insert_block")?,
            get_block: session.prepare("SELECT * FROM blocks WHERE shard_id = ? AND height = ?").await.context("Failed to prepare get_block")?,
            get_latest_block: session.prepare("SELECT * FROM blocks WHERE shard_id = ? ORDER BY height DESC LIMIT 1").await.context("Failed to prepare get_latest_block")?,
            insert_transaction: session.prepare(
                "INSERT INTO transactions (shard_id, tx_hash, block_height, from_address, to_address, amount, nonce, signature, timestamp, status)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            ).await.context("Failed to prepare insert_transaction")?,
            get_validator: session.prepare("SELECT * FROM validators WHERE address = ?").await.context("Failed to prepare get_validator")?,
            update_shard_state: session.prepare("UPDATE shards SET state_root = ? WHERE shard_id = ?").await.context("Failed to prepare update_shard_state")?,
            update_balance: session.prepare("UPDATE accounts SET balance = ? WHERE address = ? AND shard_id = ?").await.context("Failed to prepare update_balance")?,
            get_balance: session.prepare("SELECT balance FROM accounts WHERE address = ? AND shard_id = ?").await.context("Failed to prepare get_balance")?,
            // New for wallet/token/governance:
            insert_wallet: session.prepare(
                "INSERT INTO wallets (telegram_id, address, pk, sk, balance, created_at) VALUES (?, ?, ?, ?, ?, ?)"
            ).await.context("Failed to prepare insert_wallet")?,
            update_wallet_balance: session.prepare(
                "UPDATE wallets SET balance = balance + ? WHERE address = ?"
            ).await.context("Failed to prepare update_wallet_balance")?,
            insert_proposal: session.prepare(
                "INSERT INTO proposals (proposal_id, description, chain_name, interop_chain, votes_for, votes_against, quorum) VALUES (?, ?, ?, ?, ?, ?, ?)"
            ).await.context("Failed to prepare insert_proposal")?,
            insert_vote: session.prepare(
                "INSERT INTO votes (proposal_id, validator_id, vote, stake_weight, sig, timestamp) VALUES (?, ?, ?, ?, ?, ?)"
            ).await.context("Failed to prepare insert_vote")?,
        })
    }

    pub async fn insert_block(&self, shard_id: i32, block: &Block) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        self.session.execute_iter(
            &queries.insert_block,
            (shard_id, block.height as i64, &block.hash, &block.previous_hash, block.timestamp.timestamp(), &block.validator, &block.signature, &block.state_root, &block.transactions),
        ).await.context("Failed to insert block")?;
        Ok(())
    }

    pub async fn insert_wallet(&self, telegram_id: &str, address: &str, pk: &[u8], sk: &[u8], created_at: i64) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        self.session.execute_iter(
            &queries.insert_wallet,
            (telegram_id, address, pk, sk, 0_i64, created_at),
        ).await.context("Failed to insert wallet")?;
        Ok(())
    }

    pub async fn update_wallet_balance(&self, address: &str, amount: i64) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        self.session.execute_iter(
            &queries.update_wallet_balance,
            (amount, address),
        ).await.context("Failed to update wallet balance")?;
        Ok(())
    }

    pub async fn insert_proposal(&self, proposal_id: &str, description: &str, chain_name: &str, interop_chain: &str, votes_for: i64, votes_against: i64, quorum: f64) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        self.session.execute_iter(
            &queries.insert_proposal,
            (proposal_id, description, chain_name, interop_chain, votes_for, votes_against, quorum),
        ).await.context("Failed to insert proposal")?;
        Ok(())
    }

    pub async fn insert_vote(&self, proposal_id: &str, validator_id: &str, vote: bool, stake_weight: i64, sig: &[u8], timestamp: i64) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        self.session.execute_iter(
            &queries.insert_vote,
            (proposal_id, validator_id, vote, stake_weight, sig, timestamp),
        ).await.context("Failed to insert vote")?;
        Ok(())
    }

    pub async fn insert_transactions_batch(&self, shard_id: i32, transactions: Vec<Transaction>) -> Result<()> {
        let queries = self.prepared_queries.read().await;
        let mut batch = Batch::new(BatchType::Logged);
        batch.set_consistency(Consistency::LocalQuorum);
        let mut values = Vec::new();
        for tx in transactions {
            batch.append_statement(queries.insert_transaction.clone());
            values.push((
                shard_id,
                tx.tx_hash,
                tx.block_height as i64,
                tx.from_address,
                tx.to_address,
                tx.amount as i64,
                tx.nonce as i64,
                tx.signature,
                tx.timestamp,
                tx.status,
            ));
        }
        self.session.batch(&batch, &values).await.context("Failed to insert transactions batch")?;
        Ok(())
    }
}

// Migration helper for RocksDB -> ScyllaDB
pub struct DataMigrator {
    rocks_db: Arc<DB>,
    scylla_db: Arc<ScyllaCluster>,
}

impl DataMigrator {
    pub fn new(rocks_db: Arc<DB>, scylla_db: Arc<ScyllaCluster>) -> Self {
        Self { rocks_db, scylla_db }
    }

    pub async fn migrate_blocks(&self, shard_id: i32, start_height: u64, end_height: u64) -> Result<()> {
        info!("Migrating blocks {} to {} for shard {}", start_height, end_height, shard_id);
        for height in start_height..=end_height {
            if let Some(block_bytes) = self.rocks_db.get(height.to_be_bytes()).map_err(|e| {
                error!("Failed to read block from RocksDB: {:?}", e);
                anyhow!("Failed to read block from RocksDB")
            })? {
                let block: Block = bincode::deserialize(&block_bytes).map_err(|e| {
                    error!("Failed to deserialize block: {:?}", e);
                    anyhow!("Failed to deserialize block")
                })?;
                if let Err(e) = self.scylla_db.insert_block(shard_id, &block).await {
                    error!("Failed to insert block into ScyllaDB: {:?}", e);
                }
                if height % 1000 == 0 {
                    info!("Migrated {} blocks", height);
                }
            }
        }
        info!("Migration complete for shard {}", shard_id);
        Ok(())
    }
}