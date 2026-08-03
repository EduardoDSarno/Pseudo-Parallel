/* Incoming subscription messages from Redis — deserialize only.
Use convert.rs to build the domain SubscriptionManager. */

use serde::Deserialize;

use crate::market_data::types::Coins;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct IncomingSubscription {
    pub command: String,
    pub sub_type: IncomingSubscriptionType,
}

impl IncomingSubscription {
    /* Serde helper */
    pub fn parse_message(raw: &str) -> Result<IncomingSubscription, serde_json::Error> {
        serde_json::from_str(raw)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum IncomingSubscriptionType {
    Price(IncomingPriceSubscription),
}

#[derive(Debug, Deserialize)]
pub struct IncomingPriceSubscription {
    pub coin: Coins,
    pub trigger_price: f64,
    #[serde(default)]
    pub direction: Option<String>,
}
