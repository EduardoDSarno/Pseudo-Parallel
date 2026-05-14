use crate::market_data::{hyperliquid::protocols::data_models::candle::CandleHL, types::{Coins, Interval}};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Candle {
    pub open_time_ms: u64,
    pub close_time_ms: u64,
    pub coin: Coins,
    pub interval: Interval,
    pub open_price: f64,
    pub close_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub volume: f64,
    pub trade_count: u64,
}

/* THis implementation will try converting the HL candle string field received from
    the serde parse into a numeric struct Candle which we can use for market_data*/
impl TryFrom<CandleHL> for Candle 
{
    type Error = Box<dyn std::error::Error>;
    fn try_from(json: CandleHL) -> Result<Self, Self::Error> 
    {
        Ok(Candle 
        {
            open_time_ms: json.open_time_ms,
            close_time_ms: json.close_time_ms,
            coin:Coins::try_from(json.coin)? , // Result
            interval: Interval::try_from(json.interval)?, // Result
            open_price: json.open_price.parse()?,
            close_price: json.close_price.parse()?,
            high_price: json.high_price.parse()?,
            low_price: json.low_price.parse()?,
            volume: json.volume.parse()?,
            trade_count: json.trade_count,
        })
    }
}

