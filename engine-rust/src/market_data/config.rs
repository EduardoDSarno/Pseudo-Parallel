use crate::market_data::constans::{
    DEFAULT_ATR_BREAKOUT_RATIO, DEFAULT_LIVE_ATR_DEBUG_RATIO, DEFAULT_MAX_CLOSED_CANDLES,
};

#[derive(Debug, Clone, Copy)]
pub struct MarketDataConfig {
    pub max_closed_candles: usize,
    pub default_atr_breakout_ratio: f64,
    pub default_live_atr_debug_ratio: f64,
}

/* Defoult configuration for market data run */
impl Default for MarketDataConfig {
    fn default() -> Self {
        MarketDataConfig {
            max_closed_candles: DEFAULT_MAX_CLOSED_CANDLES,
            default_atr_breakout_ratio: DEFAULT_ATR_BREAKOUT_RATIO,
            default_live_atr_debug_ratio: DEFAULT_LIVE_ATR_DEBUG_RATIO,
        }
    }
}
