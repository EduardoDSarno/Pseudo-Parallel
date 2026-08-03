use crate::market_data::constans::DEFAULT_MAX_CLOSED_CANDLES;
use crate::market_data::types::{CandleKey, Coins, Interval};

#[derive(Debug, Clone)]
pub struct MarketDataConfig {
    pub max_closed_candles: usize,
    pub candle_keys: Vec<CandleKey>,
}

/* Defoult configuration for market data run */
impl Default for MarketDataConfig {
    fn default() -> Self {
        MarketDataConfig {
            max_closed_candles: DEFAULT_MAX_CLOSED_CANDLES,
            candle_keys: vec![CandleKey::new(Coins::HYPE, Interval::M5)],
        }
    }
}
