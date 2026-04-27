
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    #[serde(rename = "t")]
    pub open_time_ms: u64, // open millis
    
    #[serde(rename = "T")]
    pub close_time_ms: u64, // close millis

    #[serde(rename = "s")]
    pub coin: String,

    #[serde(rename = "i")]
    pub interval: String,

    #[serde(rename = "o")]
    pub open_price: String,

    #[serde(rename = "c")]
    pub close_price: String,

    #[serde(rename = "h")]
    pub high_price: String,

    #[serde(rename = "l")]
    pub low_price: String,

    #[serde(rename = "v")]
    pub volume: String,

    #[serde(rename = "n")]
    pub trade_count: u64

}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Interval {
    #[serde(rename = "1m")]
    OneMinute,
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "1h")]
    OneHour,
}
