use crate::market_data::{signal::price::ManualPriceDirection, types::Coins};

/* Wire / channel price sub — direction may be omitted and inferred in apply. */
#[derive(Debug, Clone)]
pub struct PriceSubscriptionSpec {
    pub coin: Coins,
    pub trigger_price: f64,
    pub direction: Option<ManualPriceDirection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionCommand {
    Subscribe,
    Unsubscribe,
}

#[derive(Debug)]
pub struct SubscriptionManager {
    pub price: PriceSubscriptionSpec,
    pub command: SubscriptionCommand,
}
