use crate::market_data::constans::DEFAULT_MAX_CLOSED_CANDLES;

#[derive(Debug, Clone, Copy)]
pub struct MarketDataConfig
{
    pub max_closed_candles: usize,
}

impl Default for MarketDataConfig
{
    fn default() -> Self
    {
        MarketDataConfig
        {
            max_closed_candles: DEFAULT_MAX_CLOSED_CANDLES,
        }
    }
}
