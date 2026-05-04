use crate::market_data::{engine::Engine, types::candle::{Candle, CandleKey, Coins, Interval}};

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


pub fn handle_candle_event(engine: &mut Engine, candle: Candle) 
{
    if let Some(closed) = engine.handle_candle(candle) 
    {
        if let Some(alert) = engine.evaluate_breakout(&closed) 
        {
            println!("BREAKOUT: {:?}", alert);
        }
    }
}

