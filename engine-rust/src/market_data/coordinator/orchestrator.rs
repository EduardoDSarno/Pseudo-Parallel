use crate::market_data::{
    alert_subscriptions::apply::apply_subscription,
    coordinator::{dispatch, market_update::MarketUpdate},
    runtime::MarketDataRuntime,
};

/* This part of the code is where the orchestration of thigs heppen feeding data, evaluating the data,
and running singal and logs */
impl MarketDataRuntime {
    pub fn process(&mut self, update: MarketUpdate) {
        match update {
            MarketUpdate::Candle(candle) => {
                tracing::debug!(
                    coin = ?candle.coin,
                    interval = ?candle.interval,
                    close = candle.close_price,
                    "orchestrator: processing candle update"
                );

                // feed data to candle store (currenlty just for candle)
                let snapshot = super::candle_ingest::apply_candle(&mut self.candle_store, candle);
                tracing::debug!(
                    coin = ?snapshot.candle_key.coin,
                    close = snapshot.current_price,
                    ?snapshot.candle_key,
                    "orchestrator: candle store ingest complete"
                );

                let coin = snapshot.candle_key.coin;
                let alerts = snapshot
                    .previous_price
                    .map(|previous_price| {
                        self.alert_service_mut().take_crossed(
                            coin,
                            previous_price,
                            snapshot.current_price,
                        )
                    })
                    .unwrap_or_default();
                tracing::debug!(
                    coin = ?coin,
                    alert_count = alerts.len(),
                    "orchestrator: signal evaluation complete"
                );

                if alerts.is_empty() {
                    tracing::trace!(coin = ?coin, "orchestrator: no alerts to dispatch");
                } else {
                    tracing::info!(
                        coin = ?coin,
                        alert_count = alerts.len(),
                        "orchestrator: dispatching alerts"
                    );
                    dispatch::dispatch_alerts(&alerts, self.alert_publisher.as_ref());
                }
            }
            // if the update is subscription apply it to the books
            MarketUpdate::Subscription(sub) => {
                if let Err(err) = apply_subscription(self, &sub) {
                    tracing::warn!(
                        error = %err,
                        ?sub,
                        "orchestrator: subscription apply failed"
                    );
                }
            }
        }
    }
}
