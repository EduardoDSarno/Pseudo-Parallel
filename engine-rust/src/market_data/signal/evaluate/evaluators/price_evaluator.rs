use crate::market_data::{
    signal::{
        event::{Alert, Event},
        price::PriceAlertService,
    },
    types::Coins,
};

pub struct PriceEvaluator;

impl PriceEvaluator {
    pub fn new() -> Self {
        PriceEvaluator
    }

    /* THis function will be called in every price update where we will check for alerts per coin looking up in alert
    service */
    pub fn evaluate_price(
        &self,
        alert_service: &PriceAlertService,
        coin: Coins,
        previous_price: f64,
        current_price: f64,
    ) -> Vec<Alert> {
        let mut alerts = Vec::new();

        for manual_alert in alert_service.crossed_above(coin, previous_price, current_price) {
            alerts.push(Alert::manual_price(
                coin,
                Event::ManualPriceTriggered {
                    trigger_price: manual_alert.trigger_price,
                    direction: manual_alert.direction,
                    previous_price,
                    current_price,
                },
            ));
        }

        for manual_alert in alert_service.crossed_below(coin, previous_price, current_price) {
            alerts.push(Alert::manual_price(
                coin,
                Event::ManualPriceTriggered {
                    trigger_price: manual_alert.trigger_price,
                    direction: manual_alert.direction,
                    previous_price,
                    current_price,
                },
            ));
        }

        alerts
    }
}
