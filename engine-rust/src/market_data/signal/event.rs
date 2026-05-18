use crate::market_data::types::CandleKey;

#[derive(Debug)]
pub enum Event
{
    AtrEvaluation,
}
#[derive(Debug)]
pub struct Alert
{
    pub key: CandleKey,
    pub event: Event,
}

impl Alert {
    pub fn new(key: CandleKey, event: Event) -> Self 
    {
        Alert { key, event }
    }
}
