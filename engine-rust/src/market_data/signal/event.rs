use crate::market_data::{
    engine::Engine,
    signal::indicators::evaluate_indicators::IndicatorEvaluator,
    types::{Candle, CandleKey},
};

#[derive(Debug)]
pub enum Event
{
    ATR
    {
        atr: f64,
        live_tr: f64,
        ratio: f64,
        spike_level: u64,
        open_time_ms: u64,
    },
}
#[derive(Debug)]
pub struct BreakoutAlert 
{
    pub key: CandleKey,
    pub event: Event,
}

impl BreakoutAlert {
    pub fn new(key: CandleKey, event: Event) -> Self 
    {
        BreakoutAlert { key, event }
    }
}


pub fn handle_candle_event(engine: &mut Engine, indicator_evaluator: &mut IndicatorEvaluator, candle: Candle)
{
    let live_candle = candle.clone();
    engine.handle_candle(candle);

    if let Some(alert) = indicator_evaluator.evaluate_live_breakout(engine, &live_candle)
    {
        let Event::ATR { atr, live_tr, ratio, spike_level, open_time_ms } = alert.event;

        tracing::info!
        (
            coin = ?alert.key.coin,
            interval = ?alert.key.interval,
            open_time = open_time_ms,
            atr = atr,
            live_tr = live_tr,
            ratio = ratio,
            spike_level = spike_level,
            "LIVE BREAKOUT detected"
        );
    }
}
