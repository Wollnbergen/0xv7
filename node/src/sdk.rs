//! Production Scylla-backed SDK (scylla 1.3.x).

use anyhow::{anyhow, Context, Result};
use scylla::{
    client::session::Session, client::session_builder::SessionBuilder, client::Compression,
};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{info, warn};

use crate::migrations;
use crate::ChainConfig;

pub struct SultanSDK {
    pub(crate) session: Session,
    pub(crate) chain: ChainConfig,
}

impl SultanSDK {
    pub async fn new(chain: ChainConfig, contact_point: Option<&str>) -> Result<Self> {
        let contact = contact_point.unwrap_or("127.0.0.1:9042");
        let session = connect_with_retries(contact, 30, Duration::from_secs(1)).await?;

        ensure_keyspace(&session).await?;
        migrations::run_migrations(&session).await?;
        ensure_extra_tables(&session).await?;

        Ok(Self { session, chain })
    }

    // Wallets

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

    pub async fn set_balance_abs(&self, address: &str, new_balance: u64) -> Result<()> {
        let now = now_ms();
        self.session
            .query_unpaged(
                "UPDATE sultan.wallets SET balance = ?, updated_at = ?, last_update_tx = ?
                 WHERE address = ?",
                (new_balance as i64, now, "set_balance", address.to_string()),
            )
            .await
            .context("set_balance_abs")?;
        Ok(())
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

    // Validators

    pub async fn validator_register(
        &self,
        validator_id: &str,
        address: &str,
        stake: i64,
        metadata: Option<&str>,
    ) -> Result<()> {
        self.session
            .query_unpaged(
                "INSERT INTO sultan.validators (validator_id, address, stake, metadata)
                 VALUES (?, ?, ?, ?) IF NOT EXISTS",
                (
                    validator_id.to_string(),
                    address.to_string(),
                    stake,
                    metadata.unwrap_or(""),
                ),
            )
            .await
            .context("validator_register")?;
        Ok(())
    }

    pub async fn validator_update_stake(&self, validator_id: &str, new_stake: i64) -> Result<()> {
        self.session
            .query_unpaged(
                "UPDATE sultan.validators SET stake = ? WHERE validator_id = ?",
                (new_stake, validator_id.to_string()),
            )
            .await
            .context("validator_update_stake")?;
        Ok(())
    }

    pub async fn validator_get(&self, validator_id: &str) -> Result<Option<ValidatorInfo>> {
        let res = self
            .session
            .query_unpaged(
                "SELECT validator_id, address, stake, metadata
                 FROM sultan.validators WHERE validator_id = ?",
                (validator_id.to_string(),),
            )
            .await
            .context("validator_get")?;

        let rows_res = res
            .into_rows_result()
            .context("validator_get rows_result")?;
        let mut it = rows_res
            .rows::<(String, String, i64, Option<String>)>()
            .context("validator_get rows()")?;
        if let Some(row) = it.next() {
            let (vid, addr, stake, meta) = row.context("validator row decode")?;
            return Ok(Some(ValidatorInfo {
                validator_id: vid,
                address: addr,
                stake,
                metadata: meta.unwrap_or_default(),
            }));
        }
        Ok(None)
    }

    // Proposals

    pub async fn proposal_create(
        &self,
        proposal_id: &str,
        title: Option<&str>,
        description: Option<&str>,
        status: Option<&str>,
    ) -> Result<()> {
        self.session
            .query_unpaged(
                "INSERT INTO sultan.proposals (proposal_id, title, description, created_at, status)
                 VALUES (?, ?, ?, toTimestamp(now()), ?) IF NOT EXISTS",
                (
                    proposal_id.to_string(),
                    title.unwrap_or(""),
                    description.unwrap_or(""),
                    status.unwrap_or("open"),
                ),
            )
            .await
            .context("proposal_create")?;
        Ok(())
    }

    pub async fn proposal_get(&self, proposal_id: &str) -> Result<Option<ProposalInfo>> {
        let res = self
            .session
            .query_unpaged(
                "SELECT proposal_id, title, description, toUnixTimestamp(created_at), status
                 FROM sultan.proposals WHERE proposal_id = ?",
                (proposal_id.to_string(),),
            )
            .await
            .context("proposal_get")?;

        let rows_res = res.into_rows_result().context("proposal_get rows_result")?;
        let mut it = rows_res
            .rows::<(
                String,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<String>,
            )>()
            .context("proposal_get rows()")?;
        if let Some(row) = it.next() {
            let (pid, title, desc, created_at_ms, status) = row.context("proposal row decode")?;
            return Ok(Some(ProposalInfo {
                proposal_id: pid,
                title,
                description: desc,
                created_at_ms,
                status,
            }));
        }
        Ok(None)
    }

    // Votes

    pub async fn vote_cast(
        &self,
        proposal_id: &str,
        validator_id: &str,
        vote_yes: bool,
    ) -> Result<()> {
        self.session
            .query_unpaged(
                "INSERT INTO sultan.votes (proposal_id, validator_id, vote, ts)
                 VALUES (?, ?, ?, toTimestamp(now()))",
                (proposal_id.to_string(), validator_id.to_string(), vote_yes),
            )
            .await
            .context("vote_cast")?;
        Ok(())
    }

    pub async fn votes_tally(&self, proposal_id: &str) -> Result<(i64, i64)> {
        let res = self
            .session
            .query_unpaged(
                "SELECT vote FROM sultan.votes WHERE proposal_id = ?",
                (proposal_id.to_string(),),
            )
            .await
            .context("votes_tally")?;

        let rows_res = res.into_rows_result().context("votes_tally rows_result")?;
        let mut it = rows_res.rows::<(bool,)>().context("votes_tally rows()")?;
        let (mut yes, mut no) = (0_i64, 0_i64);
        while let Some(row) = it.next() {
            let (v,) = row.context("vote decode")?;
            if v {
                yes += 1;
            } else {
                no += 1;
            }
        }
        Ok((yes, no))
    }

    // Token supply

    pub async fn supply_get(&self, name: &str) -> Result<Option<i64>> {
        let res = self
            .session
            .query_unpaged(
                "SELECT total FROM sultan.token_supply WHERE name = ?",
                (name.to_string(),),
            )
            .await
            .context("supply_get")?;
        let rows_res = res.into_rows_result().context("supply_get rows_result")?;
        let mut it = rows_res.rows::<(i64,)>().context("supply_get rows()")?;
        if let Some(row) = it.next() {
            let (total,) = row.context("supply row decode")?;
            return Ok(Some(total));
        }
        Ok(None)
    }

    pub async fn supply_set(&self, name: &str, total: i64) -> Result<()> {
        self.session
            .query_unpaged(
                "INSERT INTO sultan.token_supply (name, total) VALUES (?, ?)",
                (name.to_string(), total),
            )
            .await
            .context("supply_set")?;
        Ok(())
    }

    // Staking, APY, governance placeholders (recorded to transfers)

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

// Types

#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    pub validator_id: String,
    pub address: String,
    pub stake: i64,
    pub metadata: String,
}

#[derive(Debug, Clone)]
pub struct ProposalInfo {
    pub proposal_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created_at_ms: Option<i64>,
    pub status: Option<String>,
}

// Utils

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

async fn ensure_extra_tables(session: &Session) -> Result<()> {
    let stmts = [
        "CREATE TABLE IF NOT EXISTS sultan.validators (validator_id text PRIMARY KEY, address text, stake bigint, metadata text)",
        "CREATE TABLE IF NOT EXISTS sultan.proposals (proposal_id text PRIMARY KEY, title text, description text, created_at timestamp, status text)",
        "CREATE TABLE IF NOT EXISTS sultan.votes (proposal_id text, validator_id text, vote boolean, ts timestamp, PRIMARY KEY (proposal_id, validator_id))",
        "CREATE TABLE IF NOT EXISTS sultan.token_supply (name text PRIMARY KEY, total bigint)",
    ];
    for cql in stmts {
        let _ = session.query_unpaged(cql, ()).await;
    }
    Ok(())
}

async fn connect_with_retries(contact: &str, attempts: u32, delay: Duration) -> Result<Session> {
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=attempts {
        match SessionBuilder::new()
            .known_node(contact)
            .compression(Some(Compression::Lz4))
            .build()
            .await
        {
            Ok(s) => {
                info!("Connected to Scylla at {} (attempt {attempt})", contact);
                return Ok(s);
            }
            Err(e) => {
                last_err = Some(
                    anyhow!(e)
                        .context(format!("connect scylla at {} (attempt {attempt})", contact)),
                );
                if attempt < attempts {
                    warn!(
                        "Scylla not ready yet (attempt {attempt}/{attempts}); retrying in {}s",
                        delay.as_secs()
                    );
                    sleep(delay).await;
                }
            }
        }
    }
    Err(last_err
        .unwrap_or_else(|| anyhow!("Scylla connect failed without error"))
        .context("exhausted Scylla connect retries"))
}
