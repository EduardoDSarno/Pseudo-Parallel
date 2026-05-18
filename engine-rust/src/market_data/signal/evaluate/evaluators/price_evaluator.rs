use std::collections::HashMap;

use crate::market_data::{
    engine::MarketView,
    signal::{
        event::{Alert, Event},
        price::{AlertBook, ManualPriceAlert, ManualPriceDirection},
    },
    types::Coins,
};

const TEST_HYPE_ABOVE_PRICE: f64 = 47.6;
const TEST_HYPE_BELOW_PRICE: f64 = 47.4;

pub struct PriceEvaluator {
    alert_book: AlertBook,
    last_price_by_coin: HashMap<Coins, f64>,
}

impl PriceEvaluator {
    pub fn new() -> Self {
        let mut alert_book = AlertBook::new();
        /* Hardcoded for runtime testing until the alert stream starts feeding this book */
        let test_alerts = [
            ManualPriceAlert::new(
                Coins::HYPE,
                TEST_HYPE_ABOVE_PRICE,
                ManualPriceDirection::Above,
            ),
            ManualPriceAlert::new(
                Coins::HYPE,
                TEST_HYPE_BELOW_PRICE,
                ManualPriceDirection::Below,
            ),
        ];

        for alert in test_alerts {
            if let Err(err) = alert_book.insert_alert(alert) {
                tracing::error!(error = %err, "Could not insert hardcoded manual price alert");
            }
        }

        PriceEvaluator {
            alert_book,
            last_price_by_coin: HashMap::new(),
        }
    }

    pub fn price_evaluator(&mut self, view: &MarketView<'_>) -> Vec<Alert> {
        let coin = view.live_candle.coin;
        let current_price = view.live_candle.close_price;

        /* First price is only used to create the base point, because crossing needs 2 prices */
        let previous_price = match self.last_price_by_coin.insert(coin, current_price) {
            Some(price) => price,
            None => return Vec::new(),
        };

        let mut alerts = Vec::new();

        /* Up and down use different maps, so a same price can have both directions */
        for manual_alert in
            self.alert_book
                .alerts_crossed_above(coin, previous_price, current_price)
        {
            alerts.push(Alert::new(
                view.key.clone(),
                Event::ManualPriceTriggered {
                    trigger_price: manual_alert.trigger_price,
                    direction: manual_alert.direction,
                    previous_price,
                    current_price,
                },
            ));
        }

        for manual_alert in
            self.alert_book
            .alerts_crossed_below(coin, previous_price, current_price)
        {
            alerts.push(Alert::new(
                view.key.clone(),
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
