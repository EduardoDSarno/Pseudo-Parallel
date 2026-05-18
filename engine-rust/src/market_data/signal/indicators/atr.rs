use std::collections::VecDeque;

use crate::market_data::{constans::MIN_CANDLES_FOR_ATR, types::Candle};

pub struct ATR {
    pub atr: Option<f64>,
    pub live_tr: f64,
    pub ratio: f64,
    pub spike_level: u64,
    pub open_time_ms: u64,
}

impl ATR {
    /* Build baseline ATR from the mean true range over closed candles */
    pub fn from_average(avg: f64) -> Self {
        ATR {
            atr: Some(avg),
            live_tr: 0.0,
            ratio: 0.0,
            spike_level: 0,
            open_time_ms: 0,
        }
    }

    /* Attach live true range and compute ratio against the baseline ATR */
    pub fn with_live(self, live_tr: f64, open_time_ms: u64) -> Option<Self> {
        let baseline = self.atr?;

        /* Ratio is the live candle move compared with the stable ATR baseline */
        Some(ATR {
            atr: Some(baseline),
            live_tr,
            ratio: live_tr / baseline,
            spike_level: self.spike_level,
            open_time_ms,
        })
    }

    pub fn baseline(&self) -> Option<f64> {
        self.atr
    }
}

/* This function calcualte the true range of a candle putting in consideration
possible gaps in price duee to low liquidity */
pub fn calculate_true_range(prev_candle: &Candle, curr_candle: &Candle) -> f64 {
    /* True range checks the normal candle size and possible gaps from last close */
    let current_candle_range = curr_candle.high_price - curr_candle.low_price;
    let gap_up_range = (curr_candle.high_price - prev_candle.close_price).abs();
    let gap_down_range = (curr_candle.low_price - prev_candle.close_price).abs();

    let max_gap_range = gap_down_range.max(gap_up_range);
    let tr = current_candle_range.max(max_gap_range);

    tr
}

/* This function will be used to calculate the ATR, by simply getting a vec of candle
references, looping through them, getting each TR, and calculating the Mean with all
of the TR's */
pub fn calculate_average_true_range(candle_buffer: &Vec<Candle>) -> Option<ATR> {
    if candle_buffer.len() < MIN_CANDLES_FOR_ATR {
        return None;
    }

    let mut true_ranges: VecDeque<f64> = VecDeque::new();

    /* Start at 1 because every TR needs the previous candle close */
    for i in 1..candle_buffer.len() {
        let tr = calculate_true_range(&candle_buffer[i - 1], &candle_buffer[i]);
        true_ranges.push_back(tr);
    }

    let avg = mean(&true_ranges)?;
    Some(ATR::from_average(avg))
}

/* Function helper to calucalte the  mean of the tr Vec use Option to be safer on
case of using NaN*/
fn mean(buf: &VecDeque<f64>) -> Option<f64> {
    if buf.is_empty() {
        return None;
    }
    Some(buf.iter().sum::<f64>() / buf.len() as f64)
}
