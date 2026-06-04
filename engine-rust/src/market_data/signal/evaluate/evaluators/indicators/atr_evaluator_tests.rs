use std::collections::VecDeque;

use crate::market_data::{
    candle_store::CandleView,
    signal::{
        evaluate::evaluators::indicators::atr_evaluator::AtrEvaluator,
        indicator_rules::{AtrRule, IndicatorRuleId},
    },
    types::{Candle, CandleKey, Coins, Interval},
};

fn candle(open_time_ms: u64, high_price: f64, low_price: f64, close_price: f64) -> Candle {
    Candle {
        open_time_ms,
        close_time_ms: open_time_ms + 300_000,
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

fn volatile_closed_view() -> (CandleKey, VecDeque<Candle>, Candle) {
    (
        CandleKey::new(Coins::HYPE, Interval::M5),
        VecDeque::from([
            candle(0, 101.0, 99.0, 100.0),
            candle(300_000, 101.0, 99.0, 100.0),
            candle(600_000, 101.0, 99.0, 100.0),
            candle(900_000, 120.0, 90.0, 115.0),
        ]),
        candle(1_200_000, 100.5, 100.0, 100.5),
    )
}

fn atr_rule(breakout_ratio: f64) -> AtrRule {
    AtrRule {
        breakout_ratio,
        debug_ratio: 0.8,
    }
}

#[test]
fn atr_rule_with_high_threshold_does_not_alert() {
    let mut evaluator = AtrEvaluator::new();
    let key = CandleKey::new(Coins::HYPE, Interval::M5);
    let closed_candles = VecDeque::from([
        candle(0, 101.0, 99.0, 100.0),
        candle(300_000, 101.0, 99.0, 100.0),
        candle(600_000, 101.0, 99.0, 100.0),
    ]);
    let live_candle = candle(900_000, 106.0, 100.0, 106.0);
    let view = CandleView {
        key: &key,
        closed_candles: &closed_candles,
        live_candle: &live_candle,
    };

    let alert = evaluator.evaluate_atr(&view, IndicatorRuleId(1), &atr_rule(4.0));

    assert!(alert.is_none());
}

#[test]
fn volatile_open_bar_does_not_affect_closed_only_atr() {
    let mut evaluator = AtrEvaluator::new();
    let key = CandleKey::new(Coins::HYPE, Interval::M5);
    let closed_candles = VecDeque::from([
        candle(0, 101.0, 99.0, 100.0),
        candle(300_000, 101.0, 99.0, 100.0),
        candle(600_000, 101.0, 99.0, 100.0),
    ]);
    let live_candle = candle(900_000, 150.0, 100.0, 150.0);
    let view = CandleView {
        key: &key,
        closed_candles: &closed_candles,
        live_candle: &live_candle,
    };

    let alert = evaluator.evaluate_atr(&view, IndicatorRuleId(1), &atr_rule(2.5));

    assert!(alert.is_none());
}

#[test]
fn atr_rule_with_lower_threshold_alerts() {
    let mut evaluator = AtrEvaluator::new();
    let (key, closed_candles, live_candle) = volatile_closed_view();
    let view = CandleView {
        key: &key,
        closed_candles: &closed_candles,
        live_candle: &live_candle,
    };

    let alert = evaluator.evaluate_atr(&view, IndicatorRuleId(1), &atr_rule(2.5));

    assert!(alert.is_some());
    assert_eq!(alert.unwrap().rule_id, IndicatorRuleId(1));
}

#[test]
fn duplicate_suppression_is_per_indicator_rule_id() {
    let mut evaluator = AtrEvaluator::new();
    let (key, closed_candles, live_candle) = volatile_closed_view();
    let view = CandleView {
        key: &key,
        closed_candles: &closed_candles,
        live_candle: &live_candle,
    };

    let first = evaluator.evaluate_atr(&view, IndicatorRuleId(1), &atr_rule(2.5));
    let second = evaluator.evaluate_atr(&view, IndicatorRuleId(1), &atr_rule(2.5));
    let other_rule = evaluator.evaluate_atr(&view, IndicatorRuleId(2), &atr_rule(2.5));

    assert!(first.is_some());
    assert!(second.is_none());
    assert!(other_rule.is_some());
}
