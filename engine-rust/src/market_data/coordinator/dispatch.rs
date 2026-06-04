use crate::market_data::signal::event::{Alert, Event};

/* Currenlty we are just loggin the alerts in the future we will be dispatchign to a stream */
pub fn log_alerts(alerts: &[Alert]) {
    for alert in alerts {
        match &alert.event {
            Event::AtrBreakout {
                indicator_rule_id,
                atr,
                live_tr,
                ratio,
                spike_level,
                open_time_ms,
            } => {
                let key = alert
                    .key
                    .as_ref()
                    .expect("ATR alerts always carry a candle key");
                tracing::info!(
                    coin = ?alert.coin,
                    indicator_rule_id = indicator_rule_id.0,
                    interval = ?key.interval,
                    open_time = open_time_ms,
                    atr = atr,
                    live_tr = live_tr,
                    ratio = ratio,
                    spike_level = spike_level,
                    "ATR breakout detected"
                );
            }
            Event::ManualPriceTriggered {
                trigger_price,
                direction,
                previous_price,
                current_price,
            } => {
                tracing::info!(
                    coin = ?alert.coin,
                    trigger_price = trigger_price,
                    direction = ?direction,
                    previous_price = previous_price,
                    current_price = current_price,
                    "Manual price alert triggered"
                );
            }
        }
    }
}
