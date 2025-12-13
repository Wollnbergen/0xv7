use anyhow::Result;
use tracing::info;

pub struct TelegramBot;

impl TelegramBot {
    pub fn new() -> Self {
        info!("Initializing Telegram bot for one-tap UX");
        Self
    }

    pub async fn stake(&self, amount: u64) -> Result<()> {
        info!(
            "One-tap staking {} SLTN at APY ~13.33% (hide complexity)",
            amount
        );
        Ok(())
    }
}

impl Default for TelegramBot {
    fn default() -> Self {
        Self::new()
    }
}
