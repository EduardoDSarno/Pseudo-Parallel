use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsLevel {
    pub price: String,
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookL2 {
    pub coin: String,
    pub levels: [Vec<WsLevel>; 2], // [bids, asks]
    pub time_ms: u64,              
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    #[serde(rename = "t")]
    pub open_time_ms: u64, // open millis

    #[serde(rename = "T")]
    pub close_time_ms: u64, // close millis

    #[serde(rename = "s")]
    pub coin: String,

    #[serde(rename = "i")]
    pub interval: String,

    #[serde(rename = "o")]
    pub open_price: f64,

    #[serde(rename = "c")]
    pub close_price: f64,

    #[serde(rename = "h")]
    pub high_price: f64,

    #[serde(rename = "l")]
    pub low_price: f64,

    #[serde(rename = "v")]
    pub volume: f64,

    #[serde(rename = "n")]
    pub trade_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeSide {
    #[serde(rename = "B")]
    Buy,
    #[serde(rename = "S")]
    Sell,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub coin: String,
    pub side: TradeSide,
    pub px: String, // price as string
    pub sz: String, // size as string
    pub hash: String,
    pub time: u64,  // millis
    pub tid: u64,   // trade id
    pub users: [String; 2], // [buyer, seller]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    L2Book(OrderBookL2),
    Candle(Candle),
    Trade(Trade),
}