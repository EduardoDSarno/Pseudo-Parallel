use std::collections::VecDeque;

use crate::market_data::{
    candle_store::CandleView,
    signal::{
        evaluate::evaluators::{
            indicator_evaluator::IndicatorEvaluator, price_evaluator::PriceEvaluator,
        },
        indicator_rules::{AtrRule, IndicatorRule, IndicatorRuleId, IndicatorRuleKind},
    },
    types::{Candle, CandleKey, Coins, Interval},
};

#[test]
fn evaluate_price_with_empty_service_returns_no_alerts() {
    let evaluator = PriceEvaluator::new();
    let service = crate::market_data::signal::price::PriceAlertService::new();

    assert!(evaluator
        .evaluate_price(&service, Coins::HYPE, 29.0, 30.0)
        .is_empty());
}

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

fn candle_view<'a>(
    key: &'a CandleKey,
    closed_candles: &'a VecDeque<Candle>,
    live_candle: &'a Candle,
) -> CandleView<'a> {
    CandleView {
        key,
        closed_candles,
        live_candle,
    }
}

fn atr_rule(id: u64, key: CandleKey, breakout_ratio: f64) -> IndicatorRule {
    IndicatorRule {
        id: IndicatorRuleId(id),
        key,
        kind: IndicatorRuleKind::Atr(AtrRule {
            breakout_ratio,
            debug_ratio: 0.8,
        }),
    }
}

#[test]
fn evaluate_indicators_with_no_rules_returns_no_alerts() {
    let mut evaluator = IndicatorEvaluator::new();
    let key = CandleKey::new(Coins::HYPE, Interval::M5);
    let closed_candles = VecDeque::from([
        candle(0, 101.0, 99.0, 100.0),
        candle(300_000, 101.0, 99.0, 100.0),
    ]);
    let live_candle = candle(600_000, 106.0, 100.0, 106.0);
    let view = candle_view(&key, &closed_candles, &live_candle);

    assert!(evaluator.evaluate_indicator(&view, &[]).is_empty());
}

#[test]
fn indicator_evaluator_skips_rule_for_wrong_key() {
    let mut evaluator = IndicatorEvaluator::new();
    let view_key = CandleKey::new(Coins::HYPE, Interval::M5);
    let rule_key = CandleKey::new(Coins::HYPE, Interval::M15);
    let closed_candles = VecDeque::from([
        candle(0, 101.0, 99.0, 100.0),
        candle(300_000, 101.0, 99.0, 100.0),
    ]);
    let live_candle = candle(600_000, 106.0, 100.0, 106.0);
    let view = candle_view(&view_key, &closed_candles, &live_candle);
    let rule = atr_rule(1, rule_key, 2.5);

    assert!(evaluator.evaluate_indicator(&view, &[rule]).is_empty());
}
