use std::error::Error;

use crate::market_data::{
    alert_subscriptions::{
        command::{SubscriptionCommand, SubscriptionManager, SubscriptionType},
        price_resolve::build_manual_price_alert,
    },
    runtime::MarketDataRuntime,
};

pub fn apply_subscription(
    runtime: &mut MarketDataRuntime,
    sub: &SubscriptionManager,
) -> Result<(), Box<dyn Error>> {
    match sub.command {
        SubscriptionCommand::Subscribe => match &sub.sub_type {
            SubscriptionType::Price(spec) => {
                let alert = build_manual_price_alert(
                    runtime,
                    spec.coin,
                    spec.trigger_price,
                    spec.direction,
                )?;
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
                let alert = build_manual_price_alert(
                    runtime,
                    spec.coin,
                    spec.trigger_price,
                    spec.direction,
                )
                .map_err(|err| {
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
                runtime.alert_service_mut().unsubscribe(key).map_err(|err| {
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
