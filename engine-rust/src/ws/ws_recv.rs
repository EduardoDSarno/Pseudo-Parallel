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
    pub open_price: String,

    #[serde(rename = "c")]
    pub close_price: String,

    #[serde(rename = "h")]
    pub high_price: String,

    #[serde(rename = "l")]
    pub low_price: String,

    #[serde(rename = "v")]
    pub volume: String,

    #[serde(rename = "n")]
    pub trade_count: u64

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
#[serde(tag = "channel", content = "data")]
pub enum WsMessageRecv {
    // Wrapper for the first confirmation message
    #[serde(rename = "subscriptionResponse")]
    SubscriptionResponse(SubscriptionResponseData),

    #[serde(rename = "l2Book")]
    L2Book(OrderBookL2),

    #[serde(rename = "candle")]
    Candle(Candle),

    #[serde(rename = "trades")]
    Trade(Vec<Trade>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResponseData {
    pub method: String,
    pub subscription: SubscriptionResponseSubscription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResponseSubscription {
    #[serde(rename = "type")]
    pub sub_type: String,
    pub coin: String,
    pub interval: Option<String>, // just for candles
}

