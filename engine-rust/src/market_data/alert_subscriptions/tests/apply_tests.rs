use crate::market_data::{
    alert_subscriptions::{
        apply::apply_subscription,
        command::{PriceSubscriptionSpec, SubscriptionCommand, SubscriptionManager},
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
fn apply_infers_above_from_live_m5_price() {
    let mut runtime = MarketDataRuntime::new(MarketDataConfig::default());
    runtime
        .candle_store
        .set_last_seen(CandleKey::new(Coins::HYPE, Interval::M5), live_candle(65.0));

    let subscription = SubscriptionManager {
        command: SubscriptionCommand::Subscribe,
        price: PriceSubscriptionSpec {
            coin: Coins::HYPE,
            trigger_price: 70.0,
            direction: None,
        },
    };

    apply_subscription(&mut runtime, &subscription).unwrap();

    let alerts = runtime
        .alert_service_mut()
        .take_crossed(Coins::HYPE, 65.0, 71.0);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].direction, ManualPriceDirection::Above);
}
