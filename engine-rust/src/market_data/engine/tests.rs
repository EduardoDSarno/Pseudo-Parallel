use super::core::*;
use crate::market_data::{constans::*, signal::event::{BreakoutAlert, Event}, types::{Candle, CandleKey, Coins, Interval}};

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
        volume: 1.0,
        trade_count: 1,
    }
}

fn seeded_engine() -> Engine
{
    let mut engine = Engine::new();
    let candles = (0..MAX_LENGTH_CANDLE_BUFFER)
        .map(|i| candle(i as u64 * M5_INTERVAL_MS, 100.5, 99.5, 100.0))
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
        .map(|i| candle(i as u64 * M5_INTERVAL_MS, 100.5, 99.5, 100.0))
        .collect();

    engine.buffers.insert(key, candles);

    let live = candle(100 * M5_INTERVAL_MS, 103.0, 100.0, 103.0);
    assert!(engine.evaluate_live_breakout(&live).is_none());
}

#[test]
fn first_live_spike_alerts_at_threshold()
{
    let mut engine = seeded_engine();
    let live = candle(100 * M5_INTERVAL_MS, 102.5, 100.0, 102.5);

    let alert = engine.evaluate_live_breakout(&live).unwrap();

    assert_eq!(spike_level(alert), 1);
}

#[test]
fn repeated_live_update_under_next_level_does_not_alert()
{
    let mut engine = seeded_engine();
    let live = candle(100 * M5_INTERVAL_MS, 102.5, 100.0, 102.5);
    let stronger_same_level = candle(100 * M5_INTERVAL_MS, 104.9, 100.0, 104.9);

    assert!(engine.evaluate_live_breakout(&live).is_some());
    assert!(engine.evaluate_live_breakout(&stronger_same_level).is_none());
}

#[test]
fn same_candle_alerts_again_at_next_spike_level()
{
    let mut engine = seeded_engine();
    let live = candle(100 * M5_INTERVAL_MS, 102.5, 100.0, 102.5);
    let next_level = candle(100 * M5_INTERVAL_MS, 105.0, 100.0, 105.0);

    assert_eq!(spike_level(engine.evaluate_live_breakout(&live).unwrap()), 1);
    assert_eq!(spike_level(engine.evaluate_live_breakout(&next_level).unwrap()), 2);
}

#[test]
fn new_candle_resets_live_alert_state()
{
    let mut engine = seeded_engine();
    let first_live = candle(100 * M5_INTERVAL_MS, 102.5, 100.0, 102.5);
    let next_live = candle(101 * M5_INTERVAL_MS, 105.5, 102.5, 105.5);

    engine.handle_candle(first_live.clone());
    assert_eq!(spike_level(engine.evaluate_live_breakout(&first_live).unwrap()), 1);

    engine.handle_candle(next_live.clone());
    assert_eq!(spike_level(engine.evaluate_live_breakout(&next_live).unwrap()), 1);
}
