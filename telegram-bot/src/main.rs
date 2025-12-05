//! Sultan Telegram Bot - One-Tap Staking & Gas-Free Transactions
//!
//! Features:
//! - Instant wallet creation in Telegram
//! - One-tap staking (5000 SLTN minimum)
//! - Gas-free transactions
//! - Real-time balance/rewards tracking
//! - Push notifications for validator events

use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use reqwest::Client;

const BOT_TOKEN: &str = env!("TELEGRAM_BOT_TOKEN");
const API_ENDPOINT: &str = "https://api.sultan.finance";
const MIN_STAKE: u64 = 5_000_000_000_000; // 5,000 SLTN

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserWallet {
    telegram_id: i64,
    address: String,
    balance: u64,
    staked: u64,
    rewards: u64,
    is_validator: bool,
}

struct BotState {
    wallets: Arc<RwLock<std::collections::HashMap<i64, UserWallet>>>,
    client: Client,
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    log::info!("🤖 Starting Sultan Telegram Bot...");

    let bot = Bot::new(BOT_TOKEN);
    let state = Arc::new(BotState {
        wallets: Arc::new(RwLock::new(std::collections::HashMap::new())),
        client: Client::new(),
    });

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
) -> ResponseResult<()> {
    let text = msg.text().unwrap_or("");
    let chat_id = msg.chat.id;
    let user_id = msg.from().map(|u| u.id.0 as i64).unwrap_or(0);

    match text {
        "/start" => {
            let welcome = format!(
                "🪙 *Welcome to Sultan Finance!*\n\n\
                The fastest blockchain with *zero gas fees*.\n\n\
                • 2-second finality\n\
                • Native ETH/SOL/TON/BTC swaps\n\
                • 26.67% staking APY\n\n\
                Choose an option below:"
            );

            let keyboard = make_keyboard(&[
                vec![("💰 Create Wallet", "create_wallet")],
                vec![("📊 Check Balance", "check_balance")],
                vec![("⚡ Stake Now", "stake")],
                vec![("🔄 Swap Tokens", "swap")],
            ]);

            bot.send_message(chat_id, welcome)
                .parse_mode(ParseMode::Markdown)
                .reply_markup(keyboard)
                .await?;
        }
        "/balance" => {
            check_balance(&bot, chat_id, user_id, &state).await?;
        }
        "/stake" => {
            stake_menu(&bot, chat_id, user_id, &state).await?;
        }
        "/rewards" => {
            check_rewards(&bot, chat_id, user_id, &state).await?;
        }
        _ => {
            bot.send_message(chat_id, "Use /start to see available commands")
                .await?;
        }
    }

    Ok(())
}

async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    state: Arc<BotState>,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().unwrap().chat.id;
    let user_id = q.from.id.0 as i64;
    let data = q.data.as_deref().unwrap_or("");

    bot.answer_callback_query(q.id).await?;

    match data {
        "create_wallet" => {
            create_wallet(&bot, chat_id, user_id, &state).await?;
        }
        "check_balance" => {
            check_balance(&bot, chat_id, user_id, &state).await?;
        }
        "stake" => {
            stake_menu(&bot, chat_id, user_id, &state).await?;
        }
        "stake_5000" => {
            stake_tokens(&bot, chat_id, user_id, 5_000_000_000_000, &state).await?;
        }
        "stake_10000" => {
            stake_tokens(&bot, chat_id, user_id, 10_000_000_000_000, &state).await?;
        }
        "stake_50000" => {
            stake_tokens(&bot, chat_id, user_id, 50_000_000_000_000, &state).await?;
        }
        "unstake" => {
            unstake_tokens(&bot, chat_id, user_id, &state).await?;
        }
        "claim_rewards" => {
            claim_rewards(&bot, chat_id, user_id, &state).await?;
        }
        "swap" => {
            swap_menu(&bot, chat_id, &state).await?;
        }
        _ => {}
    }

    Ok(())
}

async fn create_wallet(
    bot: &Bot,
    chat_id: ChatId,
    user_id: i64,
    state: &BotState,
) -> ResponseResult<()> {
    let mut wallets = state.wallets.write().await;
    
    if wallets.contains_key(&user_id) {
        bot.send_message(chat_id, "✅ You already have a wallet!")
            .await?;
        return Ok(());
    }

    // Generate wallet via API
    let response = state.client
        .post(format!("{}/wallet/create", API_ENDPOINT))
        .json(&serde_json::json!({ "telegram_id": user_id }))
        .send()
        .await
        .unwrap();

    let wallet: UserWallet = response.json().await.unwrap();
    
    let message = format!(
        "✅ *Wallet Created!*\n\n\
        Address: `{}`\n\
        Balance: {} SLTN\n\n\
        ⚠️ Save this address securely!\n\n\
        *Gas-free transactions enabled* ⚡",
        wallet.address,
        wallet.balance as f64 / 1_000_000_000_000.0
    );

    wallets.insert(user_id, wallet);

    bot.send_message(chat_id, message)
        .parse_mode(ParseMode::Markdown)
        .await?;

    Ok(())
}

async fn check_balance(
    bot: &Bot,
    chat_id: ChatId,
    user_id: i64,
    state: &BotState,
) -> ResponseResult<()> {
    let wallets = state.wallets.read().await;
    
    if let Some(wallet) = wallets.get(&user_id) {
        let message = format!(
            "💰 *Your Balance*\n\n\
            Address: `{}`\n\
            Available: *{:.2} SLTN*\n\
            Staked: *{:.2} SLTN*\n\
            Rewards: *{:.2} SLTN*\n\n\
            Validator: {}\n\
            APY: *26.67%*",
            wallet.address,
            wallet.balance as f64 / 1_000_000_000_000.0,
            wallet.staked as f64 / 1_000_000_000_000.0,
            wallet.rewards as f64 / 1_000_000_000_000.0,
            if wallet.is_validator { "✅ Active" } else { "❌ Inactive" }
        );

        let keyboard = make_keyboard(&[
            vec![("⚡ Stake", "stake"), ("💸 Claim Rewards", "claim_rewards")],
            vec![("🔄 Swap", "swap"), ("« Back", "start")],
        ]);

        bot.send_message(chat_id, message)
            .parse_mode(ParseMode::Markdown)
            .reply_markup(keyboard)
            .await?;
    } else {
        bot.send_message(chat_id, "❌ No wallet found. Use /start to create one.")
            .await?;
    }

    Ok(())
}

async fn stake_menu(
    bot: &Bot,
    chat_id: ChatId,
    user_id: i64,
    state: &BotState,
) -> ResponseResult<()> {
    let wallets = state.wallets.read().await;
    
    if let Some(wallet) = wallets.get(&user_id) {
        let message = format!(
            "⚡ *One-Tap Staking*\n\n\
            Available: *{:.2} SLTN*\n\
            Minimum: *5,000 SLTN*\n\
            APY: *26.67%*\n\n\
            Choose amount to stake:",
            wallet.balance as f64 / 1_000_000_000_000.0
        );

        let keyboard = make_keyboard(&[
            vec![("5,000 SLTN", "stake_5000")],
            vec![("10,000 SLTN", "stake_10000")],
            vec![("50,000 SLTN", "stake_50000")],
            vec![("🔓 Unstake All", "unstake"), ("« Back", "start")],
        ]);

        bot.send_message(chat_id, message)
            .parse_mode(ParseMode::Markdown)
            .reply_markup(keyboard)
            .await?;
    } else {
        bot.send_message(chat_id, "❌ Create a wallet first with /start")
            .await?;
    }

    Ok(())
}

async fn stake_tokens(
    bot: &Bot,
    chat_id: ChatId,
    user_id: i64,
    amount: u64,
    state: &BotState,
) -> ResponseResult<()> {
    let mut wallets = state.wallets.write().await;
    
    if let Some(wallet) = wallets.get_mut(&user_id) {
        if wallet.balance < amount {
            bot.send_message(chat_id, "❌ Insufficient balance")
                .await?;
            return Ok(());
        }

        // Call staking API (gas-free)
        let response = state.client
            .post(format!("{}/staking/stake", API_ENDPOINT))
            .json(&serde_json::json!({
                "address": wallet.address,
                "amount": amount
            }))
            .send()
            .await
            .unwrap();

        if response.status().is_success() {
            wallet.balance -= amount;
            wallet.staked += amount;
            wallet.is_validator = wallet.staked >= MIN_STAKE;

            let message = format!(
                "✅ *Staking Successful!*\n\n\
                Staked: *{:.2} SLTN*\n\
                Total Staked: *{:.2} SLTN*\n\
                Validator: {}\n\n\
                *Zero gas fees* ⚡\n\
                Earning *26.67% APY* 🚀",
                amount as f64 / 1_000_000_000_000.0,
                wallet.staked as f64 / 1_000_000_000_000.0,
                if wallet.is_validator { "✅ Active" } else { "❌ Inactive" }
            );

            bot.send_message(chat_id, message)
                .parse_mode(ParseMode::Markdown)
                .await?;
        }
    }

    Ok(())
}

async fn unstake_tokens(
    bot: &Bot,
    chat_id: ChatId,
    user_id: i64,
    state: &BotState,
) -> ResponseResult<()> {
    let mut wallets = state.wallets.write().await;
    
    if let Some(wallet) = wallets.get_mut(&user_id) {
        if wallet.staked == 0 {
            bot.send_message(chat_id, "❌ No staked tokens")
                .await?;
            return Ok(());
        }

        // Call unstaking API
        let response = state.client
            .post(format!("{}/staking/unstake", API_ENDPOINT))
            .json(&serde_json::json!({ "address": wallet.address }))
            .send()
            .await
            .unwrap();

        if response.status().is_success() {
            let amount = wallet.staked;
            wallet.balance += amount;
            wallet.staked = 0;
            wallet.is_validator = false;

            let message = format!(
                "✅ *Unstaking Successful!*\n\n\
                Returned: *{:.2} SLTN*\n\
                Available Balance: *{:.2} SLTN*\n\n\
                You can restake anytime!",
                amount as f64 / 1_000_000_000_000.0,
                wallet.balance as f64 / 1_000_000_000_000.0
            );

            bot.send_message(chat_id, message)
                .parse_mode(ParseMode::Markdown)
                .await?;
        }
    }

    Ok(())
}

async fn claim_rewards(
    bot: &Bot,
    chat_id: ChatId,
    user_id: i64,
    state: &BotState,
) -> ResponseResult<()> {
    let mut wallets = state.wallets.write().await;
    
    if let Some(wallet) = wallets.get_mut(&user_id) {
        if wallet.rewards == 0 {
            bot.send_message(chat_id, "❌ No rewards to claim yet")
                .await?;
            return Ok(());
        }

        // Call rewards API
        let response = state.client
            .post(format!("{}/staking/claim", API_ENDPOINT))
            .json(&serde_json::json!({ "address": wallet.address }))
            .send()
            .await
            .unwrap();

        if response.status().is_success() {
            let rewards = wallet.rewards;
            wallet.balance += rewards;
            wallet.rewards = 0;

            let message = format!(
                "💸 *Rewards Claimed!*\n\n\
                Claimed: *{:.2} SLTN*\n\
                New Balance: *{:.2} SLTN*\n\n\
                *Zero gas fees* ⚡",
                rewards as f64 / 1_000_000_000_000.0,
                wallet.balance as f64 / 1_000_000_000_000.0
            );

            bot.send_message(chat_id, message)
                .parse_mode(ParseMode::Markdown)
                .await?;
        }
    }

    Ok(())
}

async fn swap_menu(
    bot: &Bot,
    chat_id: ChatId,
    state: &BotState,
) -> ResponseResult<()> {
    let message = "🔄 *Token Swaps*\n\n\
        Swap between:\n\
        • ETH ↔ SLTN (<3s)\n\
        • SOL ↔ SLTN (<3s)\n\
        • TON ↔ SLTN (<3s)\n\
        • BTC ↔ SLTN (<3s)\n\n\
        *Zero gas fees* ⚡\n\n\
        Coming soon in app!";

    bot.send_message(chat_id, message)
        .parse_mode(ParseMode::Markdown)
        .await?;

    Ok(())
}

async fn check_rewards(
    bot: &Bot,
    chat_id: ChatId,
    user_id: i64,
    state: &BotState,
) -> ResponseResult<()> {
    let wallets = state.wallets.read().await;
    
    if let Some(wallet) = wallets.get(&user_id) {
        let daily_rewards = (wallet.staked as f64 * 0.2667) / 365.0 / 1_000_000_000_000.0;
        
        let message = format!(
            "📊 *Staking Rewards*\n\n\
            Staked: *{:.2} SLTN*\n\
            Pending: *{:.2} SLTN*\n\n\
            Daily: ~{:.2} SLTN\n\
            Monthly: ~{:.2} SLTN\n\
            Yearly: ~{:.2} SLTN\n\n\
            APY: *26.67%*",
            wallet.staked as f64 / 1_000_000_000_000.0,
            wallet.rewards as f64 / 1_000_000_000_000.0,
            daily_rewards,
            daily_rewards * 30.0,
            daily_rewards * 365.0
        );

        bot.send_message(chat_id, message)
            .parse_mode(ParseMode::Markdown)
            .await?;
    }

    Ok(())
}

fn make_keyboard(buttons: &[Vec<(&str, &str)>]) -> InlineKeyboardMarkup {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = buttons
        .iter()
        .map(|row| {
            row.iter()
                .map(|(text, data)| InlineKeyboardButton::callback(text.to_string(), data.to_string()))
                .collect()
        })
        .collect();

    InlineKeyboardMarkup::new(keyboard)
}
