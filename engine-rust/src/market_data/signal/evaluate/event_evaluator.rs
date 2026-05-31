use crate::market_data::{
    engine::MarketView,
    signal::{
        evaluate::evaluators::{
            indicator_evaluator::IndicatorEvaluator, price_evaluator::PriceEvaluator,
        },
        event::Alert,
        price::PriceAlertService,
    },
    types::Coins,
};

pub struct EventEvaluator {
    price_evaluator: PriceEvaluator,
    indicator_evaluator: IndicatorEvaluator,
}

impl EventEvaluator {
    pub fn new(max_closed_candles: usize) -> Self {
        EventEvaluator {
            price_evaluator: PriceEvaluator::new(),
            indicator_evaluator: IndicatorEvaluator::new(max_closed_candles),
        }
    }

    pub fn evaluate_price(
        &self,
        alert_service: &PriceAlertService, // borrowed each call
        coin: Coins,
        previous_price: f64,
        current_price: f64,
    ) -> Vec<Alert> {
        self.price_evaluator
            .evaluate_price(alert_service, coin, previous_price, current_price)
    }
    pub fn evaluate_indicators(&mut self, view: &MarketView<'_>) -> Vec<Alert> {
        self.indicator_evaluator.evaluate_indicator(view)
    }
}
