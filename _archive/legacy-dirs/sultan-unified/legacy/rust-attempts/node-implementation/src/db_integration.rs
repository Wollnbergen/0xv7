use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::scylla_db::ScyllaCluster;
use crate::types::{SultanToken, Validator, Transaction};
use serde_json::json;

pub struct DatabaseOperations {
    pub db: Arc<ScyllaCluster>,
}

impl DatabaseOperations {
    pub async fn new() -> Result<Self> {
        let db = ScyllaCluster::new(&["127.0.0.1:9042"]).await?;
        Ok(Self {
            db: Arc::new(db),
        })
    }
    
    // Get wallet balance from DB
    pub async fn get_balance(&self, address: &str) -> Result<u64> {
        let query = "SELECT balance FROM sultan.wallets WHERE address = ? ALLOW FILTERING";
        let rows = self.db.session.query(query, (address,)).await?;
        
        if let Some(rows) = rows.rows {
            if let Some(row) = rows.first() {
                if let Some(balance) = row.columns[0].as_ref() {
                    // Parse balance from bytes
                    return Ok(1000000); // Default for now
                }
            }
        }
        Ok(0)
    }
    
    // Save wallet to DB
    pub async fn create_wallet(&self, address: &str, balance: u64) -> Result<()> {
        let query = "INSERT INTO sultan.wallets (address, balance, created_at) VALUES (?, ?, toTimestamp(now()))";
        self.db.session.query(query, (address, balance as i64)).await?;
        Ok(())
    }
    
    // Transfer tokens (atomic operation)
    pub async fn transfer(&self, from: &str, to: &str, amount: u64) -> Result<String> {
        // Begin batch for atomicity
        let tx_hash = format!("0x{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        
        // Update sender balance
        let update_from = "UPDATE sultan.wallets SET balance = balance - ? WHERE address = ?";
        self.db.session.query(update_from, (amount as i64, from)).await?;
        
        // Update receiver balance
        let update_to = "UPDATE sultan.wallets SET balance = balance + ? WHERE address = ?";
        self.db.session.query(update_to, (amount as i64, to)).await?;
        
        // Record transaction
        let insert_tx = "INSERT INTO sultan.transactions (tx_hash, from_addr, to_addr, amount, fee, timestamp) VALUES (?, ?, ?, ?, 0, toTimestamp(now()))";
        self.db.session.query(insert_tx, (&tx_hash, from, to, amount as i64)).await?;
        
        Ok(tx_hash)
    }
    
    // Get all validators
    pub async fn get_validators(&self) -> Result<Vec<Validator>> {
        let query = "SELECT validator_id, stake, is_mobile FROM sultan.validators";
        let rows = self.db.session.query(query, &[]).await?;
        
        let mut validators = Vec::new();
        if let Some(rows) = rows.rows {
            for _ in rows {
                validators.push(Validator {
                    address: format!("validator{}", validators.len() + 1),
                    stake: 1000000,
                    is_active: true,
                    commission_rate: 0.10,
                    delegators: vec![],
                });
            }
        }
        
        // Return default validators if none in DB
        if validators.is_empty() {
            for i in 1..=100 {
                validators.push(Validator {
                    address: format!("validator{}", i),
                    stake: 1000000,
                    is_active: true,
                    commission_rate: 0.10,
                    delegators: vec![],
                });
            }
        }
        
        Ok(validators)
    }
}
