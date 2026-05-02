use std::collections::{HashMap, VecDeque};

use tokio::time::Interval;

use crate::market_data::types::candle::{COINS, Candle, CandleKey};

// THis struct will be responsible for handling Candle data and to store in memory 
// the current data we need
pub struct Engine
{
    buffers: HashMap<CandleKey, VecDeque<Candle>>,
    last_seen: HashMap<CandleKey, Candle>
}

