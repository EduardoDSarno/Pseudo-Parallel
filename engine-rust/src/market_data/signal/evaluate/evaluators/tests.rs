use std::collections::VecDeque;

use crate::market_data::{
    engine::MarketView,
    signal::evaluate::evaluators::price_evaluator::PriceEvaluator,
    types::{Candle, CandleKey, Coins, Interval},
};

const TEST_OPEN_TIME: u64 = 1_000;

fn candle(close_price: f64) -> Candle {
    Candle {
        open_time_ms: TEST_OPEN_TIME,
        close_time_ms: TEST_OPEN_TIME + 1,
        coin: Coins::HYPE,
        interval: Interval::M5,
        open_price: close_price,
        close_price,
        high_price: close_price,
        low_price: close_price,
        volume: 1.0,
        trade_count: 1,
    }
}

fn key() -> CandleKey {
    CandleKey::new(Coins::HYPE, Interval::M5)
}

#[test]
fn first_price_only_records_state() {
    let mut evaluator = PriceEvaluator::new();
    let key = key();
    let closed_candles = VecDeque::new();
    let live_candle = candle(29.0);
    let view = MarketView {
        key: &key,
        closed_candles: &closed_candles,
        live_candle: &live_candle,
    };

    assert!(evaluator.price_evaluator(&view).is_empty());
}
