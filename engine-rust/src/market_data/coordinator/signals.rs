use crate::market_data::{
    constans::PRICE_ALERT_INTERVAL_MS,
    coordinator::{candle_ingest::IngestedCandleSnapshot, indicators, signal_input::SignalInput},
    runtime::MarketDataRuntime,
    signal::event::Alert,
};

impl MarketDataRuntime {
    /* Will run signals based on the input type received */
    pub(crate) fn run_signals(&mut self, input: SignalInput) -> Vec<Alert> {
        match input {
            SignalInput::Candle(snapshot) => self.run_signals_for_candle(snapshot),
        }
    }

    /* This function will run signals for candle by checking alerts for price changed and evaluate indicator*/
    fn run_signals_for_candle(&mut self, snapshot: IngestedCandleSnapshot) -> Vec<Alert> {
        let coin = snapshot.candle_key.coin;
        let price_alerts = if snapshot.candle_key.interval.to_ms() == PRICE_ALERT_INTERVAL_MS {
            self.price_alerts_if_coin_price_changed(coin, snapshot.close_price)
        } else {
            tracing::trace!(
                coin = ?coin,
                interval = ?snapshot.candle_key.interval,
                "orchestrator: skipping price signals for non-price interval"
            );
            Vec::new()
        };
        tracing::debug!(
            coin = ?coin,
            price_alert_count = price_alerts.len(),
            "orchestrator: price signal pass complete"
        );

        let mut alerts = price_alerts;

        if let Some(view) = self.candle_store.market_view(&snapshot.candle_key) {
            if snapshot.bar_just_closed {
                let indicator_rules = self
                    .indicator_rule_service()
                    .rules_for_key(&snapshot.candle_key);
                let indicator_alerts = indicators::evaluate_indicator_alerts(
                    &mut self.event_evaluator,
                    &view,
                    &indicator_rules,
                );
                tracing::debug!(
                    candle_key = ?snapshot.candle_key,
                    indicator_rule_count = indicator_rules.len(),
                    indicator_alert_count = indicator_alerts.len(),
                    "orchestrator: indicator signal pass complete"
                );
                alerts.extend(indicator_alerts);
            } else {
                tracing::trace!(
                    candle_key = ?snapshot.candle_key,
                    "orchestrator: skipping indicators until bar close (closed-candle v1)"
                );
            }
        } else {
            let closed = self.candle_store.closed_buffer(&snapshot.candle_key);
            let has_closed_buffer = closed.is_some();
            let closed_len = closed.map(|buf| buf.len()).unwrap_or(0);
            let has_last_seen = self.candle_store.last_seen(&snapshot.candle_key).is_some();
            let max_closed_candles = self.max_closed_candles();

            let waiting_for_closed_history =
                has_last_seen && (!has_closed_buffer || closed_len == 0);

            if waiting_for_closed_history {
                tracing::debug!(
                    ?snapshot.candle_key,
                    has_closed_buffer,
                    closed_len,
                    has_last_seen,
                    max_closed_candles,
                    "orchestrator: skipping indicators, no closed candle history yet"
                );
            } else {
                tracing::warn!(
                    ?snapshot.candle_key,
                    has_closed_buffer,
                    closed_len,
                    has_last_seen,
                    max_closed_candles,
                    "orchestrator: skipping indicators, no candle view"
                );
            }
        }

        alerts
    }
}
