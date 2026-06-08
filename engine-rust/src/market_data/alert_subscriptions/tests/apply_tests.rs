use crate::market_data::{
    alert_subscriptions::{
        apply::apply_subscription,
        command::{
            PriceSubscriptionSpec, SubscriptionCommand, SubscriptionManager, SubscriptionType,
        },
    },
    config::MarketDataConfig,
    runtime::MarketDataRuntime,
    signal::price::ManualPriceDirection,
    types::Coins,
};

#[test]
fn apply_infers_above_when_last_market_price_below_trigger() {
    let mut runtime = MarketDataRuntime::new(MarketDataConfig::default());
    runtime.set_last_market_price(Coins::HYPE, 65.0);

    let sub = SubscriptionManager {
        command: SubscriptionCommand::Subscribe,
        sub_type: SubscriptionType::Price(PriceSubscriptionSpec {
            coin: Coins::HYPE,
            trigger_price: 70.0,
            direction: None,
        }),
    };

    apply_subscription(&mut runtime, &sub).unwrap();

    let key = crate::market_data::signal::price::alert::ManualPriceAlert::new(
        Coins::HYPE,
        70.0,
        ManualPriceDirection::Above,
    )
    .alert_key()
    .unwrap();
    assert!(runtime.alert_service().get(&key).is_some());
}
