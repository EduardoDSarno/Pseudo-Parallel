
use serde::{Deserialize, Serialize};
use crate::market_data::hyperliquid::protocols::{candle::Candle, subscribe::{Method, Subscription, SubscriptionResponseData}};

#[derive(Serialize, Deserialize, Debug)]
pub struct WsMessage {
    method: Method,
    subscription: Subscription
}

// Creating Message
impl WsMessage {
    pub fn new(method: Method, subscription: Subscription) -> WsMessage {
        WsMessage {
            method,
            subscription,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "channel", content = "data")]
pub enum WsMessageRecv {
    // Wrapper for the first confirmation message
    #[serde(rename = "subscriptionResponse")]
    SubscriptionResponse(SubscriptionResponseData),

    #[serde(rename = "candle")]
    Candle(Candle),
}

