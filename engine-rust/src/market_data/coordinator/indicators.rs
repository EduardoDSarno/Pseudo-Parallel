use crate::market_data::{
    engine::MarketView,
    signal::{evaluate::event_evaluator::EventEvaluator, event::Alert},
};

pub(crate) fn evaluate_indicator_alerts(
    event_evaluator: &mut EventEvaluator,
    view: &MarketView<'_>,
) -> Vec<Alert> {
    event_evaluator.evaluate_indicators(view)
}
