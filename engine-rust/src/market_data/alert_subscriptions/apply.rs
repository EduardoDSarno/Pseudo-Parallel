use std::error::Error;

use crate::market_data::{
    alert_subscriptions::command::{
        PriceSubscriptionSpec, SubscriptionCommand, SubscriptionManager,
    },
    runtime::MarketDataRuntime,
    signal::price::{alert::ManualPriceAlert, build_manual_price_alert},
    types::{CandleKey, Coins, Interval},
};

pub fn apply_subscription(
    runtime: &mut MarketDataRuntime,
    sub: &SubscriptionManager,
) -> Result<(), Box<dyn Error>> {
    match sub.command {
        SubscriptionCommand::Subscribe => {
            let alert = resolve_price_alert(runtime, &sub.price)?;
            runtime.alert_service_mut().subscribe(alert)?;
        }
        SubscriptionCommand::Unsubscribe => {
            let alert = resolve_price_alert(runtime, &sub.price).map_err(|err| {
                tracing::error!(
                    error = %err,
                    spec = ?sub.price,
                    "apply_subscription: could not resolve price alert for unsubscribe"
                );
                err
            })?;
            let key = alert.alert_key().map_err(|err| {
                tracing::error!(
                    error = %err,
                    ?alert,
                    "apply_subscription: invalid price alert key for unsubscribe"
                );
                err
            })?;
            runtime
                .alert_service_mut()
                .unsubscribe(key)
                .map_err(|err| {
                    tracing::warn!(
                        error = %err,
                        ?alert,
                        "apply_subscription: price alert unsubscribe failed"
                    );
                    err
                })?;
        }
    }
    Ok(())
}

fn resolve_price_alert(
    runtime: &MarketDataRuntime,
    spec: &PriceSubscriptionSpec,
) -> Result<ManualPriceAlert, Box<dyn Error>> {
    let reference_price = if spec.direction.is_none() {
        reference_price_for_coin(runtime, spec.coin)
    } else {
        None
    };
    build_manual_price_alert(
        spec.coin,
        spec.trigger_price,
        spec.direction,
        reference_price,
    )
}

/* Use the latest M5 close when direction was not provided */
fn reference_price_for_coin(runtime: &MarketDataRuntime, coin: Coins) -> Option<f64> {
    let key = CandleKey::new(coin, Interval::M5);
    runtime
        .candle_store
        .last_seen(&key)
        .map(|candle| candle.close_price)
}
