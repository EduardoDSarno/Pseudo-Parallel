use crate::market_data::{
    config::MarketDataConfig,
    coordinator::{candle_ingest::IngestedCandleSnapshot, signal_input::SignalInput},
    runtime::MarketDataRuntime,
    signal::{
        event::Event,
        price::{alert::ManualPriceAlert, ManualPriceDirection},
    },
    types::{CandleKey, Coins, Interval},
};

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

fn snapshot(interval: Interval, close_price: f64) -> IngestedCandleSnapshot {
    IngestedCandleSnapshot {
        candle_key: CandleKey::new(Coins::HYPE, interval),
        close_price,
    }
}

#[test]
fn m5_candle_runs_price_alert_evaluation() {
    let mut runtime = runtime_with_price_alert();

    let alerts = runtime.run_signals(SignalInput::Candle(snapshot(Interval::M5, 43.0)));

    assert_eq!(alerts.len(), 1);
    assert!(matches!(
        alerts[0].event,
        Event::ManualPriceTriggered { .. }
    ));
}

#[test]
fn non_m5_candle_skips_price_alert_evaluation() {
    let mut runtime = runtime_with_price_alert();

    let alerts = runtime.run_signals(SignalInput::Candle(snapshot(Interval::M15, 43.0)));

    assert!(alerts.is_empty());
}
