use crate::market_data::hyperliquid::protocols::data_models::candle::{self, CandleHL};

#[derive(Debug, Clone)]
pub struct Candle {
    pub open_time_ms: u64,
    pub close_time_ms: u64,
    pub coin: String,
    pub interval: String,
    pub open_price: f64,
    pub close_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub volume: f64,
    pub trade_count: u64,
}

impl TryFrom<CandleHL> for Candle 
{
    type Error = std::num::ParseFloatError;
    fn try_from(json: CandleHL) -> Result<Self, Self::Error> 
    {
        Ok(Candle 
        {
            open_time_ms: json.open_time_ms,
            close_time_ms: json.close_time_ms,
            coin: json.coin,
            interval: json.interval,
            open_price: json.open_price.parse()?,
            close_price: json.close_price.parse()?,
            high_price: json.high_price.parse()?,
            low_price: json.low_price.parse()?,
            volume: json.volume.parse()?,
            trade_count: json.trade_count,
        })
    }
}