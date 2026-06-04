use std::collections::HashMap;

use crate::market_data::{
    candle_store::CandleView,
    constans::{MIN_CANDLES_FOR_ATR, NO_SPIKE_LEVEL},
    signal::indicator_rules::{AtrRule, IndicatorRuleId},
    signal::indicators_logic::atr::{calculate_average_true_range, calculate_true_range, ATR},
    types::CandleKey,
};

#[derive(Debug, Clone)]
struct LiveAlertState {
    open_time_ms: u64,
    last_spike_level: u64,
}

pub struct AtrAlert {
    pub rule_id: IndicatorRuleId,
    pub key: CandleKey,
    pub atr: ATR,
}

pub struct AtrEvaluator {
    live_alerts: HashMap<IndicatorRuleId, LiveAlertState>,
}

impl AtrEvaluator {
    pub fn new() -> Self {
        AtrEvaluator {
            live_alerts: HashMap::new(),
        }
    }

    pub fn evaluate_atr(
        &mut self,
        view: &CandleView<'_>,
        rule_id: IndicatorRuleId,
        rule: &AtrRule,
    ) -> Option<AtrAlert> {
        if view.closed_candles.len() < MIN_CANDLES_FOR_ATR {
            tracing::debug!(coin = ?view.key.coin, interval = ?view.key.interval, len = view.closed_candles.len(), min = MIN_CANDLES_FOR_ATR, "ATR buffer warming up");
            return None;
        }

        let closed = view.closed_candles;
        let previous_closed = closed.get(closed.len() - 2)?;
        let latest_closed = closed.back()?;

        let atr_input = closed.iter().cloned().collect();
        let closed_bar_tr = calculate_true_range(previous_closed, latest_closed);
        let mut atr = calculate_average_true_range(&atr_input)?
            .with_live(closed_bar_tr, latest_closed.open_time_ms)?;

        /* Level 1 means first threshold, level 2 means second threshold, and so on */
        let spike_level = (atr.ratio / rule.breakout_ratio).floor() as u64;
        atr.spike_level = spike_level;

        if atr.ratio >= rule.breakout_ratio * rule.debug_ratio {
            tracing::debug!(coin = ?view.key.coin, interval = ?view.key.interval, indicator_rule_id = rule_id.0, open_time = atr.open_time_ms, closed_bar_tr = atr.live_tr, atr = ?atr.baseline(), ratio = atr.ratio, spike_level = spike_level, "ATR evaluated on closed candles");
        }

        if spike_level == NO_SPIKE_LEVEL {
            return None;
        }

        /* This state stops the same candle from alerting the same spike level again */
        let state = self.live_alerts.entry(rule_id).or_insert(LiveAlertState {
            open_time_ms: atr.open_time_ms,
            last_spike_level: NO_SPIKE_LEVEL,
        });

        if state.open_time_ms != atr.open_time_ms {
            state.open_time_ms = atr.open_time_ms;
            state.last_spike_level = NO_SPIKE_LEVEL;
        }

        if spike_level <= state.last_spike_level {
            return None;
        }

        state.last_spike_level = spike_level;
        Some(AtrAlert {
            rule_id,
            key: view.key.clone(),
            atr,
        })
    }
}
