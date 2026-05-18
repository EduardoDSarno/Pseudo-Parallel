use std::collections::VecDeque;

use crate::market_data::{
    engine::MarketView,
    signal::{evaluate::evaluators::price_evaluator::PriceEvaluator, event::Event},
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

#[test]
fn second_price_crossing_hardcoded_above_returns_alert() {
    let mut evaluator = PriceEvaluator::new();
    let key = key();
    let closed_candles = VecDeque::new();
    let first_live = candle(29.0);
    let first_view = MarketView {
        key: &key,
        closed_candles: &closed_candles,
        live_candle: &first_live,
    };
    evaluator.price_evaluator(&first_view);

    let second_live = candle(31.0);
    let second_view = MarketView {
        key: &key,
        closed_candles: &closed_candles,
        live_candle: &second_live,
    };
    let alerts = evaluator.price_evaluator(&second_view);

    assert_eq!(alerts.len(), 1);
    assert!(matches!(
        alerts[0].event,
        Event::ManualPriceTriggered {
            trigger_price: 30.0,
            ..
        }
    ));
}
