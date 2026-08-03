use super::core::CandleStore;
use crate::market_data::{
    constans::M5_INTERVAL_MS,
    types::{Candle, CandleKey, Coins, Interval},
};

const TEST_MAX_CLOSED_CANDLES: usize = 3;

fn candle(open_time_ms: u64) -> Candle {
    Candle {
        open_time_ms,
        close_time_ms: open_time_ms + M5_INTERVAL_MS,
        coin: Coins::HYPE,
        interval: Interval::M5,
        open_price: 100.0,
        close_price: 100.0,
        high_price: 100.5,
        low_price: 99.5,
        volume: 1.0,
        trade_count: 1,
    }
}

fn key() -> CandleKey {
    CandleKey::new(Coins::HYPE, Interval::M5)
}

#[test]
fn last_seen_can_be_updated() {
    let mut store = CandleStore::new(TEST_MAX_CLOSED_CANDLES);
    let live = candle(100 * M5_INTERVAL_MS);

    store.set_last_seen(key(), live.clone());

    assert_eq!(
        store.last_seen(&key()).unwrap().open_time_ms,
        live.open_time_ms
    );
}

#[test]
fn closed_buffer_is_capped_at_configured_size() {
    let mut store = CandleStore::new(TEST_MAX_CLOSED_CANDLES);

    for index in 0..=TEST_MAX_CLOSED_CANDLES {
        store.push_closed_candle(key(), candle(index as u64 * M5_INTERVAL_MS));
    }

    let buffer = store.closed_buffer(&key()).unwrap();
    assert_eq!(buffer.len(), TEST_MAX_CLOSED_CANDLES);
    assert_eq!(buffer.front().unwrap().open_time_ms, M5_INTERVAL_MS);
}
