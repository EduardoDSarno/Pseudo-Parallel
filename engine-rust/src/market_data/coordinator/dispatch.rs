use tokio::sync::mpsc;

use crate::{
    market_data::signal::price::alert::TriggeredPriceAlert,
    redis_transport::outgoing::OutgoingManualPriceAlert,
};

/* publisher is None when no redis publisher has been wired up (e.g. in tests) —
alerts are still evaluated, just not sent anywhere. */
pub fn dispatch_alerts(
    alerts: &[TriggeredPriceAlert],
    publisher: Option<&mpsc::UnboundedSender<String>>,
) {
    for alert in alerts {
        let outgoing_alert = OutgoingManualPriceAlert::new(
            alert.coin,
            alert.trigger_price,
            alert.direction,
            alert.current_price,
        );

        let message = match outgoing_alert.convert() {
            Ok(message) => message,
            Err(err) => {
                tracing::error!(error = ?err, "Failed to serialize outgoing manual price alert");
                continue;
            }
        };

        if let Some(sender) = publisher {
            if let Err(err) = sender.send(message) {
                tracing::error!(error = %err, "failed to send alert to publisher channel");
            }
        }
    }
}
