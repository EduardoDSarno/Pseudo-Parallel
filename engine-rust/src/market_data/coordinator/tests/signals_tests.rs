use crate::market_data::{
    config::MarketDataConfig,
    constans::M5_INTERVAL_MS,
    coordinator::{candle_ingest::IngestedCandleSnapshot, signal_input::SignalInput},
    runtime::MarketDataRuntime,
    signal::{
        event::Event,
        indicator_rules::{AtrRule, IndicatorRuleKind},
        price::{alert::ManualPriceAlert, ManualPriceDirection},
    },
    types::Candle,
    types::{CandleKey, Coins, Interval},
};

use std::collections::VecDeque;

const TEST_TRIGGER_PRICE: f64 = 42.0;

fn runtime_with_price_alert() -> MarketDataRuntime {
    let mut runtime = MarketDataRuntime::new(MarketDataConfig::default());
    runtime
        .alert_service_mut()
        .subscribe(ManualPriceAlert::new(
            Coins::HYPE,
            TEST_TRIGGER_PRICE,
            ManualPriceDirection::Above,
        ))
        .unwrap();
    runtime.set_last_market_price(Coins::HYPE, 40.0);
    runtime
}

fn snapshot(interval: Interval, close_price: f64, bar_just_closed: bool) -> IngestedCandleSnapshot {
    IngestedCandleSnapshot {
        candle_key: CandleKey::new(Coins::HYPE, interval),
        close_price,
        bar_just_closed,
    }
}

fn candle(open_time_ms: u64, high_price: f64, low_price: f64, close_price: f64) -> Candle {
    Candle {
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

#[test]
fn m5_candle_runs_price_alert_evaluation() {
    let mut runtime = runtime_with_price_alert();

    let alerts = runtime.run_signals(SignalInput::Candle(snapshot(Interval::M5, 43.0, false)));

    assert_eq!(alerts.len(), 1);
    assert!(matches!(
        alerts[0].event,
        Event::ManualPriceTriggered { .. }
    ));
}

#[test]
fn non_m5_candle_skips_price_alert_evaluation() {
    let mut runtime = runtime_with_price_alert();

    let alerts = runtime.run_signals(SignalInput::Candle(snapshot(Interval::M15, 43.0, false)));

    assert!(alerts.is_empty());
}

#[test]
fn indicator_rules_emit_alert_through_signal_pipeline() {
    let config = MarketDataConfig {
        max_closed_candles: 4,
        ..MarketDataConfig::default()
    };
    let mut runtime = MarketDataRuntime::new(config);
    let key = CandleKey::new(Coins::HYPE, Interval::M5);
    let closed_candles = VecDeque::from([
        candle(0, 101.0, 99.0, 100.0),
        candle(M5_INTERVAL_MS, 101.0, 99.0, 100.0),
        candle(2 * M5_INTERVAL_MS, 101.0, 99.0, 100.0),
        candle(3 * M5_INTERVAL_MS, 120.0, 90.0, 115.0),
    ]);
    let live_candle = candle(4 * M5_INTERVAL_MS, 100.5, 100.0, 100.5);

    runtime
        .candle_store
        .seed_candles_at(closed_candles, 4 * M5_INTERVAL_MS + 1)
        .unwrap();
    runtime.candle_store.set_last_seen(key.clone(), live_candle);
    runtime.indicator_rule_service_mut().subscribe(
        key,
        IndicatorRuleKind::Atr(AtrRule {
            breakout_ratio: 2.5,
            debug_ratio: 0.8,
        }),
    );

    let alerts = runtime.run_signals(SignalInput::Candle(snapshot(Interval::M5, 100.5, true)));

    assert!(alerts
        .iter()
        .any(|alert| matches!(alert.event, Event::AtrBreakout { .. })));
}

#[test]
fn indicator_rules_skip_eval_until_bar_close() {
    let config = MarketDataConfig {
        max_closed_candles: 4,
        ..MarketDataConfig::default()
    };
    let mut runtime = MarketDataRuntime::new(config);
    let key = CandleKey::new(Coins::HYPE, Interval::M5);
    let closed_candles = VecDeque::from([
        candle(0, 101.0, 99.0, 100.0),
        candle(M5_INTERVAL_MS, 101.0, 99.0, 100.0),
        candle(2 * M5_INTERVAL_MS, 101.0, 99.0, 100.0),
        candle(3 * M5_INTERVAL_MS, 120.0, 90.0, 115.0),
    ]);

    runtime
        .candle_store
        .seed_candles_at(closed_candles, 4 * M5_INTERVAL_MS + 1)
        .unwrap();
    runtime
        .candle_store
        .set_last_seen(key.clone(), candle(4 * M5_INTERVAL_MS, 150.0, 100.0, 150.0));
    runtime.indicator_rule_service_mut().subscribe(
        key,
        IndicatorRuleKind::Atr(AtrRule {
            breakout_ratio: 2.5,
            debug_ratio: 0.8,
        }),
    );

    let alerts = runtime.run_signals(SignalInput::Candle(snapshot(Interval::M5, 150.0, false)));

    assert!(alerts.is_empty());
}
