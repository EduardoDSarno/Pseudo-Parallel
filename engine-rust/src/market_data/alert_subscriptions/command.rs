use crate::market_data::signal::{
    indicator_rules::indicator::Indicator, price::alert::ManualPriceAlert,
};

#[derive(Debug, Clone)]
pub enum SubscriptionType {
    Price(ManualPriceAlert),
    Indicator(Indicator),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionCommand {
    Subscribe,
    Unsubscribe,
}

pub struct SubscriptionManager {
    pub sub_type: SubscriptionType,
    pub command: SubscriptionCommand,
}

