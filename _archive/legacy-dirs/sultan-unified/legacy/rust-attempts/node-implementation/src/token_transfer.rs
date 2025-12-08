//! Token Transfer Module

use crate::database::{Database, Transfer};
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Clone)]
pub struct TransferManager {
    db: Arc<Mutex<Database>>,
}

impl TransferManager {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    pub fn transfer(
        &self,
        from: &str,
        to: &str,
        amount: u64,
        memo: Option<String>,
    ) -> Result<Transfer> {
        if amount == 0 {
            return Err(anyhow!("Transfer amount must be greater than 0"));
        }

        if from == to {
            return Err(anyhow!("Cannot transfer to same address"));
        }

        let mut db = self.db.lock().unwrap();

        // Check sender balance
        let sender_balance = db.wallets.get(from)
            .map(|w| w.balance)
            .unwrap_or(0);

        if sender_balance < amount {
            return Err(anyhow!(
                "Insufficient balance: have {}, need {}", 
                sender_balance, amount
            ));
        }

        // Create recipient wallet if needed
        if !db.wallets.contains_key(to) {
            db.create_wallet(to)?;
        }

        // Execute transfer
        if let Some(sender_wallet) = db.wallets.get_mut(from) {
            sender_wallet.balance -= amount;
        }

        let to_address = if to.starts_with("sultan1") {
            to.to_string()
        } else {
            format!("sultan1{}", to)
        };

        if let Some(recipient_wallet) = db.wallets.get_mut(&to_address) {
            recipient_wallet.balance += amount;
        }

        // Record transfer
        let transfer = Transfer {
            id: format!("tx_{}", uuid::Uuid::new_v4()),
            from: from.to_string(),
            to: to_address.clone(),
            amount,
            fee: 0,
            timestamp: chrono::Utc::now().timestamp(),
            memo,
        };

        db.transfers.insert(transfer.id.clone(), transfer.clone());

        info!("Transfer: {} -> {} amount: {}", from, to_address, amount);
        Ok(transfer)
    }
}
