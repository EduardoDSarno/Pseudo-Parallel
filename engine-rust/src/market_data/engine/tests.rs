use super::core::*;
use std::collections::VecDeque;
use crate::market_data::{constans::*, types::{Candle, CandleKey, Coins, Interval}};

const TEST_BASE_CLOSE: f64 = 100.0;
const TEST_BASE_HIGH: f64 = 100.5;
const TEST_BASE_LOW: f64 = 99.5;
const TEST_VOLUME: f64 = 1.0;
const TEST_TRADE_COUNT: u64 = 1;
const TEST_MAX_CLOSED_CANDLES: usize = 3;

const TEST_LIVE_OPEN_TIME: u64 = 100 * M5_INTERVAL_MS;
const TEST_NEXT_OPEN_TIME: u64 = 101 * M5_INTERVAL_MS;

fn candle(open_time_ms: u64, high_price: f64, low_price: f64, close_price: f64) -> Candle
{
    Candle
    {
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

fn seeded_engine() -> Engine
{
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);
    let candles = (0..TEST_MAX_CLOSED_CANDLES)
        .map(|i| candle(i as u64 * M5_INTERVAL_MS, TEST_BASE_HIGH, TEST_BASE_LOW, TEST_BASE_CLOSE))
        .collect();

    engine.seed_candles(candles).unwrap();
    engine
}

fn test_key() -> CandleKey
{
    CandleKey::new(Coins::HYPE, Interval::M5)
}

#[test]
fn seed_candles_rejects_empty_buffer()
{
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);

    assert!(engine.seed_candles(VecDeque::new()).is_err());
}

#[test]
fn seed_candles_rejects_short_buffer()
{
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);
    let candles = (0..TEST_MAX_CLOSED_CANDLES - 1)
        .map(|i| candle(i as u64 * M5_INTERVAL_MS, TEST_BASE_HIGH, TEST_BASE_LOW, TEST_BASE_CLOSE))
        .collect();

    assert!(engine.seed_candles(candles).is_err());
}

#[test]
fn seed_candles_trims_to_configured_size()
{
    let mut engine = Engine::new(TEST_MAX_CLOSED_CANDLES);
    let candles = (0..TEST_MAX_CLOSED_CANDLES + 2)
        .map(|i| candle(i as u64 * M5_INTERVAL_MS, TEST_BASE_HIGH, TEST_BASE_LOW, TEST_BASE_CLOSE))
        .collect();

    engine.seed_candles(candles).unwrap();

    let buffer = engine.buffers.get(&test_key()).unwrap();

    assert_eq!(buffer.len(), TEST_MAX_CLOSED_CANDLES);
    assert_eq!(buffer.front().unwrap().open_time_ms, 2 * M5_INTERVAL_MS);
}

#[test]
fn handle_candle_closes_previous_candle_when_open_time_changes()
{
    let mut engine = seeded_engine();
    let first_live = candle(TEST_LIVE_OPEN_TIME, 102.5, TEST_BASE_CLOSE, 102.5);
    let next_live = candle(TEST_NEXT_OPEN_TIME, 105.5, 102.5, 105.5);

    assert!(engine.handle_candle(first_live.clone()).is_none());
    assert_eq!(engine.handle_candle(next_live).unwrap().open_time_ms, first_live.open_time_ms);
    assert_eq!(engine.buffers.get(&test_key()).unwrap().len(), TEST_MAX_CLOSED_CANDLES);
}
