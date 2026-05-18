use std::collections::HashMap;

use crate::market_data::{
    constans::{ATR_BREAKOUT_RATIO, LIVE_ATR_DEBUG_RATIO, MIN_VALID_ATR, NO_SPIKE_LEVEL},
    engine::Engine,
    signal::{
        event::{BreakoutAlert, Event},
        indicators::atr::{calculate_average_true_range, calculate_true_range},
    },
    types::{Candle, CandleKey},
};

#[derive(Debug, Clone)]
struct LiveAlertState
{
    open_time_ms: u64,
    last_spike_level: u64,
}

pub struct Evaluator
{
    live_alerts: HashMap<CandleKey, LiveAlertState>,
    max_closed_candles: usize,
}

impl Evaluator
{
    pub fn new(max_closed_candles: usize) -> Self
    {
        Evaluator
        {
            live_alerts: HashMap::new(),
            max_closed_candles,
        }
    }

    /* This checks the forming candle against an ATR baseline built only from closed candles.
       The alert state prevents repeated alerts until a higher ATR multiple is reached. */
    pub fn evaluate_live_breakout(&mut self, engine: &Engine, candle: &Candle) -> Option<BreakoutAlert>
    {
        let key = CandleKey::create_key_from_candle(candle);
        let buf = engine.buffers.get(&key)?;

        if buf.len() < self.max_closed_candles
        {
            tracing::debug!(coin = ?key.coin, interval = ?key.interval, len = buf.len(), max = self.max_closed_candles, "Buffer warming up");
            return None;
        }

        let atr_input: Vec<Candle> = buf.iter().cloned().collect();
        let atr = calculate_average_true_range(&atr_input)?;
        if atr <= MIN_VALID_ATR
        {
            tracing::warn!(coin = ?key.coin, interval = ?key.interval, atr = atr, "ATR is not valid for live breakout");
            return None;
        }

        let latest_closed = buf.back()?;
        let live_tr = calculate_true_range(latest_closed, candle);
        let ratio = live_tr / atr;
        let spike_level = (ratio / ATR_BREAKOUT_RATIO).floor() as u64;

        if ratio >= ATR_BREAKOUT_RATIO * LIVE_ATR_DEBUG_RATIO
        {
            tracing::debug!(coin = ?key.coin, interval = ?key.interval, open_time = candle.open_time_ms, live_tr = live_tr, atr = atr, ratio = ratio, spike_level = spike_level, "Live ATR evaluated");
        }

        if spike_level == NO_SPIKE_LEVEL
        {
            return None;
        }

        let state = self.live_alerts.entry(key.clone()).or_insert(LiveAlertState
        {
            open_time_ms: candle.open_time_ms,
            last_spike_level: NO_SPIKE_LEVEL,
        });

        if state.open_time_ms != candle.open_time_ms
        {
            state.open_time_ms = candle.open_time_ms;
            state.last_spike_level = NO_SPIKE_LEVEL;
        }

        if spike_level <= state.last_spike_level
        {
            return None;
        }

        state.last_spike_level = spike_level;
        Some(BreakoutAlert::new(key, Event::ATR
        {
            atr,
            live_tr,
            ratio,
            spike_level,
            open_time_ms: candle.open_time_ms,
        }))
    }
}
