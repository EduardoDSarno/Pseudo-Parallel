use crate::market_data::{
    engine::MarketView,
    signal::{
        evaluate::evaluators::indicators::atr_evaluator::AtrEvaluator,
        event::{Alert, Event},
    },
};

pub struct IndicatorEvaluator
{
    atr_evaluator: AtrEvaluator,
}

impl IndicatorEvaluator
{
    pub fn new(_max_closed_candles: usize) -> Self
    {
        IndicatorEvaluator
        {
            atr_evaluator: AtrEvaluator::new(),
        }
    }

    /* THis function will contain every indicator evaluation and return a vec of alers
        if any */
    pub fn evaluate_indicator(&mut self, view: &MarketView<'_>) -> Vec<Alert>
    {
        let mut alerts = Vec::new();

        if let Some(atr_alert) = self.atr_evaluator.evaluate_atr(view)
        {
            if let Some(baseline) = atr_alert.atr.baseline()
            {
                alerts.push(Alert::new(atr_alert.key, Event::AtrBreakout
                {
                    atr: baseline,
                    live_tr: atr_alert.atr.live_tr,
                    ratio: atr_alert.atr.ratio,
                    spike_level: atr_alert.atr.spike_level,
                    open_time_ms: atr_alert.atr.open_time_ms,
                }));
            }
        }

        alerts
    }
}
