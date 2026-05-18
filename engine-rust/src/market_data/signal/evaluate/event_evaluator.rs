use crate::market_data::{
    engine::MarketView,
    signal::{
        evaluate::evaluators::{
            indicator_evaluator::IndicatorEvaluator,
            price_evaluator::PriceEvaluator,
        },
        event::Alert,
    },
};

pub struct EventEvaluator
{
    price_evaluator: PriceEvaluator,
    indicator_evaluator: IndicatorEvaluator,
}

impl EventEvaluator
{
    pub fn new(max_closed_candles: usize) -> Self
    {
        EventEvaluator
        {
            price_evaluator: PriceEvaluator::new(),
            indicator_evaluator: IndicatorEvaluator::new(max_closed_candles),
        }
    }

    pub fn evaluate(&mut self, view: &MarketView<'_>) -> Vec<Alert>
    {
        let mut alerts = Vec::new();

        alerts.extend(self.price_evaluator.price_evaluator(view));
        alerts.extend(self.indicator_evaluator.evaluate_indicator(view));

        alerts
    }
}
