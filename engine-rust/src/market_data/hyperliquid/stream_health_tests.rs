use std::time::Instant;

use crate::market_data::{
    constans::{M5_INTERVAL_MS, STREAM_STALE_MULTIPLIER},
    hyperliquid::stream_health::{is_stream_stale, CandleStreamHealth},
    types::{Candle, CandleKey, Coins, Interval},
};

#[test]
fn stale_when_elapsed_exceeds_twice_interval() {
    assert!(is_stream_stale(
        M5_INTERVAL_MS * 2 + 1,
        M5_INTERVAL_MS,
        STREAM_STALE_MULTIPLIER
    ));
}

#[test]
fn not_stale_at_exactly_twice_interval() {
    assert!(!is_stream_stale(
        M5_INTERVAL_MS * 2,
        M5_INTERVAL_MS,
        STREAM_STALE_MULTIPLIER
    ));
}

#[test]
fn record_candle_then_check_stale_does_not_panic() {
    let keys = [CandleKey::new(Coins::HYPE, Interval::M5)];
    let mut health = CandleStreamHealth::new(&keys, Instant::now());
    let candle = Candle {
        open_time_ms: 0,
        close_time_ms: 0,
        coin: Coins::HYPE,
        interval: Interval::M5,
        open_price: 1.0,
        close_price: 1.0,
        high_price: 1.0,
        low_price: 1.0,
        volume: 0.0,
        trade_count: 0,
    };
    health.record_candle(&candle);
    health.check_stale();
}
