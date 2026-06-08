use crate::market_data::{
    alert_subscriptions::{
        command::{PriceSubscriptionSpec, SubscriptionCommand, SubscriptionManager, SubscriptionType},
        placeholder::dev_signal_subscriptions,
    },
    config::MarketDataConfig,
    signal::price::{alert::ManualPriceAlert, ManualPriceDirection},
    types::{CandleKey, Coins, Interval},
};

use crate::market_data::runtime::MarketDataRuntime;

#[test]
fn last_market_price_returns_set_price() {
    let mut runtime = MarketDataRuntime::new(MarketDataConfig::default());
    assert_eq!(runtime.last_market_price(Coins::HYPE), None);

    runtime.set_last_market_price(Coins::HYPE, 65.0);
    assert_eq!(runtime.last_market_price(Coins::HYPE), Some(65.0));
}

#[test]
fn load_signal_subscriptions_applies_price_alerts() {
    let mut runtime = MarketDataRuntime::new(MarketDataConfig::default());
    runtime.set_last_market_price(Coins::HYPE, 60.0);

    let subs = vec![SubscriptionManager {
        command: SubscriptionCommand::Subscribe,
        sub_type: SubscriptionType::Price(PriceSubscriptionSpec {
            coin: Coins::HYPE,
            trigger_price: 70.0,
            direction: None,
        }),
    }];

    runtime.load_signal_subscriptions(subs).unwrap();

    let alert = ManualPriceAlert::new(Coins::HYPE, 70.0, ManualPriceDirection::Above);
    assert!(runtime
        .alert_service()
        .get(&alert.alert_key().unwrap())
        .is_some());
}

#[test]
fn load_signal_subscriptions_applies_dev_placeholder_subs() {
    let mut runtime = MarketDataRuntime::new(MarketDataConfig::default());
    runtime.set_last_market_price(Coins::HYPE, 60.0);

    runtime
        .load_signal_subscriptions(dev_signal_subscriptions())
        .unwrap();

    let below_697 = ManualPriceAlert::new(Coins::HYPE, 69.3, ManualPriceDirection::Below);
    assert!(runtime
        .alert_service()
        .get(&below_697.alert_key().unwrap())
        .is_some());
}

#[test]
fn load_default_indicator_rules_subscribes_atr_for_key() {
    let mut runtime = MarketDataRuntime::new(MarketDataConfig::default());
    let key = CandleKey::new(Coins::HYPE, Interval::M5);

    runtime.load_default_indicator_rules(&[key.clone()]);

    assert!(!runtime.indicator_rule_service().rules_for_key(&key).is_empty());
}
