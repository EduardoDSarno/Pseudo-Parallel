use crate::market_data::{
    alert_subscriptions::{
        command::{SubscriptionCommand, SubscriptionType},
        convert::to_subscription_manager,
        incoming::IncomingSubscription,
    },
    signal::{
        indicator_rules::{AtrRule, IndicatorRuleKind},
        price::ManualPriceDirection,
    },
    types::{Coins, Interval},
};

#[test]
fn deserializes_price_subscription() {
    let json = r#"{
        "command": "subscribe",
        "sub_type": {
            "type": "price",
            "coin": "HYPE",
            "trigger_price": 69.3,
            "direction": "below"
        }
    }"#;

    let incoming: IncomingSubscription = serde_json::from_str(json).unwrap();
    let manager = to_subscription_manager(incoming).unwrap();

    assert_eq!(manager.command, SubscriptionCommand::Subscribe);
    match manager.sub_type {
        SubscriptionType::Price(alert) => {
            assert_eq!(alert.coin, Coins::HYPE);
            assert_eq!(alert.trigger_price, 69.3);
            assert_eq!(alert.direction, ManualPriceDirection::Below);
        }
        SubscriptionType::Indicator(_) => panic!("expected price"),
    }
}

#[test]
fn deserializes_indicator_subscription() {
    let json = r#"{
        "command": "subscribe",
        "sub_type": {
            "type": "indicator",
            "coin": "HYPE",
            "interval": "5m",
            "kind": {
                "type": "atr",
                "breakout_ratio": 2.5,
                "debug_ratio": 0.8
            }
        }
    }"#;

    let incoming: IncomingSubscription = serde_json::from_str(json).unwrap();
    let manager = to_subscription_manager(incoming).unwrap();

    assert_eq!(manager.command, SubscriptionCommand::Subscribe);
    match manager.sub_type {
        SubscriptionType::Indicator(ind) => {
            assert_eq!(ind.key.coin, Coins::HYPE);
            assert_eq!(ind.key.interval, Interval::M5);
            assert_eq!(
                ind.kind,
                IndicatorRuleKind::Atr(AtrRule {
                    breakout_ratio: 2.5,
                    debug_ratio: 0.8,
                })
            );
        }
        SubscriptionType::Price(_) => panic!("expected indicator"),
    }
}

#[test]
fn rejects_indicator_subscription_without_kind() {
    let json = r#"{
        "command": "subscribe",
        "sub_type": {
            "type": "indicator",
            "coin": "HYPE",
            "interval": "5m"
        }
    }"#;

    let result: Result<IncomingSubscription, _> = serde_json::from_str(json);
    assert!(result.is_err());
}
