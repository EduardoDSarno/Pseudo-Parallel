use crate::market_data::{
    candle_store::CandleView,
    signal::{
        evaluate::evaluators::indicators::atr_evaluator::AtrEvaluator,
        event::{Alert, Event},
        indicator_rules::{IndicatorRule, IndicatorRuleKind},
    },
};

pub struct IndicatorEvaluator {
    atr_evaluator: AtrEvaluator,
}

impl IndicatorEvaluator {
    pub fn new() -> Self {
        IndicatorEvaluator {
            atr_evaluator: AtrEvaluator::new(),
        }
    }

    /* THis function will contain every indicator evaluation and return a vec of alers
    if any */
    pub fn evaluate_indicator(
        &mut self,
        view: &CandleView<'_>,
        rules: &[IndicatorRule],
    ) -> Vec<Alert> {
        let mut alerts = Vec::new();

        for rule in rules {
            if rule.key != *view.key {
                tracing::warn!(
                    rule_key = ?rule.key,
                    view_key = ?view.key,
                    "indicator rule skipped for mismatched candle key"
                );
                continue;
            }

            match &rule.kind {
                IndicatorRuleKind::Atr(atr_rule) => {
                    if let Some(atr_alert) =
                        self.atr_evaluator.evaluate_atr(view, rule.id, atr_rule)
                    {
                        if let Some(baseline) = atr_alert.atr.baseline() {
                            alerts.push(Alert::indicator(
                                atr_alert.key,
                                Event::AtrBreakout {
                                    indicator_rule_id: atr_alert.rule_id,
                                    atr: baseline,
                                    live_tr: atr_alert.atr.live_tr,
                                    ratio: atr_alert.atr.ratio,
                                    spike_level: atr_alert.atr.spike_level,
                                    open_time_ms: atr_alert.atr.open_time_ms,
                                },
                            ));
                        }
                    }
                }
            }
        }

        alerts
    }
}
