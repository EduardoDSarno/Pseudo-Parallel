use std::error::Error;

use crate::market_data::{
    runtime::MarketDataRuntime,
    alert_subscriptions::command::{SubscriptionCommand, SubscriptionManager, SubscriptionType},
};

pub fn apply_subscription(
    runtime: &mut MarketDataRuntime,
    sub: &SubscriptionManager,
) -> Result<(), Box<dyn Error>> {
    match sub.command {
        SubscriptionCommand::Subscribe => match &sub.sub_type {
            SubscriptionType::Price(alert) => {
                runtime.alert_service_mut().subscribe(alert.clone())?;
            }
            SubscriptionType::Indicator(ind) => {
                runtime
                    .indicator_rule_service_mut()
                    .subscribe(ind.key.clone(), ind.kind.clone());
            }
        },
        SubscriptionCommand::Unsubscribe => match &sub.sub_type {
            SubscriptionType::Price(alert) => {
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
