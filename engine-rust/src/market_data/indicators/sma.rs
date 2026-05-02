use std::collections::VecDeque;
use crate::market_data::{constans::MAX_LENGTH_CANDLE_BUFFER, types::candle::Candle};

pub fn calculate_sma(buf: &VecDeque<Candle>) -> Option<f64> 
{
    if buf.len() < MAX_LENGTH_CANDLE_BUFFER 
    {
        return None;
    }
    let sum: f64 = buf.iter().map(|c| c.close_price).sum();
    Some(sum / buf.len() as f64)
}