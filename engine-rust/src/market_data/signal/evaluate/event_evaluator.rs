use crate::market_data::{
    candle_store::CandleView,
    signal::{
        evaluate::evaluators::{
            indicator_evaluator::IndicatorEvaluator, price_evaluator::PriceEvaluator,
        },
        event::Alert,
        indicator_rules::IndicatorRule,
        price::PriceAlertService,
    },
    types::Coins,
};

pub struct EventEvaluator {
    price_evaluator: PriceEvaluator,
    indicator_evaluator: IndicatorEvaluator,
}

impl EventEvaluator {
    pub fn new() -> Self {
        EventEvaluator {
            price_evaluator: PriceEvaluator::new(),
            indicator_evaluator: IndicatorEvaluator::new(),
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
    pub fn evaluate_indicators(
        &mut self,
        view: &CandleView<'_>,
        rules: &[IndicatorRule],
    ) -> Vec<Alert> {
        self.indicator_evaluator.evaluate_indicator(view, rules)
    }
}
