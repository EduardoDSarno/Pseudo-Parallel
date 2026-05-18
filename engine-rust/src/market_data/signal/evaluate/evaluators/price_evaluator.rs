use crate::market_data::{
    engine::MarketView,
    signal::event::Alert,
};

pub struct PriceEvaluator;

impl PriceEvaluator
{
    pub fn new() -> Self
    {
        PriceEvaluator
    }

    pub fn price_evaluator(&mut self, _view: &MarketView<'_>) -> Vec<Alert>
    {
        Vec::new()
    }
}
