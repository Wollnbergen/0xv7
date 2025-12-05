// Wrapper to maintain compatibility with existing Sultan SDK

use cosmrs::{Any, Coin, tx::Msg};
use tendermint_rpc::Client;

pub struct SultanCosmosSDK {
    client: Client,
    chain_id: String,
}

impl SultanCosmosSDK {
    pub async fn new(endpoint: &str, chain_id: &str) -> Result<Self, Error> {
        let client = Client::new(endpoint).await?;
        Ok(Self {
            client,
            chain_id: chain_id.to_string(),
        })
    }
    
    // Wrap existing SDK methods to use Cosmos
    pub async fn create_wallet(&self) -> Result<String, Error> {
        // Generate Cosmos-compatible wallet
        let wallet = cosmrs::crypto::secp256k1::SigningKey::random();
        let address = wallet.public_key().account_id("sultan").to_string();
        Ok(address)
    }
    
    pub async fn transfer(&self, from: &str, to: &str, amount: u128) -> Result<String, Error> {
        // Create Cosmos bank send message
        let msg = cosmrs::bank::MsgSend {
            from_address: from.parse()?,
            to_address: to.parse()?,
            amount: vec![Coin {
                denom: "usultan".to_string(),
                amount: amount.into(),
            }],
        };
        
        // Sign and broadcast transaction
        let tx_hash = self.broadcast_msg(msg).await?;
        Ok(tx_hash)
    }
}
