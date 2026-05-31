use crate::market_data::{
    coordinator::{
        dispatch, market_update::MarketUpdate, signal_input::SignalInput,
    },
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

                // feed data to engine (currenlty just for candle)
                let snapshot = super::candle_ingest::apply_candle(&mut self.engine, candle);
                tracing::debug!(
                    coin = ?snapshot.candle_key.coin,
                    close = snapshot.close_price,
                    ?snapshot.candle_key,
                    "orchestrator: engine ingest complete"
                );

                // running singnals (currenlty just for candle)
                let coin = snapshot.candle_key.coin;
                let alerts = self.run_signals(SignalInput::Candle(snapshot));
                tracing::debug!(
                    coin = ?coin,
                    alert_count = alerts.len(),
                    "orchestrator: signal evaluation complete"
                );

                // logging alerts in the future steam will be added
                if alerts.is_empty() {
                    tracing::trace!(coin = ?coin, "orchestrator: no alerts to dispatch");
                } else {
                    tracing::info!(
                        coin = ?coin,
                        alert_count = alerts.len(),
                        "orchestrator: dispatching alerts"
                    );
                    dispatch::log_alerts(&alerts);
                }
            }
        }
    }
}
