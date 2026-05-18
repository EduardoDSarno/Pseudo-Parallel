use std::collections::HashMap;

use crate::market_data::{
    constans::{ATR_BREAKOUT_RATIO, LIVE_ATR_DEBUG_RATIO, MIN_CANDLES_FOR_ATR, NO_SPIKE_LEVEL},
    engine::MarketView,
    signal::indicators::atr::{calculate_average_true_range, calculate_true_range, ATR},
    types::CandleKey,
};

#[derive(Debug, Clone)]
struct LiveAlertState {
    open_time_ms: u64,
    last_spike_level: u64,
}

pub struct AtrAlert {
    pub key: CandleKey,
    pub atr: ATR,
}

pub struct AtrEvaluator {
    live_alerts: HashMap<CandleKey, LiveAlertState>,
}

impl AtrEvaluator {
    pub fn new() -> Self {
        AtrEvaluator {
            live_alerts: HashMap::new(),
        }
    }

    pub fn evaluate_atr(&mut self, view: &MarketView<'_>) -> Option<AtrAlert> {
        if view.closed_candles.len() < MIN_CANDLES_FOR_ATR {
            tracing::debug!(coin = ?view.key.coin, interval = ?view.key.interval, len = view.closed_candles.len(), min = MIN_CANDLES_FOR_ATR, "ATR buffer warming up");
            return None;
        }

        let atr_input = view.closed_candles.iter().cloned().collect();
        let latest_closed = view.closed_candles.back()?;

        /* ATR baseline uses closed candles, but live_tr uses the candle forming now */
        let live_tr = calculate_true_range(latest_closed, view.live_candle);
        let mut atr = calculate_average_true_range(&atr_input)?
            .with_live(live_tr, view.live_candle.open_time_ms)?;

        /* Level 1 means first threshold, level 2 means second threshold, and so on */
        let spike_level = (atr.ratio / ATR_BREAKOUT_RATIO).floor() as u64;
        atr.spike_level = spike_level;

        if atr.ratio >= ATR_BREAKOUT_RATIO * LIVE_ATR_DEBUG_RATIO {
            tracing::debug!(coin = ?view.key.coin, interval = ?view.key.interval, open_time = atr.open_time_ms, live_tr = atr.live_tr, atr = ?atr.baseline(), ratio = atr.ratio, spike_level = spike_level, "Live ATR evaluated");
        }

        if spike_level == NO_SPIKE_LEVEL {
            return None;
        }

        /* This state stops the same candle from alerting the same spike level again */
        let state = self
            .live_alerts
            .entry(view.key.clone())
            .or_insert(LiveAlertState {
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
            key: view.key.clone(),
            atr,
        })
    }
}
