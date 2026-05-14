use serde::{Deserialize, Serialize};

use crate::market_data::constans::{H1_INTERVAL_MS, M15_INTERVAL_MS, M1_INTERVAL_MS, M5_INTERVAL_MS};

/* Enumerate intervals strings into hard values for easy use */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Interval
{
    #[serde(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    M5,
    #[serde(rename = "15m")]
    M15,
    #[serde(rename = "1h")]
    H1,
}

/* This implementation has the goal of making the interval time in MS match our interval
    enums, so request windows can be calculated from the interval itself. */
impl Interval
{
    pub fn to_ms(&self) -> u64
    {
        match self
        {
            Interval::M1 => M1_INTERVAL_MS,
            Interval::M5 => M5_INTERVAL_MS,
            Interval::M15 => M15_INTERVAL_MS,
            Interval::H1 => H1_INTERVAL_MS,
        }
    }
}

// This function will match the interval with the string
impl TryFrom<String> for Interval
{
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error>
    {
        match value.as_str()
        {
            "1m" => Ok(Interval::M1),
            "5m" => Ok(Interval::M5),
            "15m" => Ok(Interval::M15),
            "1h" => Ok(Interval::H1),
            other => Err(format!("unknown interval: {}", other)),
        }
    }
}
