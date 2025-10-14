//! Production Scylla-backed SDK (scylla 1.3.x).

use anyhow::{anyhow, Context, Result};
use scylla::{
    client::session::Session, client::session_builder::SessionBuilder, client::Compression,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::migrations;
use crate::ChainConfig;

pub struct SultanSDK {
    session: Session,
    chain: ChainConfig,
}

impl SultanSDK {
    pub async fn new(chain: ChainConfig, contact_point: Option<&str>) -> Result<Self> {
        let contact = contact_point.unwrap_or("127.0.0.1:9042");
        let session = SessionBuilder::new()
            .known_node(contact)
            .compression(Some(Compression::Lz4))
            .build()
            .await
            .with_context(|| format!("connect scylla at {}", contact))?;

        ensure_keyspace(&session).await?;
        migrations::run_migrations(&session).await?;

        Ok(Self { session, chain })
    }

    pub async fn create_wallet(&self, telegram_id: &str) -> Result<String> {
        let addr = derive_address_from_telegram(telegram_id);
        let now = now_ms();

        self.session
            .query_unpaged(
                "INSERT INTO sultan.wallets (address, balance, created_at, updated_at, last_update_tx)
                 VALUES (?, 0, ?, ?, ?) IF NOT EXISTS",
                (addr.clone(), now, now, format!("create:{telegram_id}")),
            )
            .await
            .context("insert wallet")?;

        Ok(addr)
    }

    pub async fn get_balance(&self, address: &str) -> Result<u64> {
        let res = self
            .session
            .query_unpaged(
                "SELECT balance FROM sultan.wallets WHERE address = ?",
                (address.to_string(),),
            )
            .await
            .context("select balance")?;

        let rows_res = res.into_rows_result().context("balance rows_result")?;
        let mut it = rows_res.rows::<(i64,)>().context("balance rows()")?;
        if let Some(row) = it.next() {
            let (bal,) = row.context("row decode")?;
            return Ok(u64::try_from(bal).unwrap_or(0));
        }
        Ok(0)
    }

    pub async fn mint_token(&self, to: &str, amount: u64) -> Result<String> {
        let _ = self.create_wallet_if_missing(to).await;

        let mut attempts = 0usize;
        loop {
            attempts += 1;
            let current = self.get_balance_i64(to).await?;
            let new = current.saturating_add(amount as i64);
            let now = now_ms();

            let res = self
                .session
                .query_unpaged(
                    "UPDATE sultan.wallets
                     SET balance = ?, updated_at = ?, last_update_tx = ?
                     WHERE address = ? IF balance = ?",
                    (new, now, format!("mint:{amount}"), to.to_string(), current),
                )
                .await
                .context("mint LWT update")?;

            let rows_res = res.into_rows_result().context("mint LWT rows_result")?;
            let mut it = rows_res.rows::<(bool,)>().context("mint rows()")?;
            if let Some(row) = it.next() {
                let (applied,) = row.context("applied decode")?;
                if applied {
                    return Ok("ok".to_string());
                }
            }
            if attempts >= 5 {
                return Err(anyhow!("mint failed due to concurrent updates"));
            }
        }
    }

    pub async fn stake(&self, validator_id: &str, amount: u64) -> Result<bool> {
        let now = now_ms();
        let tx_id = format!("stake:{validator_id}:{amount}:{now}");
        self.session
            .query_unpaged(
                "INSERT INTO sultan.transfers (tx_id, from_address, to_address, amount, status, created_at, updated_at, last_error)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (tx_id, "", validator_id, amount as i64, "staked", now, now, ""),
            )
            .await
            .context("record stake")?;
        Ok(true)
    }

    pub async fn query_apy(&self, is_validator: bool) -> Result<f64> {
        let base = self.chain.inflation_rate / 100.0;
        Ok(if is_validator { base + 0.04 } else { base })
    }

    pub async fn vote_on_proposal(
        &self,
        proposal_id: &str,
        vote: bool,
        validator_id: &str,
    ) -> Result<bool> {
        let now = now_ms();
        let tx_id = format!("vote:{proposal_id}:{vote}:{now}");
        self.session
            .query_unpaged(
                "INSERT INTO sultan.transfers (tx_id, from_address, to_address, amount, status, created_at, updated_at, last_error)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    tx_id,
                    validator_id,
                    proposal_id,
                    0i64,
                    if vote { "vote_yes" } else { "vote_no" },
                    now,
                    now,
                    "",
                ),
            )
            .await
            .context("record vote")?;
        Ok(true)
    }

    pub async fn cross_chain_swap(&self, from: &str, amount: u64) -> Result<bool> {
        let now = now_ms();
        let tx_id = format!("xswap:{amount}:{now}");
        self.session
            .query_unpaged(
                "INSERT INTO sultan.transfers (tx_id, from_address, to_address, amount, status, created_at, updated_at, last_error)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (tx_id, from, "bridge", amount as i64, "initiated", now, now, ""),
            )
            .await
            .context("record cross_chain_swap")?;
        Ok(true)
    }

    // Helpers

    async fn create_wallet_if_missing(&self, address: &str) -> Result<()> {
        let now = now_ms();
        self.session
            .query_unpaged(
                "INSERT INTO sultan.wallets (address, balance, created_at, updated_at, last_update_tx)
                 VALUES (?, 0, ?, ?, ?) IF NOT EXISTS",
                (address.to_string(), now, now, "auto-create"),
            )
            .await
            .context("ensure wallet")?;
        Ok(())
    }

    async fn get_balance_i64(&self, address: &str) -> Result<i64> {
        let res = self
            .session
            .query_unpaged(
                "SELECT balance FROM sultan.wallets WHERE address = ?",
                (address.to_string(),),
            )
            .await
            .context("select balance i64")?;

        let rows_res = res.into_rows_result().context("balance i64 rows_result")?;
        let mut it = rows_res.rows::<(i64,)>().context("balance i64 rows()")?;
        if let Some(row) = it.next() {
            let (bal,) = row.context("row decode")?;
            return Ok(bal);
        }
        Ok(0)
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn derive_address_from_telegram(telegram_id: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    telegram_id.hash(&mut hasher);
    let h = hasher.finish();
    let body = format!("{:016x}{:024x}", h, h);
    format!("0x{}", &body[..40])
}

async fn ensure_keyspace(session: &Session) -> Result<()> {
    let dev = std::env::var("SULTAN_DEV").unwrap_or_else(|_| "true".into());
    let rf: u32 = std::env::var("SULTAN_RF")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(if dev == "true" { 1 } else { 3 });

    let cql = format!(
        "CREATE KEYSPACE IF NOT EXISTS sultan WITH replication = {{ 'class': 'SimpleStrategy', 'replication_factor': {} }}",
        rf
    );
    let _ = session.query_unpaged(cql, ()).await;
    Ok(())
}
