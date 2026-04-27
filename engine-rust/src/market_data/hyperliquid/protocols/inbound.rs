use serde::{Deserialize, Serialize};

use crate::market_data::hyperliquid::protocols::{data_models::candle::Candle, subscribe::{Method, SubscriptionData}};

#[derive(Deserialize, Serialize)]
#[serde(tag = "channel", content = "data", rename_all = "camelCase")]
pub enum InboundMessage
{
    SubscriptionResponse(SubscriptionResponseData),
    Candle(Candle)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct SubscriptionResponseData 
{
    pub method: Method,
    pub subscription: SubscriptionData,
}
