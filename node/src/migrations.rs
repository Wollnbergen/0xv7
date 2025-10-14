// filepath: /workspaces/0xv7/node/src/migrations.rs
use anyhow::{Context, Result};
use scylla::client::session::Session;

pub async fn run_migrations(session: &Session) -> Result<()> {
    // Wallets
    session
        .query_unpaged(
            "CREATE TABLE IF NOT EXISTS sultan.wallets (
                address text PRIMARY KEY,
                balance bigint,
                created_at bigint,
                updated_at bigint,
                last_update_tx text
            )",
            (),
        )
        .await
        .context("create table sultan.wallets")?;

    // Transfers
    session
        .query_unpaged(
            "CREATE TABLE IF NOT EXISTS sultan.transfers (
                tx_id text PRIMARY KEY,
                from_address text,
                to_address text,
                amount bigint,
                status text,
                created_at bigint,
                updated_at bigint,
                last_error text
            )",
            (),
        )
        .await
        .context("create table sultan.transfers")?;

    // Helpful indexes (best-effort)
    let _ = session
        .query_unpaged(
            "CREATE INDEX IF NOT EXISTS transfers_status_idx ON sultan.transfers (status)",
            (),
        )
        .await;
    let _ = session
        .query_unpaged(
            "CREATE INDEX IF NOT EXISTS transfers_from_idx ON sultan.transfers (from_address)",
            (),
        )
        .await;
    let _ = session
        .query_unpaged(
            "CREATE INDEX IF NOT EXISTS transfers_to_idx ON sultan.transfers (to_address)",
            (),
        )
        .await;

    Ok(())
}
