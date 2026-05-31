use super::core::*;
use crate::market_data::{
    constans::*,
    types::{Candle, CandleKey, Coins, Interval},
};
use std::collections::VecDeque;

const TEST_BASE_CLOSE: f64 = 100.0;
const TEST_BASE_HIGH: f64 = 100.5;
const TEST_BASE_LOW: f64 = 99.5;
const TEST_VOLUME: f64 = 1.0;
const TEST_TRADE_COUNT: u64 = 1;
const TEST_MAX_CLOSED_CANDLES: usize = 3;

const TEST_LIVE_OPEN_TIME: u64 = 100 * M5_INTERVAL_MS;

fn candle(open_time_ms: u64, high_price: f64, low_price: f64, close_price: f64) -> Candle {
    Candle {
        open_time_ms,
        close_time_ms: open_time_ms + M5_INTERVAL_MS,
        coin: Coins::HYPE,
        interval: Interval::M5,
        open_price: close_price,
        close_price,
        high_price,
        low_price,
        volume: TEST_VOLUME,
        trade_count: TEST_TRADE_COUNT,
    }
}

fn test_key() -> CandleKey {
    CandleKey::new(Coins::HYPE, Interval::M5)
}

#[test]
fn seed_candles_rejects_empty_buffer() {
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);

    assert!(engine.seed_candles(VecDeque::new()).is_err());
}

#[test]
fn seed_candles_rejects_short_buffer() {
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);
    let candles = (0..TEST_MAX_CLOSED_CANDLES - 1)
        .map(|i| {
            candle(
                i as u64 * M5_INTERVAL_MS,
                TEST_BASE_HIGH,
                TEST_BASE_LOW,
                TEST_BASE_CLOSE,
            )
        })
        .collect();

    assert!(engine.seed_candles(candles).is_err());
}

#[test]
fn seed_candles_trims_to_configured_size() {
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);
    let candles = (0..TEST_MAX_CLOSED_CANDLES + 2)
        .map(|i| {
            candle(
                i as u64 * M5_INTERVAL_MS,
                TEST_BASE_HIGH,
                TEST_BASE_LOW,
                TEST_BASE_CLOSE,
            )
        })
        .collect();

    engine.seed_candles(candles).unwrap();

    let buffer = engine.closed_buffer(&test_key()).unwrap();

    assert_eq!(buffer.len(), TEST_MAX_CLOSED_CANDLES);
    assert_eq!(buffer.front().unwrap().open_time_ms, 2 * M5_INTERVAL_MS);
}

#[test]
fn seed_candles_sets_last_seen_from_latest_candle() {
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);
    let candles = (0..TEST_MAX_CLOSED_CANDLES)
        .map(|i| {
            candle(
                i as u64 * M5_INTERVAL_MS,
                TEST_BASE_HIGH,
                TEST_BASE_LOW,
                TEST_BASE_CLOSE,
            )
        })
        .collect();

    engine.seed_candles(candles).unwrap();

    let buffer = engine.closed_buffer(&test_key()).unwrap();
    let last_seen = engine.last_seen(&test_key()).unwrap();

    assert_eq!(last_seen.open_time_ms, buffer.back().unwrap().open_time_ms);
}

#[test]
fn seed_candles_at_keeps_forming_candle_out_of_closed_buffer() {
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);
    let candles = (0..TEST_MAX_CLOSED_CANDLES + 1)
        .map(|i| {
            candle(
                i as u64 * M5_INTERVAL_MS,
                TEST_BASE_HIGH,
                TEST_BASE_LOW,
                TEST_BASE_CLOSE,
            )
        })
        .collect();
    let forming_open_time = TEST_MAX_CLOSED_CANDLES as u64 * M5_INTERVAL_MS;
    let seed_end_time = forming_open_time + 1;

    engine.seed_candles_at(candles, seed_end_time).unwrap();

    let buffer = engine.closed_buffer(&test_key()).unwrap();
    let last_seen = engine.last_seen(&test_key()).unwrap();

    assert_eq!(buffer.len(), TEST_MAX_CLOSED_CANDLES);
    assert_eq!(
        buffer.back().unwrap().open_time_ms,
        forming_open_time - M5_INTERVAL_MS
    );
    assert_eq!(last_seen.open_time_ms, forming_open_time);
}

#[test]
fn seed_candles_at_uses_latest_closed_as_last_seen_when_no_forming_candle() {
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);
    let candles = (0..TEST_MAX_CLOSED_CANDLES + 1)
        .map(|i| {
            candle(
                i as u64 * M5_INTERVAL_MS,
                TEST_BASE_HIGH,
                TEST_BASE_LOW,
                TEST_BASE_CLOSE,
            )
        })
        .collect();
    let latest_open_time = TEST_MAX_CLOSED_CANDLES as u64 * M5_INTERVAL_MS;
    let seed_end_time = latest_open_time + M5_INTERVAL_MS + 1;

    engine.seed_candles_at(candles, seed_end_time).unwrap();

    let buffer = engine.closed_buffer(&test_key()).unwrap();
    let last_seen = engine.last_seen(&test_key()).unwrap();

    assert_eq!(buffer.len(), TEST_MAX_CLOSED_CANDLES);
    assert_eq!(buffer.back().unwrap().open_time_ms, latest_open_time);
    assert_eq!(last_seen.open_time_ms, latest_open_time);
}

#[test]
fn seed_candles_at_rejects_short_closed_buffer_after_forming_split() {
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);
    let candles = (0..TEST_MAX_CLOSED_CANDLES)
        .map(|i| {
            candle(
                i as u64 * M5_INTERVAL_MS,
                TEST_BASE_HIGH,
                TEST_BASE_LOW,
                TEST_BASE_CLOSE,
            )
        })
        .collect();
    let forming_open_time = (TEST_MAX_CLOSED_CANDLES - 1) as u64 * M5_INTERVAL_MS;
    let seed_end_time = forming_open_time + 1;

    assert!(engine.seed_candles_at(candles, seed_end_time).is_err());
}

#[test]
fn last_seen_can_be_updated() {
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);
    let live = candle(TEST_LIVE_OPEN_TIME, 102.5, TEST_BASE_CLOSE, 102.5);

    engine.set_last_seen(test_key(), live.clone());

    assert_eq!(
        engine.last_seen(&test_key()).unwrap().open_time_ms,
        live.open_time_ms
    );
}

#[test]
fn push_closed_candle_caps_buffer_at_configured_size() {
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);

    for i in 0..TEST_MAX_CLOSED_CANDLES + 1 {
        engine.push_closed_candle(
            test_key(),
            candle(
                i as u64 * M5_INTERVAL_MS,
                TEST_BASE_HIGH,
                TEST_BASE_LOW,
                TEST_BASE_CLOSE,
            ),
        );
    }

    assert_eq!(
        engine
            .closed_buffer(&test_key())
            .unwrap()
            .front()
            .unwrap()
            .open_time_ms,
        M5_INTERVAL_MS
    );
    assert_eq!(
        engine.closed_buffer(&test_key()).unwrap().len(),
        TEST_MAX_CLOSED_CANDLES
    );
}
