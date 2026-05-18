use std::collections::{HashMap, VecDeque};
use crate::market_data::types::{Candle, CandleKey};

// THis struct will be responsible for handling Candle data and to store in memory 
// the current data we need
pub struct Engine
{
    pub buffers: HashMap<CandleKey, VecDeque<Candle>>,
    last_seen: HashMap<CandleKey, Candle>,
    pub(super) max_closed_candles: usize,
}


impl Engine
{
    // Lazy approach to create a new engine simply empty Hashmaps
    pub fn new(max_closed_candles: usize) -> Self
    {
        Engine 
        {
            buffers: HashMap::new(),
            last_seen: HashMap::new(),
            max_closed_candles,
        }
    }

    pub fn last_seen(&self, candle_key: &CandleKey) -> Option<&Candle>
    {
        self.last_seen.get(candle_key)
    }

    pub fn set_last_seen(&mut self, candle_key: CandleKey, candle: Candle)
    {
        self.last_seen.insert(candle_key, candle);
    }

    pub fn push_closed_candle(&mut self, candle_key: CandleKey, candle: Candle)
    {
        let buf = self.buffers.entry(candle_key)
            .or_insert_with(VecDeque::new);
        buf.push_back(candle);

        if buf.len() > self.max_closed_candles
        {
            buf.pop_front();
        }
    }
}
