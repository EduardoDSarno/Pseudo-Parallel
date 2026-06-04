use std::collections::VecDeque;

use crate::market_data::{
    candle_store::CandleStore,
    coordinator::candle_ingest::apply_candle,
    types::{Candle, CandleKey, Coins, Interval},
};

const TEST_MAX_CLOSED: usize = 3;

fn candle(open_time_ms: u64) -> Candle {
    Candle {
        open_time_ms,
        close_time_ms: open_time_ms + 300_000,
        coin: Coins::HYPE,
        interval: Interval::M5,
        open_price: 1.0,
        close_price: 1.0,
        high_price: 1.0,
        low_price: 1.0,
        volume: 0.0,
        trade_count: 0,
    }
}

#[test]
fn new_bar_does_not_duplicate_tail_already_in_closed_buffer() {
    let mut engine = CandleStore::new(TEST_MAX_CLOSED);
    let key = CandleKey::new(Coins::HYPE, Interval::M5);
    let candles: VecDeque<Candle> = (0..TEST_MAX_CLOSED)
        .map(|i| candle(i as u64 * 300_000))
        .collect();

    engine.seed_candles(candles).unwrap();
    let len_after_seed = engine.closed_buffer(&key).unwrap().len();

    let tail_open = engine.last_seen(&key).unwrap().open_time_ms;
    let next_bar = candle(tail_open + 300_000);
    apply_candle(&mut engine, next_bar);

    assert_eq!(engine.closed_buffer(&key).unwrap().len(), len_after_seed);
}
