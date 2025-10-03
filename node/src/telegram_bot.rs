#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let bot = TelegramBot::new();
    bot.stake(1000).await?;
    Ok(())
}
use anyhow::Result;
use tracing::info;

pub struct TelegramBot;

impl TelegramBot {
    pub fn new() -> Self {
        info!("Initializing Telegram bot for one-tap UX");
        Self
    }

    pub async fn stake(&self, amount: u64) -> Result<()> {
        info!("One-tap staking {} SLTN at APY ~26.67% (hide complexity)", amount);
        Ok(())
    }
}
