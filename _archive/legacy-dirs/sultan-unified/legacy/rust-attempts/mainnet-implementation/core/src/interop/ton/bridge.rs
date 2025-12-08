use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonBridge {
    pub smart_contract: String,
    pub workchain: i32,
    pub supported_tokens: Vec<String>,
}

impl TonBridge {
    pub fn new() -> Self {
        TonBridge {
            smart_contract: "EQSultan...".to_string(),
            workchain: 0,
            supported_tokens: vec![
                "TON".to_string(),
                "USDT".to_string(),
            ],
        }
    }
    
    pub async fn bridge_from_ton(&self, msg_id: &str, amount: u64) -> Result<String, String> {
        Ok(format!("Bridged {} TON with zero fees", amount))
    }
}
