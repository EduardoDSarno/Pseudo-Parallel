use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum COINS{
    HYPE,
    BTC,
    ETH,
}

pub const HYPERLIQUID_WS_URL: &str = "wss://api.hyperliquid.xyz/ws";