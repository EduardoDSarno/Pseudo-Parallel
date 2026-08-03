use crate::market_data::{
    alert_subscriptions::command::{
        PriceSubscriptionSpec, SubscriptionCommand, SubscriptionManager,
    },
    config::MarketDataConfig,
    constans::M5_INTERVAL_MS,
    runtime::MarketDataRuntime,
    signal::price::ManualPriceDirection,
    types::{Candle, CandleKey, Coins, Interval},
};

fn live_candle(close_price: f64) -> Candle {
    Candle {
        open_time_ms: 0,
        close_time_ms: M5_INTERVAL_MS,
        coin: Coins::HYPE,
        interval: Interval::M5,
        open_price: close_price,
        close_price,
        high_price: close_price,
        low_price: close_price,
        volume: 0.0,
        trade_count: 0,
    }
}

#[test]
fn subscription_without_direction_uses_live_m5_price() {
    let mut runtime = MarketDataRuntime::new(MarketDataConfig::default());
    runtime
        .candle_store
        .set_last_seen(CandleKey::new(Coins::HYPE, Interval::M5), live_candle(60.0));

    runtime
        .load_signal_subscriptions(vec![SubscriptionManager {
            command: SubscriptionCommand::Subscribe,
            price: PriceSubscriptionSpec {
                coin: Coins::HYPE,
                trigger_price: 70.0,
                direction: None,
            },
        }])
        .unwrap();

    let alerts = runtime
        .alert_service_mut()
        .take_crossed(Coins::HYPE, 60.0, 71.0);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].direction, ManualPriceDirection::Above);
}

#[test]
fn explicit_direction_does_not_need_live_price() {
    let mut runtime = MarketDataRuntime::new(MarketDataConfig::default());

    runtime
        .load_signal_subscriptions(vec![SubscriptionManager {
            command: SubscriptionCommand::Subscribe,
            price: PriceSubscriptionSpec {
                coin: Coins::HYPE,
                trigger_price: 69.3,
                direction: Some(ManualPriceDirection::Below),
            },
        }])
        .unwrap();

    let alerts = runtime
        .alert_service_mut()
        .take_crossed(Coins::HYPE, 70.0, 69.0);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].direction, ManualPriceDirection::Below);
}
