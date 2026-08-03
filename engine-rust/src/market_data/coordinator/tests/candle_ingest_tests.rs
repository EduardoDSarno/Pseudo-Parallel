use crate::market_data::{
    candle_store::CandleStore,
    constans::M5_INTERVAL_MS,
    coordinator::candle_ingest::apply_candle,
    types::{Candle, CandleKey, Coins, Interval},
};

const TEST_MAX_CLOSED: usize = 3;

fn candle(open_time_ms: u64, close_price: f64) -> Candle {
    Candle {
        open_time_ms,
        close_time_ms: open_time_ms + M5_INTERVAL_MS,
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
fn first_update_sets_live_candle_without_previous_price() {
    let mut store = CandleStore::new(TEST_MAX_CLOSED);
    let snapshot = apply_candle(&mut store, candle(0, 40.0));

    assert_eq!(snapshot.previous_price, None);
    assert_eq!(snapshot.current_price, 40.0);
    assert_eq!(
        store.last_seen(&snapshot.candle_key).unwrap().close_price,
        40.0
    );
}

#[test]
fn new_bar_moves_previous_candle_to_closed_buffer_once() {
    let mut store = CandleStore::new(TEST_MAX_CLOSED);
    let key = CandleKey::new(Coins::HYPE, Interval::M5);

    apply_candle(&mut store, candle(0, 40.0));
    let snapshot = apply_candle(&mut store, candle(M5_INTERVAL_MS, 41.0));
    apply_candle(&mut store, candle(M5_INTERVAL_MS, 42.0));

    assert_eq!(snapshot.previous_price, Some(40.0));
    assert_eq!(store.closed_buffer(&key).unwrap().len(), 1);
    assert_eq!(store.closed_buffer(&key).unwrap()[0].close_price, 40.0);
    assert_eq!(store.last_seen(&key).unwrap().close_price, 42.0);
}
