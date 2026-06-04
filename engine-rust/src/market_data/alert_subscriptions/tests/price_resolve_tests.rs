use crate::market_data::{
    alert_subscriptions::{
        apply::apply_subscription,
        command::{PriceSubscriptionSpec, SubscriptionCommand, SubscriptionManager, SubscriptionType},
        price_resolve::{build_manual_price_alert, resolve_price_direction},
    },
    config::MarketDataConfig,
    runtime::MarketDataRuntime,
    signal::price::ManualPriceDirection,
    types::Coins,
};

#[test]
fn resolve_price_direction_above_when_market_below_trigger() {
    assert_eq!(
        resolve_price_direction(65.0, 70.0).unwrap(),
        ManualPriceDirection::Above
    );
}

#[test]
fn resolve_price_direction_below_when_market_above_trigger() {
    assert_eq!(
        resolve_price_direction(75.0, 70.0).unwrap(),
        ManualPriceDirection::Below
    );
}

#[test]
fn resolve_price_direction_rejects_equal_reference_and_trigger() {
    assert!(resolve_price_direction(70.0, 70.0).is_err());
}

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

    let alert = build_manual_price_alert(&runtime, Coins::HYPE, 70.0, None).unwrap();
    assert_eq!(alert.direction, ManualPriceDirection::Above);
    assert!(runtime.alert_service().get(&alert.alert_key().unwrap()).is_some());
}
