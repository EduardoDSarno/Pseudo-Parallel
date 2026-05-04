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
            
            let Event::ATR { prev_atr, breakout_atr } = alert.event;
            
            // Print debug statement for Break out Detection
            tracing::info!
            (
                coin = ?alert.key.coin,
                interval = ?alert.key.interval,
                atr = prev_atr,
                tr = breakout_atr,
                difference = breakout_atr - prev_atr,
                ratio = breakout_atr / prev_atr,
                "BREAKOUT detected"
            );
            
        }
    }
}


