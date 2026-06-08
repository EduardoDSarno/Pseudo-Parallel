use crate::market_data::{alert_subscriptions::command::SubscriptionManager, types::Candle};

/* Expandable market updates types */
pub enum MarketUpdate {
    Candle(Candle),
    Subscription(SubscriptionManager),
}
