use crate::market_data::{engine::Engine, types::candle::{Candle, CandleKey}};

#[derive(Debug)]
pub enum Event
{
    ATR {prev_atr: f64, breakout_atr: f64},
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


pub fn handle_candle_event(engine: &mut Engine, candle: Candle) 
{
    
    if let Some(closed) = engine.handle_candle(candle) 
    {
        if let Some(alert) = engine.evaluate_breakout(&closed) 
        {
            tracing::info!(coin = ?alert.key.coin, interval = ?alert.key.interval, event = ?alert.event, "BREAKOUT detected");
        }
    }
}


