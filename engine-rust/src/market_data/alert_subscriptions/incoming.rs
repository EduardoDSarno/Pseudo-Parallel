/* Incoming subscription messages from JSON / Redis / API — deserialize only.
Use convert.rs to build SubscriptionManager. */

use serde::Deserialize;

use crate::market_data::types::{Coins, Interval};

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
    Indicator(IncomingIndicatorSubscription),
}

#[derive(Debug, Deserialize)]
pub struct IncomingIndicatorSubscription {
    pub coin: Coins,
    pub interval: Interval,
    pub kind: IncomingIndicatorKind,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum IncomingIndicatorKind {
    Atr(IncomingAtrRule),
}

#[derive(Debug, Deserialize)]
pub struct IncomingAtrRule {
    pub breakout_ratio: f64,
    pub debug_ratio: f64,
}

#[derive(Debug, Deserialize)]
pub struct IncomingPriceSubscription {
    pub coin: Coins,
    pub trigger_price: f64,
    #[serde(default)]
    pub direction: Option<String>,
}
