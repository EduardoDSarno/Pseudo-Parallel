use std::collections::HashMap;

use crate::market_data::{
    constans::{ATR_BREAKOUT_RATIO, LIVE_ATR_DEBUG_RATIO, NO_SPIKE_LEVEL},
    engine::MarketView,
    signal::{
        evaluate::evaluate_atr::{AtrEvaluation, AtrEvaluator},
        event::{Alert, Event},
    },
    types::CandleKey,
};

#[derive(Debug, Clone)]
struct LiveAlertState
{
    open_time_ms: u64,
    last_spike_level: u64,
}

pub struct Evaluator
{
    atr_evaluator: AtrEvaluator,
    atr_alerts: HashMap<CandleKey, LiveAlertState>,
}

impl Evaluator
{
    pub fn new(max_closed_candles: usize) -> Self
    {
        Evaluator
        {
            atr_evaluator: AtrEvaluator::new(max_closed_candles),
            atr_alerts: HashMap::new(),
        }
    }

    pub fn evaluate(&mut self, view: MarketView<'_>) -> Vec<Alert>
    {
        let mut alerts = Vec::new();

        if let Some(atr) = self.atr_evaluator.evaluate_live_atr(view)
        {
            if let Some(alert) = self.evaluate_atr_breakout(atr)
            {
                alerts.push(alert);
            }
        }

        alerts
    }

    fn evaluate_atr_breakout(&mut self, atr: AtrEvaluation) -> Option<Alert>
    {
        let spike_level = (atr.ratio / ATR_BREAKOUT_RATIO).floor() as u64;

        if atr.ratio >= ATR_BREAKOUT_RATIO * LIVE_ATR_DEBUG_RATIO
        {
            tracing::debug!(coin = ?atr.key.coin, interval = ?atr.key.interval, open_time = atr.open_time_ms, live_tr = atr.live_tr, atr = atr.atr, ratio = atr.ratio, spike_level = spike_level, "Live ATR evaluated");
        }

        if spike_level == NO_SPIKE_LEVEL
        {
            return None;
        }

        let state = self.atr_alerts.entry(atr.key.clone()).or_insert(LiveAlertState
        {
            open_time_ms: atr.open_time_ms,
            last_spike_level: NO_SPIKE_LEVEL,
        });

        if state.open_time_ms != atr.open_time_ms
        {
            state.open_time_ms = atr.open_time_ms;
            state.last_spike_level = NO_SPIKE_LEVEL;
        }

        if spike_level <= state.last_spike_level
        {
            return None;
        }

        state.last_spike_level = spike_level;
        Some(Alert::new(atr.key, Event::AtrEvaluation))
    }
}
