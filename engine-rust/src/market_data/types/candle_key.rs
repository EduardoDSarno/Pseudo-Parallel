use serde::{Deserialize, Serialize};

use crate::market_data::types::{Candle, Coins, Interval};

// Candle Key for subscription and data analysis
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CandleKey {
    pub coin: Coins,
    pub interval: Interval,
}

impl CandleKey {
    pub fn new(coin: Coins, interval: Interval) -> CandleKey {
        CandleKey { coin, interval }
    }

    pub fn create_key_from_candle(candle: &Candle) -> CandleKey {
        CandleKey::new(candle.coin.clone(), candle.interval.clone())
    }
}
