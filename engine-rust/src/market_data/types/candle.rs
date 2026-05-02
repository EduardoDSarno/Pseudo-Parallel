use serde::{Serialize, Deserialize};
use crate::market_data::{hyperliquid::protocols::data_models::candle::{CandleHL}};

#[derive(Debug, Clone)]
pub struct Candle {
    pub open_time_ms: u64,
    pub close_time_ms: u64,
    pub coin: COINS,
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
            coin:COINS::try_from(json.coin)? , // Result
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

/* Enumerate intervals strings into hard values for easy use */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Interval {
    #[serde(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    M5,
    #[serde(rename = "15m")]
    M15,
    #[serde(rename = "1h")]
    H1,
}

// This function will match the inverval with the string
impl TryFrom<String> for Interval {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "1m" => Ok(Interval::M1),
            "5m" => Ok(Interval::M5),
            "15m" => Ok(Interval::M15),
            "1h" => Ok(Interval::H1),
            other => Err(format!("unknown interval: {}", other)),
        }
    }
}

/*Enumerate coin strings into our hard values */
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum COINS
{
    HYPE,
    BTC,
    ETH,
}

/* Implementation to handle conversion */
impl TryFrom<String> for COINS {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "HYPE" => Ok(COINS::HYPE),
            "BTC" => Ok(COINS::BTC),
            "ETH" => Ok(COINS::ETH),
            other => Err(format!("unknown coin: {}", other)),
        }
    }
}


// Candle Key for subscription and Data analysis
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CandleKey
{
    coin: COINS,
    interval: Interval
}

impl CandleKey{
    pub fn new(coin: COINS, interval:Interval) -> Result<CandleKey, Box<dyn std::error::Error>>
    {
        let candle_key = CandleKey
        {
            coin: coin,
            interval: interval
        };
        Ok(candle_key)
    }
}

