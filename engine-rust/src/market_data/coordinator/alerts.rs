use crate::market_data::{
    runtime::MarketDataRuntime,
    signal::{
        event::{Alert, Event},
        price::{LevelKey, PriceKey},
    },
    types::Coins,
};

impl MarketDataRuntime 
{
    /* This function will coordinate the alert triggergin on price change. It's job is to update the price
        then evaluate if any alerts were triggered, them from the alerts trigger we get the key levels in our price books
        of which will be disarmed*/
    pub(crate) fn price_alerts_if_coin_price_changed(
        &mut self,
        coin: Coins,
        current_price: f64,
    ) -> Vec<Alert> {
        let Some(previous_price) = self.last_market_price(coin) else {
            self.set_last_market_price(coin, current_price);
            return Vec::new();
        };

        if previous_price == current_price {
            return Vec::new();
        }

        self.set_last_market_price(coin, current_price);

        let alerts = self.event_evaluator.evaluate_price(
            self.alert_service(),
            coin,
            previous_price,
            current_price,
        );

        let keys = level_keys_from_manual_price_alerts_fired(&alerts);
        self.alert_service_mut().disarm_levels(keys);
        alerts
    }
}

/* This functions is responsible for each alert triggeed to create a key of the levelKey of the alers and return the
    combination of them */
fn level_keys_from_manual_price_alerts_fired(alerts: &[Alert]) -> Vec<LevelKey> {
    let mut keys = Vec::new();

    for alert in alerts {
        let Event::ManualPriceTriggered {
            trigger_price,
            direction,
            ..
        } = &alert.event
        else {
            continue;
        };

        let Some(price_key) = PriceKey::from_price(*trigger_price) else {
            tracing::warn!(
                coin = ?alert.coin,
                trigger_price,
                "skipping disarm: invalid trigger price"
            );
            continue;
        };

        keys.push(LevelKey::new(alert.coin, price_key, *direction));
    }

    keys
}
