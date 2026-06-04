use crate::market_data::{
    candle_store::CandleView,
    signal::{
        evaluate::event_evaluator::EventEvaluator, event::Alert, indicator_rules::IndicatorRule,
    },
};

pub(crate) fn evaluate_indicator_alerts(
    event_evaluator: &mut EventEvaluator,
    view: &CandleView<'_>,
    rules: &[IndicatorRule],
) -> Vec<Alert> {
    event_evaluator.evaluate_indicators(view, rules)
}
