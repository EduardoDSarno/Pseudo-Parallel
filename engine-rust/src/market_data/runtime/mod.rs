#[cfg(test)]
use std::error::Error;

use tokio::sync::mpsc;

#[cfg(test)]
use crate::market_data::alert_subscriptions::{
    apply::apply_subscription, command::SubscriptionManager,
};
use crate::market_data::{
    candle_store::CandleStore, config::MarketDataConfig, signal::price::PriceAlertService,
};

/* This file is the composition root for the market data runtime.
It owns all the pieces (candle store, alert service, evaluators) in one place.
The coordinator holds the playbook (process + live_loop) but does not own state —
main holds a MarketDataRuntime and the live loop calls into it. */

/* MarketDataRuntime owns candle state, active price alerts, and alert delivery */
pub struct MarketDataRuntime {
    pub candle_store: CandleStore,
    alert_service: PriceAlertService,
    /* None until main wires up the redis publisher — dispatch_alerts skips publishing
    when this is unset (e.g. in tests that don't need a live redis connection). */
    pub(crate) alert_publisher: Option<mpsc::UnboundedSender<String>>,
}

impl MarketDataRuntime {
    pub fn new(config: MarketDataConfig) -> Self {
        MarketDataRuntime {
            candle_store: CandleStore::new(config.max_closed_candles),
            alert_service: PriceAlertService::new(),
            alert_publisher: None,
        }
    }

    /* Wires the redis alert publisher sender in after construction — main calls this
    once spawn_alert_publisher has set up the background task. */
    pub fn set_alert_publisher(&mut self, sender: mpsc::UnboundedSender<String>) {
        self.alert_publisher = Some(sender);
    }

    /* Mutable access for subscription updates and crossing checks */
    pub fn alert_service_mut(&mut self) -> &mut PriceAlertService {
        &mut self.alert_service
    }

    #[cfg(test)]
    pub fn load_signal_subscriptions(
        &mut self,
        subs: Vec<SubscriptionManager>,
    ) -> Result<(), Box<dyn Error>> {
        for sub in subs {
            apply_subscription(self, &sub)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
