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

#[derive(Debug)]
pub enum MarketEvent {
    Candle(Candle),

    // future
    // Trade(Trade),
    // L2Update(L2Book),
}

pub fn handle_events(engine: &mut Engine, event: MarketEvent) 
{
    match event 
    {
        MarketEvent::Candle(candle) => 
        {
            if let Some(closed) = engine.handle_candle(candle) 
            {
                if let Some(alert) = engine.evaluate_breakout(&closed) 
                {
                    println!("BREAKOUT: {:?}", alert);
                }
            }
        }
        // MarketEvent::Trade(trade) => { ... }
    }
}

