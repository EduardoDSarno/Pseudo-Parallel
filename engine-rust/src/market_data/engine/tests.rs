use super::core::*;
use crate::market_data::{constans::*, signal::event::{BreakoutAlert, Event}, types::{Candle, CandleKey, Coins, Interval}};

const TEST_BASE_CLOSE: f64 = 100.0;
const TEST_BASE_HIGH: f64 = 100.5;
const TEST_BASE_LOW: f64 = 99.5;
const TEST_VOLUME: f64 = 1.0;
const TEST_TRADE_COUNT: u64 = 1;

const TEST_LIVE_OPEN_TIME: u64 = 100 * M5_INTERVAL_MS;
const TEST_NEXT_OPEN_TIME: u64 = 101 * M5_INTERVAL_MS;

const FIRST_SPIKE_LEVEL: u64 = 1;
const SECOND_SPIKE_LEVEL: u64 = 2;

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
    let mut engine = Engine::new();
    let candles = (0..MAX_LENGTH_CANDLE_BUFFER)
        .map(|i| candle(i as u64 * M5_INTERVAL_MS, TEST_BASE_HIGH, TEST_BASE_LOW, TEST_BASE_CLOSE))
        .collect();

    engine.seed_candles(candles).unwrap();
    engine
}

fn spike_level(alert: BreakoutAlert) -> u64
{
    let Event::ATR { spike_level, .. } = alert.event;
    spike_level
}

#[test]
fn no_live_alert_when_buffer_is_warming_up()
{
    let mut engine = Engine::new();
    let key = CandleKey::new(Coins::HYPE, Interval::M5);
    let candles = (0..MAX_LENGTH_CANDLE_BUFFER - 1)
        .map(|i| candle(i as u64 * M5_INTERVAL_MS, TEST_BASE_HIGH, TEST_BASE_LOW, TEST_BASE_CLOSE))
        .collect();

    engine.buffers.insert(key, candles);

    let live = candle(TEST_LIVE_OPEN_TIME, 103.0, TEST_BASE_CLOSE, 103.0);
    assert!(engine.evaluate_live_breakout(&live).is_none());
}

#[test]
fn first_live_spike_alerts_at_threshold()
{
    let mut engine = seeded_engine();
    let live = candle(TEST_LIVE_OPEN_TIME, 102.5, TEST_BASE_CLOSE, 102.5);

    let alert = engine.evaluate_live_breakout(&live).unwrap();

    assert_eq!(spike_level(alert), FIRST_SPIKE_LEVEL);
}

#[test]
fn repeated_live_update_under_next_level_does_not_alert()
{
    let mut engine = seeded_engine();
    let live = candle(TEST_LIVE_OPEN_TIME, 102.5, TEST_BASE_CLOSE, 102.5);
    let stronger_same_level = candle(TEST_LIVE_OPEN_TIME, 104.9, TEST_BASE_CLOSE, 104.9);

    assert!(engine.evaluate_live_breakout(&live).is_some());
    assert!(engine.evaluate_live_breakout(&stronger_same_level).is_none());
}

#[test]
fn same_candle_alerts_again_at_next_spike_level()
{
    let mut engine = seeded_engine();
    let live = candle(TEST_LIVE_OPEN_TIME, 102.5, TEST_BASE_CLOSE, 102.5);
    let next_level = candle(TEST_LIVE_OPEN_TIME, 105.0, TEST_BASE_CLOSE, 105.0);

    assert_eq!(spike_level(engine.evaluate_live_breakout(&live).unwrap()), FIRST_SPIKE_LEVEL);
    assert_eq!(spike_level(engine.evaluate_live_breakout(&next_level).unwrap()), SECOND_SPIKE_LEVEL);
}

#[test]
fn new_candle_resets_live_alert_state()
{
    let mut engine = seeded_engine();
    let first_live = candle(TEST_LIVE_OPEN_TIME, 102.5, TEST_BASE_CLOSE, 102.5);
    let next_live = candle(TEST_NEXT_OPEN_TIME, 105.5, 102.5, 105.5);

    engine.handle_candle(first_live.clone());
    assert_eq!(spike_level(engine.evaluate_live_breakout(&first_live).unwrap()), FIRST_SPIKE_LEVEL);

    engine.handle_candle(next_live.clone());
    assert_eq!(spike_level(engine.evaluate_live_breakout(&next_live).unwrap()), FIRST_SPIKE_LEVEL);
}
