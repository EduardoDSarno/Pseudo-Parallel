use std::collections::VecDeque;
use crate::market_data::{constans::DEFAULT_MAX_CLOSED_CANDLES, types::Candle};

// It will get the close prices from the candles isnide VecDeque in the Hashmap buffer 
// and it will add them and divide by the quantity of candles giving us the
// SIMPLE MOVING AVAREAGE
pub fn _calculate_sma(buf: &VecDeque<Candle>) -> Option<f64> 
{
    if buf.len() < DEFAULT_MAX_CLOSED_CANDLES 
    {
        return None;
    }
    let sum: f64 = buf.iter().map(|c| c.close_price).sum();
    Some(sum / buf.len() as f64)
}
