use crate::market_data::types::CandleKey;

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
