use serde::{Deserialize, Serialize};

use crate::market_data::constans::M5_INTERVAL_MS;

/* Enumerate intervals strings into hard values for easy use */
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Interval {
    #[serde(rename = "5m")]
    M5,
}

/* This implementation has the goal of making the interval time in MS match our interval
enums, so request windows can be calculated from the interval itself. */
impl Interval {
    pub fn to_ms(&self) -> u64 {
        M5_INTERVAL_MS
    }
}

// This function will match the interval with the string
impl TryFrom<String> for Interval {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "5m" => Ok(Interval::M5),
            other => Err(format!("unknown interval: {}", other)),
        }
    }
}
