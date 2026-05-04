use std::{collections::VecDeque};
use crate::market_data::types::candle::Candle;

/* This function calcualte the true range of a candle putting in consideration 
    possible gaps in price duee to low liquidity */
pub fn calculate_true_range(prev_candle: &Candle, curr_candle: &Candle) -> f64
{
   let current_candle_range = curr_candle.high_price - curr_candle.low_price;
   let gap_up_range   = (curr_candle.high_price - prev_candle.close_price).abs();
   let gap_down_range = (curr_candle.low_price  - prev_candle.close_price).abs();

   let max_gap_range = gap_down_range.max(gap_up_range);
   let tr = current_candle_range.max(max_gap_range);

   tr
}

/* This function will be used to calculate the ATR, by simply getting a vec of candle
     references, looping through them, getting each TR, and calculating the Mean with all
     of the TR's */
pub fn calculate_average_true_range(candle_buffer : &Vec<Candle>) ->Option<f64>
{
    if candle_buffer.len() < 2 
    { 
        return None; 
    }

    let mut true_ranges:VecDeque<f64> = VecDeque::new();

    for i in 1..candle_buffer.len() 
    {
        let tr = calculate_true_range(&candle_buffer[i - 1], 
                                           &candle_buffer[i]);

        true_ranges.push_back(tr);
    }

    let atr = mean(&true_ranges);
    atr
}

/* Function helper to calucalte the  mean of the tr Vec use Option to be safer on 
    case of using NaN*/
fn mean(buf: &VecDeque<f64>) -> Option<f64> 
{
    if buf.is_empty() 
    { 
        return None; 
    }
    Some(buf.iter().sum::<f64>() / buf.len() as f64)
}