use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let bot = sultan_coordinator::telegram_bot::TelegramBot::new();
    bot.stake(1000).await
}
