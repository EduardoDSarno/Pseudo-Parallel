use std::error::Error;

use crate::market_data::{
    alert_subscriptions::command::{
        PriceSubscriptionSpec, SubscriptionCommand, SubscriptionManager, SubscriptionType,
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
        SubscriptionCommand::Subscribe => match &sub.sub_type {
            SubscriptionType::Price(spec) => {
                let alert = resolve_price_alert(runtime, spec)?;
                runtime.alert_service_mut().subscribe(alert)?;
            }
            SubscriptionType::Indicator(ind) => {
                runtime
                    .indicator_rule_service_mut()
                    .subscribe(ind.key.clone(), ind.kind.clone());
            }
        },
        SubscriptionCommand::Unsubscribe => match &sub.sub_type {
            SubscriptionType::Price(spec) => {
                let alert = resolve_price_alert(runtime, spec).map_err(|err| {
                    tracing::error!(
                        error = %err,
                        ?spec,
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
            SubscriptionType::Indicator(ind) => {
                runtime
                    .indicator_rule_service_mut()
                    .unsubscribe(ind.key.clone(), ind.kind.clone())
                    .map_err(|err| {
                        tracing::warn!(
                            error = %err,
                            key = ?ind.key,
                            "apply_subscription: indicator rule unsubscribe failed"
                        );
                        err
                    })?;
            }
        },
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

/* Same source as price crossing: last M5 tick price, else latest M5 candle close. */
fn reference_price_for_coin(runtime: &MarketDataRuntime, coin: Coins) -> Option<f64> {
    if let Some(price) = runtime.last_market_price(coin) {
        return Some(price);
    }
    let key = CandleKey::new(coin, Interval::M5);
    let view = runtime.candle_store.market_view(&key)?;
    view.closed_candles
        .back()
        .map(|c| c.close_price)
        .or(Some(view.live_candle.close_price))
}
